//! Modrinth modpack (.mrpack) parsing and safe extraction. Format: modrinth.index.json plus
//! optional `overrides/` and `client-overrides/` folders inside a zip.
use crate::{error::CoreError, instance::Loader};
use serde::Deserialize;
use std::{collections::HashMap, fs, io::Read, path::{Path, PathBuf}};

const MAX_OVERRIDE_FILE: u64 = 512 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct Index {
    #[serde(default)] pub name: String,
    pub dependencies: HashMap<String, String>,
    #[serde(default)] pub files: Vec<IndexFile>,
}
#[derive(Debug, Deserialize)]
pub struct IndexHashes { #[serde(default)] pub sha512: String }
#[derive(Debug, Deserialize)]
pub struct IndexEnv { #[serde(default)] pub client: String }
#[derive(Debug, Deserialize)]
pub struct IndexFile {
    pub path: String,
    pub hashes: IndexHashes,
    #[serde(default)] pub downloads: Vec<String>,
    #[serde(default)] pub env: Option<IndexEnv>,
}

fn zip_err(e: zip::result::ZipError) -> CoreError { CoreError::Unsupported(format!("Invalid modpack file: {e}")) }

pub fn parse_index(json: &[u8]) -> Result<Index, CoreError> { Ok(serde_json::from_slice(json)?) }

pub fn read_index(pack: &Path) -> Result<Index, CoreError> {
    let mut z = zip::ZipArchive::new(fs::File::open(pack)?).map_err(zip_err)?;
    let mut entry = z.by_name("modrinth.index.json").map_err(zip_err)?;
    let mut buf = vec![];
    (&mut entry).take(20_000_000).read_to_end(&mut buf)?;
    parse_index(&buf)
}

/// Minecraft version and mod loader the pack was built for.
pub fn target(index: &Index) -> Result<(String, Loader), CoreError> {
    let mc = index.dependencies.get("minecraft").cloned()
        .ok_or_else(|| CoreError::Unsupported("This modpack does not say which Minecraft version it needs.".into()))?;
    let d = &index.dependencies;
    let loader = if d.contains_key("fabric-loader") { Loader::Fabric }
        else if d.contains_key("quilt-loader") { Loader::Quilt }
        else if d.contains_key("neoforge") { Loader::Neoforge }
        else if d.contains_key("forge") { Loader::Forge }
        else { Loader::Vanilla };
    Ok((mc, loader))
}

/// Relative path inside the instance; rejects anything that could escape it.
pub fn safe_rel_path(p: &str) -> Result<PathBuf, CoreError> {
    let bad = || CoreError::UnsafeFilename(p.to_string());
    if p.is_empty() || p.contains('\\') || p.contains(':') || p.starts_with('/') { return Err(bad()) }
    let mut out = PathBuf::new();
    for part in p.split('/') {
        if part.is_empty() || part == "." || part == ".." || part.chars().any(|c| c.is_control()) { return Err(bad()) }
        out.push(part);
    }
    Ok(out)
}

pub fn trusted_pack_url(url: &str) -> bool {
    ["https://cdn.modrinth.com/", "https://github.com/", "https://raw.githubusercontent.com/", "https://gitlab.com/"]
        .iter().any(|p| url.starts_with(p))
}

pub fn extract_overrides(pack: &Path, mc_dir: &Path) -> Result<(), CoreError> {
    let mut z = zip::ZipArchive::new(fs::File::open(pack)?).map_err(zip_err)?;
    for i in 0..z.len() {
        let mut e = z.by_index(i).map_err(zip_err)?;
        let Some(name) = e.enclosed_name() else { continue };
        let rel = if let Ok(r) = name.strip_prefix("overrides") { r.to_path_buf() }
            else if let Ok(r) = name.strip_prefix("client-overrides") { r.to_path_buf() }
            else { continue };
        if rel.as_os_str().is_empty() { continue }
        let dest = mc_dir.join(&rel);
        if e.is_dir() { fs::create_dir_all(&dest)?; continue }
        if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
        let mut out = fs::File::create(&dest)?;
        std::io::copy(&mut (&mut e).take(MAX_OVERRIDE_FILE), &mut out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = r#"{"formatVersion":1,"game":"minecraft","name":"Cool Pack","versionId":"1",
      "files":[{"path":"mods/a.jar","hashes":{"sha1":"x","sha512":"abc"},"downloads":["https://cdn.modrinth.com/data/a.jar"],"env":{"client":"required","server":"required"},"fileSize":1},
               {"path":"mods/server.jar","hashes":{"sha512":"def"},"downloads":["https://cdn.modrinth.com/data/s.jar"],"env":{"client":"unsupported","server":"required"}}],
      "dependencies":{"minecraft":"1.20.1","fabric-loader":"0.15.0"}}"#;

    #[test]
    fn parses_index_and_target() {
        let i = parse_index(INDEX.as_bytes()).unwrap();
        assert_eq!(i.files.len(), 2);
        assert_eq!(i.files[1].env.as_ref().unwrap().client, "unsupported");
        let (mc, l) = target(&i).unwrap();
        assert_eq!(mc, "1.20.1");
        assert_eq!(l, Loader::Fabric);
    }

    #[test]
    fn loader_detection_and_missing_minecraft() {
        let mut i = parse_index(INDEX.as_bytes()).unwrap();
        i.dependencies.remove("fabric-loader");
        i.dependencies.insert("neoforge".into(), "21".into());
        assert_eq!(target(&i).unwrap().1, Loader::Neoforge);
        i.dependencies.remove("minecraft");
        assert!(target(&i).is_err());
    }

    #[test]
    fn rejects_unsafe_paths() {
        assert_eq!(safe_rel_path("mods/a.jar").unwrap(), PathBuf::from("mods").join("a.jar"));
        for bad in ["../x", "mods/../../x", "/etc/passwd", "C:/x", "a\\b", "", "mods//a.jar", "./a"] {
            assert!(safe_rel_path(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn download_hosts() {
        assert!(trusted_pack_url("https://cdn.modrinth.com/data/x"));
        assert!(trusted_pack_url("https://raw.githubusercontent.com/a/b"));
        assert!(!trusted_pack_url("http://cdn.modrinth.com/x"));
        assert!(!trusted_pack_url("https://evil.com/https://cdn.modrinth.com/"));
    }
}
