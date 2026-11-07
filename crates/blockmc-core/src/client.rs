//! Modrinth API client and the install / update workflows built on mods.rs.
use crate::{error::CoreError, instance::{InstanceConfig, InstanceStore}, mods::*};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::HashSet, fs, io::Write, path::Path};

const API: &str = "https://api.modrinth.com/v2";
const UA: &str = concat!("blockmc/", env!("CARGO_PKG_VERSION"), " (github.com/itsardazorlu-a11y/BlockMc)");
const MAX_PROJECTS_PER_INSTALL: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    pub project_id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub description: String,
    #[serde(default)] pub icon_url: Option<String>,
    #[serde(default)] pub downloads: u64,
    #[serde(default)] pub author: String,
}
#[derive(Deserialize)]
struct SearchResponse { hits: Vec<Hit> }
#[derive(Deserialize)]
struct Project { title: String }

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo { pub project_id: String, pub title: String, pub current: String, pub latest: String }

pub struct Modrinth { http: reqwest::Client }

impl Default for Modrinth { fn default() -> Self { Self::new() } }

impl Modrinth {
    pub fn new() -> Self {
        Self { http: reqwest::Client::builder().user_agent(UA).build().expect("http client") }
    }

    pub async fn search(&self, inst: &InstanceConfig, kind: ProjectKind, query: &str) -> Result<Vec<Hit>, CoreError> {
        let mut facets: Vec<Vec<String>> = vec![
            vec![format!("project_type:{}", kind.api_name())],
            vec![format!("versions:{}", inst.mc_version)],
        ];
        if kind == ProjectKind::Mod {
            facets.push(compatible_loaders(inst.loader)?.iter().map(|l| format!("categories:{l}")).collect());
        }
        let facets = serde_json::to_string(&facets)?;
        let resp: SearchResponse = self.http.get(format!("{API}/search"))
            .query(&[("query", query), ("facets", facets.as_str()), ("limit", "20")])
            .send().await?.error_for_status()?.json().await?;
        Ok(resp.hits)
    }

    async fn project_title(&self, project_id: &str) -> Result<String, CoreError> {
        let p: Project = self.http.get(format!("{API}/project/{project_id}")).send().await?.error_for_status()?.json().await?;
        Ok(p.title)
    }

    async fn versions(&self, project_id: &str, inst: &InstanceConfig, kind: ProjectKind) -> Result<Vec<Version>, CoreError> {
        if !valid_project_id(project_id) { return Err(CoreError::InvalidProject) }
        let games = serde_json::to_string(&[&inst.mc_version])?;
        let mut req = self.http.get(format!("{API}/project/{project_id}/version")).query(&[("game_versions", games.as_str())]);
        if kind == ProjectKind::Mod {
            let loaders = serde_json::to_string(&compatible_loaders(inst.loader)?)?;
            req = req.query(&[("loaders", loaders.as_str())]);
        }
        Ok(req.send().await?.error_for_status()?.json().await?)
    }

    /// Streams the file to `<name>.part`, hashes while writing, and only keeps it if the SHA-512 matches.
    async fn download(&self, file: &VersionFile, dir: &Path, filename: &str) -> Result<String, CoreError> {
        ensure_trusted_url(&file.url)?;
        fs::create_dir_all(dir)?;
        let part = dir.join(format!("{filename}.part"));
        let mut resp = self.http.get(&file.url).send().await?.error_for_status()?;
        let mut hasher = Sha512::new();
        {
            let mut out = fs::File::create(&part)?;
            while let Some(chunk) = resp.chunk().await? {
                hasher.update(&chunk);
                out.write_all(&chunk)?;
            }
        }
        let actual = format!("{:x}", hasher.finalize());
        finalize_download(&part, &dir.join(filename), &actual, &file.hashes.sha512)?;
        Ok(actual)
    }

