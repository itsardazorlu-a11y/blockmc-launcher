//! Modrinth API client and the install / update workflows built on mods.rs.
use crate::{error::CoreError, instance::{InstanceConfig, InstanceStore}, modpack, mods::*};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::{collections::HashSet, fs, io::Write, path::Path};

const API: &str = "https://api.modrinth.com/v2";
const UA: &str = concat!("blockmc/", env!("CARGO_PKG_VERSION"), " (github.com/itsardazorlu-a11y/BlockMc)");
const SORTS: [&str; 5] = ["relevance", "downloads", "follows", "newest", "updated"];
const MAX_PROJECTS_PER_INSTALL: usize = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit {
    pub project_id: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub description: String,
    #[serde(default)] pub icon_url: Option<String>,
    #[serde(default)] pub downloads: u64,
    #[serde(default)] pub author: String,
    #[serde(default)] pub follows: u64,
    #[serde(default)] pub categories: Vec<String>,
}
#[derive(Deserialize)]
struct SearchResponse { hits: Vec<Hit>, #[serde(default)] total_hits: u64 }

#[derive(Debug, Clone, Serialize)]
pub struct SearchPage { pub hits: Vec<Hit>, pub total_hits: u64 }
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

    async fn search_raw(&self, query: &str, facets: Vec<Vec<String>>, sort: &str, offset: u32, limit: u32) -> Result<SearchPage, CoreError> {
        let sort = if SORTS.contains(&sort) { sort } else { "relevance" };
        let limit = limit.clamp(1, 50).to_string();
        let offset = offset.to_string();
        let facets = serde_json::to_string(&facets)?;
        let resp: SearchResponse = self.http.get(format!("{API}/search"))
            .query(&[("query", query), ("facets", facets.as_str()), ("limit", limit.as_str()), ("index", sort), ("offset", offset.as_str())])
            .send().await?.error_for_status()?.json().await?;
        Ok(SearchPage { hits: resp.hits, total_hits: resp.total_hits })
    }

    pub async fn search(&self, inst: &InstanceConfig, kind: ProjectKind, query: &str, sort: &str, offset: u32, limit: u32) -> Result<SearchPage, CoreError> {
        let mut facets: Vec<Vec<String>> = vec![
            vec![format!("project_type:{}", kind.api_name())],
            vec![format!("versions:{}", inst.mc_version)],
        ];
        if kind == ProjectKind::Mod {
            facets.push(compatible_loaders(inst.loader)?.iter().map(|l| format!("categories:{l}")).collect());
        }
        self.search_raw(query, facets, sort, offset, limit).await
    }

    pub async fn search_modpacks(&self, query: &str, sort: &str, offset: u32, limit: u32, mc_version: Option<&str>, loader: Option<&str>) -> Result<SearchPage, CoreError> {
        let mut facets: Vec<Vec<String>> = vec![vec!["project_type:modpack".to_string()]];
        if let Some(v) = mc_version.filter(|v| !v.is_empty()) {
            if !valid_mc_version(v) { return Err(CoreError::Unsupported("Invalid Minecraft version.".into())) }
            facets.push(vec![format!("versions:{v}")]);
        }
        if let Some(l) = loader.filter(|l| !l.is_empty()) {
            if !["fabric", "forge", "neoforge", "quilt"].contains(&l) { return Err(CoreError::Unsupported("Unknown mod loader.".into())) }
            facets.push(vec![format!("categories:{l}")]);
        }
        self.search_raw(query, facets, sort, offset, limit).await
    }

