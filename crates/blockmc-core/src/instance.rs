//! Isolated instances. Every instance lives in its own directory under
//! `<root>/instances/<id>/` and nothing is ever shared between instances.
use serde::{Deserialize, Serialize};
use crate::error::CoreError;
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader { Vanilla, Fabric, Forge, Neoforge, Quilt }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    pub loader_version: Option<String>,
}

pub struct InstanceStore { root: PathBuf }

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for c in name.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() { out.push(c) }
        else if !out.ends_with('-') { out.push('-') }
    }
    let s = out.trim_matches('-').to_string();
    s.chars().take(60).collect::<String>().trim_matches('-').to_string()
}

impl InstanceStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }
    fn instances_dir(&self) -> PathBuf { self.root.join("instances") }

    /// Resolves an instance directory; rejects any id that could escape the root.
    pub fn instance_dir(&self, id: &str) -> Result<PathBuf, CoreError> {
        if !valid_id(id) { return Err(CoreError::InvalidId) }
        Ok(self.instances_dir().join(id))
    }

    pub fn create(&self, name: &str, mc_version: &str, loader: Loader) -> Result<InstanceConfig, CoreError> {
        let base = slugify(name);
        if base.is_empty() { return Err(CoreError::EmptyName) }
        fs::create_dir_all(self.instances_dir())?;
        let mut id = base.clone();
        let mut n = 2;
        while self.instance_dir(&id)?.exists() { id = format!("{base}-{n}"); n += 1; }
        let dir = self.instance_dir(&id)?;
        // create_dir (not _all) fails if another process claimed the id first.
        fs::create_dir(&dir).map_err(|e| if e.kind() == std::io::ErrorKind::AlreadyExists { CoreError::AlreadyExists } else { e.into() })?;
        fs::create_dir(dir.join("minecraft"))?;
        let cfg = InstanceConfig { id, name: name.trim().to_string(), mc_version: mc_version.to_string(), loader, loader_version: None };
        write_atomic(&dir.join("instance.json"), &serde_json::to_vec_pretty(&cfg)?)?;
        Ok(cfg)
    }

    pub fn get(&self, id: &str) -> Result<InstanceConfig, CoreError> {
        let p = self.instance_dir(id)?.join("instance.json");
        let bytes = fs::read(&p).map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { CoreError::NotFound } else { e.into() })?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn list(&self) -> Result<Vec<InstanceConfig>, CoreError> {
        let dir = self.instances_dir();
        if !dir.exists() { return Ok(vec![]) }
        let mut out = vec![];
        for entry in fs::read_dir(dir)? {
            let path = entry?.path().join("instance.json");
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(cfg) = serde_json::from_slice::<InstanceConfig>(&bytes) { out.push(cfg) }
            }
        }
        out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(out)
    }

    pub fn rename(&self, id: &str, new_name: &str) -> Result<InstanceConfig, CoreError> {
        let name = new_name.trim();
        if name.is_empty() { return Err(CoreError::EmptyName) }
        let mut cfg = self.get(id)?;
        cfg.name = name.to_string();
        write_atomic(&self.instance_dir(id)?.join("instance.json"), &serde_json::to_vec_pretty(&cfg)?)?;
        Ok(cfg)
    }

    /// Full independent copy of an instance (its own folder, mods and worlds).
    pub fn duplicate(&self, id: &str) -> Result<InstanceConfig, CoreError> {
        let mut cfg = self.get(id)?;
        let src = self.instance_dir(id)?;
        let base = slugify(&format!("{} copy", cfg.name));
        let base = if base.is_empty() { "copy".to_string() } else { base };
        let mut new_id = base.clone();
        let mut n = 2;
        while self.instance_dir(&new_id)?.exists() { new_id = format!("{base}-{n}"); n += 1; }
        let dest = self.instance_dir(&new_id)?;
        if let Err(e) = copy_dir(&src, &dest) { let _ = fs::remove_dir_all(&dest); return Err(e.into()) }
        cfg.id = new_id;
        cfg.name = format!("{} (copy)", cfg.name);
        if let Err(e) = write_atomic(&dest.join("instance.json"), &serde_json::to_vec_pretty(&cfg)?) { let _ = fs::remove_dir_all(&dest); return Err(e.into()) }
        Ok(cfg)
    }

    pub fn delete(&self, id: &str) -> Result<(), CoreError> {
        let dir = self.instance_dir(id)?;
        if !dir.join("instance.json").exists() { return Err(CoreError::NotFound) }
        fs::remove_dir_all(dir)?;
        Ok(())
    }
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = to.join(entry.file_name());
        if ty.is_dir() { copy_dir(&entry.path(), &target)?; }
        else if ty.is_file() { fs::copy(entry.path(), &target)?; }
    }
    Ok(())
}

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, data)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_list_delete_roundtrip() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        let a = s.create("My Pack", "1.20.1", Loader::Fabric).unwrap();
        let b = s.create("My Pack", "1.20.1", Loader::Fabric).unwrap();
        assert_eq!(a.id, "my-pack");
        assert_eq!(b.id, "my-pack-2");
        assert_eq!(s.list().unwrap().len(), 2);
        s.delete("my-pack").unwrap();
        assert_eq!(s.list().unwrap().len(), 1);
    }

    #[test]
    fn instances_are_isolated_directories() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        let a = s.create("A", "1.21", Loader::Vanilla).unwrap();
        let b = s.create("B", "1.21", Loader::Quilt).unwrap();
        assert_ne!(s.instance_dir(&a.id).unwrap(), s.instance_dir(&b.id).unwrap());
    }

    #[test]
    fn rename_and_duplicate_are_independent() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        let a = s.create("Base", "1.21", Loader::Fabric).unwrap();
        let mods = s.instance_dir(&a.id).unwrap().join("minecraft").join("mods");
        fs::create_dir_all(&mods).unwrap();
        fs::write(mods.join("x.jar"), b"1").unwrap();

        let r = s.rename(&a.id, "  Renamed ").unwrap();
        assert_eq!(r.name, "Renamed");
        assert_eq!(s.get(&a.id).unwrap().name, "Renamed");
        assert!(matches!(s.rename(&a.id, " "), Err(CoreError::EmptyName)));

        let b = s.duplicate(&a.id).unwrap();
        assert_ne!(a.id, b.id);
        assert_eq!(b.name, "Renamed (copy)");
        let copied = s.instance_dir(&b.id).unwrap().join("minecraft").join("mods").join("x.jar");
        assert!(copied.exists());
        fs::write(&copied, b"changed").unwrap();
        assert_eq!(fs::read(mods.join("x.jar")).unwrap(), b"1");
        assert_eq!(s.get(&b.id).unwrap().id, b.id);
    }

    #[test]
    fn rejects_path_traversal() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        for bad in ["../x", "a/b", "", "-a", "A", ".."] {
            assert!(matches!(s.instance_dir(bad), Err(CoreError::InvalidId)), "{bad}");
        }
        assert!(matches!(s.delete("../x"), Err(CoreError::InvalidId)));
    }

    #[test]
    fn empty_name_rejected_and_bad_json_skipped() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        assert!(matches!(s.create("  ", "1.20", Loader::Vanilla), Err(CoreError::EmptyName)));
        let d = t.path().join("instances/broken");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("instance.json"), b"{nope").unwrap();
        assert!(s.list().unwrap().is_empty());
    }
}
