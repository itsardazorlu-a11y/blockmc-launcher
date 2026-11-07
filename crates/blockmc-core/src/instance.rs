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

    pub fn delete(&self, id: &str) -> Result<(), CoreError> {
        let dir = self.instance_dir(id)?;
        if !dir.join("instance.json").exists() { return Err(CoreError::NotFound) }
        fs::remove_dir_all(dir)?;
        Ok(())
    }
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