    /// Release versions Modrinth knows about, newest first, down to 1.8.
    pub async fn game_versions(&self) -> Result<Vec<String>, CoreError> {
        let tags: Vec<GameVersion> = self.http.get(format!("{API}/tag/game_version")).send().await?.error_for_status()?.json().await?;
        Ok(release_versions(tags))
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

        let loaders: Vec<&str> = if kind == ProjectKind::Mod { compatible_loaders(inst.loader)? } else { vec![] };
        let mut plan: Vec<(String, Version, Vec<String>)> = vec![];
        let mut seen: HashSet<String> = HashSet::new();
        let mut queue: Vec<(String, Option<Version>)> = vec![(project_id.to_string(), None)];
        while let Some((pid, pinned)) = queue.pop() {
            if !seen.insert(pid.clone()) { continue }
            if manifest.items.iter().any(|i| i.project_id == pid) { continue }
            if seen.len() > MAX_PROJECTS_PER_INSTALL { return Err(CoreError::Unsupported("This install needs too many dependencies.".into())) }
            // A dependency may name an exact version; use it when it fits this instance, otherwise the newest that does.
            let v = match pinned.filter(|p| is_compatible(p, &inst.mc_version, &loaders)) {
                Some(p) => p,
                None => {
                    let versions = self.versions(&pid, inst, kind).await?;
                    match pick_version(&versions) {
                        Some(v) => v.clone(),
                        None => {
                            let title = self.project_title(&pid).await.unwrap_or_else(|_| pid.clone());
                            return Err(CoreError::NoCompatibleVersion(title));
                        }
                    }
                }
            };
            for d in v.dependencies.iter().filter(|d| d.dependency_type == "incompatible") {
                if let Some(other) = d.project_id.as_ref().and_then(|p| manifest.items.iter().find(|i| &i.project_id == p)) {
                    return Err(CoreError::Unsupported(format!("This mod is not compatible with {}, which is already installed.", other.title)));
                }
            }
            let mut requires = vec![];
            if kind == ProjectKind::Mod {
                for (dep_pid, dep_pinned) in self.required_refs(&v).await? {
                    requires.push(dep_pid.clone());
                    queue.push((dep_pid, dep_pinned));
                }
            }
            plan.push((pid, v, requires));
        }

        let dir = content_dir(&idir, kind);
        let mut done = vec![];
        for (pid, v, requires) in plan {
            let file = pick_primary(&v).ok_or_else(|| CoreError::NoCompatibleVersion(pid.clone()))?;
            ensure_trusted_url(&file.url)?;
            let filename = safe_filename(&file.filename, kind)?;
            let title = self.project_title(&pid).await.unwrap_or_else(|_| pid.clone());
            let sha512 = self.download(file, &dir, &filename).await?;
            let rec = InstalledMod { project_id: pid, version_id: v.id.clone(), title, version_number: v.version_number.clone(), kind, filename, sha512, requires };
            manifest.items.push(rec.clone());
            manifest.save(&idir)?;
            done.push(rec);
        }
        Ok(done)
    }

    /// Installs a Modrinth modpack (.mrpack) into a brand-new instance. If anything fails the new
    /// instance is deleted again, so a broken download never leaves a half-installed pack behind.
    pub async fn install_modpack(&self, store: &InstanceStore, project_id: &str) -> Result<InstanceConfig, CoreError> {
        if !valid_project_id(project_id) { return Err(CoreError::InvalidProject) }
        let title = self.project_title(project_id).await?;
        let versions: Vec<Version> = self.http.get(format!("{API}/project/{project_id}/version"))
            .send().await?.error_for_status()?.json().await?;
        let v = pick_version(&versions).ok_or_else(|| CoreError::NoCompatibleVersion(title.clone()))?.clone();
        let file = pick_primary(&v).ok_or_else(|| CoreError::NoCompatibleVersion(title.clone()))?;
        ensure_trusted_url(&file.url)?;
        let filename = safe_name_with_ext(&file.filename, ".mrpack")?;
        let tmp = std::env::temp_dir().join(format!("blockmc-{}", v.id));
        fs::create_dir_all(&tmp)?;
        let result = self.install_pack_file(store, &title, file, &tmp, &filename).await;
        let _ = fs::remove_dir_all(&tmp);
        result
    }

    async fn install_pack_file(&self, store: &InstanceStore, title: &str, file: &VersionFile, tmp: &Path, filename: &str) -> Result<InstanceConfig, CoreError> {
        self.download(file, tmp, filename).await?;
        let pack = tmp.join(filename);
        let index = modpack::read_index(&pack)?;
        let (mc, loader) = modpack::target(&index)?;
        let inst = store.create(title, &mc, loader)?;
        match self.fill_instance(store, &inst, &index, &pack).await {
            Ok(()) => Ok(inst),
            Err(e) => { let _ = store.delete(&inst.id); Err(e) }
        }
    }