    /// Installs a project plus its required dependencies. Everything is resolved before the first
    /// download, so an incompatible dependency fails the whole install instead of leaving half of it.
    pub async fn install(&self, store: &InstanceStore, inst: &InstanceConfig, project_id: &str, kind: ProjectKind) -> Result<Vec<InstalledMod>, CoreError> {
        let idir = store.instance_dir(&inst.id)?;
        let mut manifest = Manifest::load(&idir)?;
        if manifest.items.iter().any(|i| i.project_id == project_id) { return Err(CoreError::AlreadyInstalled) }

        let mut plan: Vec<(String, Version)> = vec![];
        let mut seen: HashSet<String> = HashSet::new();
        let mut queue = vec![project_id.to_string()];
        while let Some(pid) = queue.pop() {
            if !seen.insert(pid.clone()) { continue }
            if manifest.items.iter().any(|i| i.project_id == pid) { continue }
            if seen.len() > MAX_PROJECTS_PER_INSTALL { return Err(CoreError::Unsupported("This install needs too many dependencies.".into())) }
            let versions = self.versions(&pid, inst, kind).await?;
            let v = pick_version(&versions).ok_or_else(|| CoreError::NoCompatibleVersion(pid.clone()))?.clone();
            if kind == ProjectKind::Mod { queue.extend(required_deps(&v)); }
            plan.push((pid, v));
        }

        let dir = content_dir(&idir, kind);
        let mut done = vec![];
        for (pid, v) in plan {
            let file = pick_primary(&v).ok_or_else(|| CoreError::NoCompatibleVersion(pid.clone()))?;
            let filename = safe_filename(&file.filename, kind)?;
            let title = self.project_title(&pid).await.unwrap_or_else(|_| pid.clone());
            let sha512 = self.download(file, &dir, &filename).await?;
            let rec = InstalledMod { project_id: pid, version_id: v.id.clone(), title, version_number: v.version_number.clone(), kind, filename, sha512 };
            manifest.items.push(rec.clone());
            manifest.save(&idir)?;
            done.push(rec);
        }
        Ok(done)
    }

    pub async fn check_updates(&self, store: &InstanceStore, inst: &InstanceConfig) -> Result<Vec<UpdateInfo>, CoreError> {
        let idir = store.instance_dir(&inst.id)?;
        let mut out = vec![];
        for item in Manifest::load(&idir)?.items {
            let Ok(versions) = self.versions(&item.project_id, inst, item.kind).await else { continue };
            if let Some(v) = pick_version(&versions) {
                if v.id != item.version_id {
                    out.push(UpdateInfo { project_id: item.project_id.clone(), title: item.title.clone(), current: item.version_number.clone(), latest: v.version_number.clone() });
                }
            }
        }
        Ok(out)
    }

    pub async fn update(&self, store: &InstanceStore, inst: &InstanceConfig, project_id: &str) -> Result<(), CoreError> {
        let idir = store.instance_dir(&inst.id)?;
        let mut manifest = Manifest::load(&idir)?;
        let pos = manifest.items.iter().position(|i| i.project_id == project_id).ok_or(CoreError::NotFound)?;
        let old = manifest.items[pos].clone();
        let versions = self.versions(project_id, inst, old.kind).await?;
        let v = pick_version(&versions).ok_or_else(|| CoreError::NoCompatibleVersion(old.title.clone()))?.clone();
        if v.id == old.version_id { return Ok(()) }
        let file = pick_primary(&v).ok_or_else(|| CoreError::NoCompatibleVersion(old.title.clone()))?;
        let filename = safe_filename(&file.filename, old.kind)?;
        let dir = content_dir(&idir, old.kind);
        let sha512 = self.download(file, &dir, &filename).await?;
        if filename != old.filename {
            let old_file = dir.join(safe_filename(&old.filename, old.kind)?);
            if old_file.exists() { fs::remove_file(old_file)?; }
        }
        manifest.items[pos] = InstalledMod { version_id: v.id, version_number: v.version_number, filename, sha512, ..old };
        manifest.save(&idir)?;
        Ok(())
    }
}
