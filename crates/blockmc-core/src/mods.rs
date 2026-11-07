//! Pure (non-network) logic: file safety, checksums, version choice, install records.
use crate::{error::CoreError, instance::{InstanceStore, Loader}};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind { Mod, Resourcepack, Shader }

impl ProjectKind {
    pub fn api_name(self) -> &'static str { match self { Self::Mod => "mod", Self::Resourcepack => "resourcepack", Self::Shader => "shader" } }
    pub fn folder(self) -> &'static str { match self { Self::Mod => "mods", Self::Resourcepack => "resourcepacks", Self::Shader => "shaderpacks" } }
    pub fn ext(self) -> &'static str { match self { Self::Mod => ".jar", _ => ".zip" } }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileHashes { #[serde(default)] pub sha512: String }
#[derive(Debug, Clone, Deserialize)]
pub struct VersionFile { pub hashes: FileHashes, pub url: String, pub filename: String, #[serde(default)] pub primary: bool }
#[derive(Debug, Clone, Deserialize)]
pub struct Dependency { pub project_id: Option<String>, #[serde(default)] pub dependency_type: String }
#[derive(Debug, Clone, Deserialize)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    #[serde(default)] pub version_number: String,
    #[serde(default)] pub version_type: String,
    #[serde(default)] pub files: Vec<VersionFile>,
    #[serde(default)] pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub project_id: String,
    pub version_id: String,
    pub title: String,
    pub version_number: String,
    pub kind: ProjectKind,
    pub filename: String,
    pub sha512: String,
    /// Project ids of required dependencies (used to warn before removing something others need).
    #[serde(default)]
    pub requires: Vec<String>,
}

pub fn compatible_loaders(l: Loader) -> Result<Vec<&'static str>, CoreError> {
    Ok(match l {
        Loader::Vanilla => return Err(CoreError::Unsupported("Vanilla instances can't use mods. Create a Fabric, Forge, NeoForge or Quilt instance instead.".into())),
        Loader::Fabric => vec!["fabric"],
        Loader::Quilt => vec!["quilt", "fabric"],
        Loader::Forge => vec!["forge"],
        Loader::Neoforge => vec!["neoforge"],
    })
}

pub fn valid_project_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn safe_filename(name: &str, kind: ProjectKind) -> Result<String, CoreError> { safe_name_with_ext(name, kind.ext()) }

pub fn safe_name_with_ext(name: &str, ext: &str) -> Result<String, CoreError> {
    let bad = name.is_empty() || name.len() > 200 || name.starts_with('.') || name.contains("..")
        || name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'));
    if bad || !name.to_lowercase().ends_with(ext) {
        return Err(CoreError::UnsafeFilename(name.to_string()));
    }
    Ok(name.to_string())
}

pub fn ensure_trusted_url(url: &str) -> Result<(), CoreError> {
    if url.starts_with("https://cdn.modrinth.com/") { Ok(()) } else { Err(CoreError::UntrustedUrl(url.to_string())) }
}

/// Newest release if there is one, otherwise the newest version of any type.
pub fn pick_version(versions: &[Version]) -> Option<&Version> {
    versions.iter().find(|v| v.version_type == "release").or_else(|| versions.first())
}

pub fn pick_primary(v: &Version) -> Option<&VersionFile> {
    v.files.iter().find(|f| f.primary).or_else(|| v.files.first())
}

pub fn required_deps(v: &Version) -> Vec<String> {
    v.dependencies.iter().filter(|d| d.dependency_type == "required").filter_map(|d| d.project_id.clone()).collect()
}

/// Moves a fully downloaded `.part` file into place only if its checksum matches.
pub fn finalize_download(part: &Path, dest: &Path, actual: &str, expected: &str) -> Result<(), CoreError> {
    if expected.is_empty() { let _ = fs::remove_file(part); return Err(CoreError::HashMissing) }
    if !actual.eq_ignore_ascii_case(expected) { let _ = fs::remove_file(part); return Err(CoreError::HashMismatch) }
    fs::rename(part, dest)?;
    Ok(())
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest { pub items: Vec<InstalledMod> }

impl Manifest {
    fn path(idir: &Path) -> PathBuf { idir.join("installed.json") }
    pub fn load(idir: &Path) -> Result<Self, CoreError> {
        match fs::read(Self::path(idir)) {
            Ok(b) => Ok(serde_json::from_slice(&b)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, idir: &Path) -> Result<(), CoreError> {
        let p = Self::path(idir);
        let tmp = p.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(tmp, p)?;
        Ok(())
    }
}

pub fn content_dir(idir: &Path, kind: ProjectKind) -> PathBuf { idir.join("minecraft").join(kind.folder()) }

pub fn list_installed(store: &InstanceStore, id: &str) -> Result<Vec<InstalledMod>, CoreError> {
    Ok(Manifest::load(&store.instance_dir(id)?)?.items)
}

/// Titles of installed projects that list `project_id` as a required dependency.
pub fn dependents(store: &InstanceStore, id: &str, project_id: &str) -> Result<Vec<String>, CoreError> {
    Ok(Manifest::load(&store.instance_dir(id)?)?.items.into_iter()
        .filter(|i| i.requires.iter().any(|r| r == project_id)).map(|i| i.title).collect())
}

pub fn remove_installed(store: &InstanceStore, id: &str, project_id: &str) -> Result<(), CoreError> {
    let idir = store.instance_dir(id)?;
    let mut m = Manifest::load(&idir)?;
    let pos = m.items.iter().position(|i| i.project_id == project_id).ok_or(CoreError::NotFound)?;
    let item = m.items[pos].clone();
    let file = content_dir(&idir, item.kind).join(safe_filename(&item.filename, item.kind)?);
    if file.exists() { fs::remove_file(file)?; }
    m.items.remove(pos);
    m.save(&idir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::Loader;

    const SAMPLE: &str = r#"[{"id":"v2","project_id":"P1","version_number":"2.0","version_type":"beta",
      "files":[{"hashes":{"sha1":"x","sha512":"abc"},"url":"https://cdn.modrinth.com/data/P1/v2.jar","filename":"m-2.0.jar","primary":true}],
      "dependencies":[{"project_id":"DEP1","version_id":null,"dependency_type":"required"},{"project_id":"OPT","dependency_type":"optional"},{"project_id":null,"dependency_type":"required"}]},
      {"id":"v1","project_id":"P1","version_number":"1.0","version_type":"release","files":[],"dependencies":[]}]"#;

    #[test]
    fn parses_modrinth_json_and_picks_release() {
        let v: Vec<Version> = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(pick_version(&v).unwrap().id, "v1");
        assert_eq!(pick_primary(&v[0]).unwrap().filename, "m-2.0.jar");
        assert_eq!(required_deps(&v[0]), vec!["DEP1".to_string()]);
        assert!(pick_primary(&v[1]).is_none());
    }

    #[test]
    fn filename_safety() {
        assert!(safe_filename("sodium-0.5.jar", ProjectKind::Mod).is_ok());
        assert!(safe_filename("My Pack.ZIP", ProjectKind::Shader).is_ok());
        for bad in ["../evil.jar", "a/b.jar", "a\\b.jar", ".hidden.jar", "x.exe", "", "a:b.jar"] {
            assert!(safe_filename(bad, ProjectKind::Mod).is_err(), "{bad}");
        }
        assert!(safe_filename("pack.jar", ProjectKind::Resourcepack).is_err());
    }

    #[test]
    fn only_modrinth_cdn_is_trusted() {
        assert!(ensure_trusted_url("https://cdn.modrinth.com/data/x.jar").is_ok());
        assert!(ensure_trusted_url("http://cdn.modrinth.com/x.jar").is_err());
        assert!(ensure_trusted_url("https://cdn.modrinth.com.evil.io/x.jar").is_err());
    }

    #[test]
    fn checksum_gate() {
        let t = tempfile::tempdir().unwrap();
        let part = t.path().join("a.jar.part");
        let dest = t.path().join("a.jar");
        fs::write(&part, b"data").unwrap();
        assert!(matches!(finalize_download(&part, &dest, "aaa", "bbb"), Err(CoreError::HashMismatch)));
        assert!(!part.exists() && !dest.exists());
        fs::write(&part, b"data").unwrap();
        assert!(matches!(finalize_download(&part, &dest, "aaa", ""), Err(CoreError::HashMissing)));
        fs::write(&part, b"data").unwrap();
        finalize_download(&part, &dest, "ABC", "abc").unwrap();
        assert!(dest.exists() && !part.exists());
    }

    #[test]
    fn remove_deletes_file_and_record_only_in_its_instance() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        let a = s.create("A", "1.20.1", Loader::Fabric).unwrap();
        let b = s.create("B", "1.20.1", Loader::Fabric).unwrap();
        let mut ids = vec![];
        for inst in [&a, &b] {
            let idir = s.instance_dir(&inst.id).unwrap();
            let dir = content_dir(&idir, ProjectKind::Mod);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("m.jar"), b"x").unwrap();
            let m = Manifest { items: vec![InstalledMod { project_id: "P".into(), version_id: "v".into(), title: "M".into(), version_number: "1".into(), kind: ProjectKind::Mod, filename: "m.jar".into(), sha512: "h".into(), requires: vec![] }] };
            m.save(&idir).unwrap();
            ids.push(idir);
        }
        remove_installed(&s, &a.id, "P").unwrap();
        assert!(!content_dir(&ids[0], ProjectKind::Mod).join("m.jar").exists());
        assert!(content_dir(&ids[1], ProjectKind::Mod).join("m.jar").exists());
        assert_eq!(list_installed(&s, &a.id).unwrap().len(), 0);
        assert_eq!(list_installed(&s, &b.id).unwrap().len(), 1);
    }

    #[test]
    fn finds_dependents() {
        let t = tempfile::tempdir().unwrap();
        let s = InstanceStore::new(t.path());
        let a = s.create("A", "1.20.1", Loader::Fabric).unwrap();
        let idir = s.instance_dir(&a.id).unwrap();
        let mk = |p: &str, req: Vec<&str>| InstalledMod { project_id: p.into(), version_id: "v".into(), title: p.to_uppercase(), version_number: "1".into(), kind: ProjectKind::Mod, filename: format!("{p}.jar"), sha512: "h".into(), requires: req.into_iter().map(String::from).collect() };
        Manifest { items: vec![mk("lib", vec![]), mk("a", vec!["lib"]), mk("b", vec!["lib"]), mk("c", vec![])] }.save(&idir).unwrap();
        assert_eq!(dependents(&s, &a.id, "lib").unwrap(), vec!["A".to_string(), "B".to_string()]);
        assert!(dependents(&s, &a.id, "c").unwrap().is_empty());
    }

    #[test]
    fn vanilla_has_no_mod_loaders() {
        assert!(compatible_loaders(Loader::Vanilla).is_err());
        assert_eq!(compatible_loaders(Loader::Quilt).unwrap(), vec!["quilt", "fabric"]);
    }
}