    async fn fill_instance(&self, store: &InstanceStore, inst: &InstanceConfig, index: &modpack::Index, pack: &Path) -> Result<(), CoreError> {
        let mc_dir = store.instance_dir(&inst.id)?.join("minecraft");
        for f in &index.files {
            if f.env.as_ref().map(|e| e.client == "unsupported").unwrap_or(false) { continue }
            let rel = modpack::safe_rel_path(&f.path)?;
            let url = f.downloads.iter().find(|u| modpack::trusted_pack_url(u))
                .ok_or_else(|| CoreError::UntrustedUrl(f.downloads.first().cloned().unwrap_or_default()))?;
            let dest = mc_dir.join(&rel);
            let dir = dest.parent().ok_or_else(|| CoreError::UnsafeFilename(f.path.clone()))?;
            let name = dest.file_name().and_then(|n| n.to_str()).ok_or_else(|| CoreError::UnsafeFilename(f.path.clone()))?;
            let vf = VersionFile { hashes: FileHashes { sha512: f.hashes.sha512.clone() }, url: url.clone(), filename: name.to_string(), primary: true };
            self.download(&vf, dir, name).await?;
        }
        modpack::extract_overrides(pack, &mc_dir)
    }

    async fn version_by_id(&self, id: &str) -> Result<Version, CoreError> {
        if !valid_project_id(id) { return Err(CoreError::InvalidProject) }
        Ok(self.http.get(format!("{API}/version/{id}")).send().await?.error_for_status()?.json().await?)
    }

    /// Required dependencies of a version as (project id, exact version if the author pinned one).
    async fn required_refs(&self, v: &Version) -> Result<Vec<(String, Option<Version>)>, CoreError> {
        let mut out: Vec<(String, Option<Version>)> = vec![];
        for d in v.dependencies.iter().filter(|d| d.dependency_type == "required") {
            match (&d.project_id, &d.version_id) {
                (Some(p), Some(vid)) => out.push((p.clone(), self.version_by_id(vid).await.ok())),
                (Some(p), None) => out.push((p.clone(), None)),
                (None, Some(vid)) => { let pv = self.version_by_id(vid).await?; out.push((pv.project_id.clone(), Some(pv))) }
                (None, None) => {}
            }
        }
        Ok(out)
    }

    /// Looks at every installed mod, finds its required dependencies on Modrinth and installs any that are missing.
    pub async fn repair_dependencies(&self, store: &InstanceStore, inst: &InstanceConfig) -> Result<Vec<InstalledMod>, CoreError> {
        let idir = store.instance_dir(&inst.id)?;
        let items: Vec<InstalledMod> = Manifest::load(&idir)?.items.into_iter().filter(|i| i.kind == ProjectKind::Mod).collect();
        let mut added = vec![];
        for item in items {
            let Ok(v) = self.version_by_id(&item.version_id).await else { continue };
            let Ok(refs) = self.required_refs(&v).await else { continue };
            let ids: Vec<String> = refs.into_iter().map(|(p, _)| p).collect();
            let mut m = Manifest::load(&idir)?;
            if let Some(x) = m.items.iter_mut().find(|x| x.project_id == item.project_id) { x.requires = ids.clone(); }
            m.save(&idir)?;
            for dep in ids {
                match self.install(store, inst, &dep, ProjectKind::Mod).await {
                    Ok(mut r) => added.append(&mut r),
                    Err(CoreError::AlreadyInstalled) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(added)
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
        ensure_trusted_url(&file.url)?;
        let filename = safe_filename(&file.filename, old.kind)?;
        let dir = content_dir(&idir, old.kind);
        let sha512 = self.download(file, &dir, &filename).await?;
        if filename != old.filename {
            let old_file = dir.join(safe_filename(&old.filename, old.kind)?);
            if old_file.exists() { fs::remove_file(old_file)?; }
        }
        let requires: Vec<String> = if old.kind == ProjectKind::Mod {
            self.required_refs(&v).await.unwrap_or_default().into_iter().map(|(p, _)| p).collect()
        } else { vec![] };
        manifest.items[pos] = InstalledMod { version_id: v.id, version_number: v.version_number, filename, sha512, requires: requires.clone(), ..old };
        manifest.save(&idir)?;
        // A newer version may need libraries the old one didn't.
        if old.kind == ProjectKind::Mod {
            for dep in requires {
                match self.install(store, inst, &dep, ProjectKind::Mod).await {
                    Ok(_) | Err(CoreError::AlreadyInstalled) => {}
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }
}
