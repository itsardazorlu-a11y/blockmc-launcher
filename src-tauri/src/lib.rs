use blockmc_core::{SearchPage, InstalledMod, InstanceConfig, InstanceStore, Loader, Modrinth, ProjectKind, UpdateInfo};
use std::{path::{Path, PathBuf}, process::Command};
use tauri::State;

fn data_root() -> Result<PathBuf, String> {
    dirs::data_dir().map(|d| d.join("BlockMC")).ok_or_else(|| "Could not find your user data folder.".to_string())
}
fn store() -> Result<InstanceStore, String> { Ok(InstanceStore::new(data_root()?)) }
fn load(id: &str) -> Result<(InstanceStore, InstanceConfig), String> {
    let s = store()?;
    let inst = s.get(id).map_err(|e| e.to_string())?;
    Ok((s, inst))
}

#[tauri::command]
fn list_instances() -> Result<Vec<InstanceConfig>, String> { store()?.list().map_err(|e| e.to_string()) }

#[tauri::command]
fn create_instance(name: String, mc_version: String, loader: Loader) -> Result<InstanceConfig, String> {
    store()?.create(&name, &mc_version, loader).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_instance(id: String, name: String) -> Result<InstanceConfig, String> {
    store()?.rename(&id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn duplicate_instance(id: String) -> Result<InstanceConfig, String> {
    store()?.duplicate(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_instance(id: String) -> Result<(), String> { store()?.delete(&id).map_err(|e| e.to_string()) }

#[tauri::command]
async fn search_mods(mr: State<'_, Modrinth>, instance_id: String, kind: ProjectKind, query: String, sort: String, offset: u32, limit: u32) -> Result<SearchPage, String> {
    let (_, inst) = load(&instance_id)?;
    mr.search(&inst, kind, &query, &sort, offset, limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_modpacks(mr: State<'_, Modrinth>, query: String, sort: String, offset: u32, limit: u32, mc_version: Option<String>, loader: Option<String>) -> Result<SearchPage, String> {
    mr.search_modpacks(&query, &sort, offset, limit, mc_version.as_deref(), loader.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn game_versions(mr: State<'_, Modrinth>) -> Result<Vec<String>, String> {
    mr.game_versions().await.map_err(|e| e.to_string())
}

#[tauri::command]
fn data_folder() -> Result<String, String> { Ok(data_root()?.to_string_lossy().into_owned()) }

#[tauri::command]
fn open_data_folder() -> Result<(), String> {
    let dir = data_root()?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    open_path(&dir)
}

#[tauri::command]
async fn install_modpack(mr: State<'_, Modrinth>, project_id: String) -> Result<InstanceConfig, String> {
    mr.install_modpack(&store()?, &project_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_project(mr: State<'_, Modrinth>, instance_id: String, project_id: String, kind: ProjectKind) -> Result<Vec<InstalledMod>, String> {
    let (s, inst) = load(&instance_id)?;
    mr.install(&s, &inst, &project_id, kind).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn repair_dependencies(mr: State<'_, Modrinth>, instance_id: String) -> Result<Vec<InstalledMod>, String> {
    let (s, inst) = load(&instance_id)?;
    mr.repair_dependencies(&s, &inst).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn list_installed(instance_id: String) -> Result<Vec<InstalledMod>, String> {
    blockmc_core::mods::list_installed(&store()?, &instance_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn dependents(instance_id: String, project_id: String) -> Result<Vec<String>, String> {
    blockmc_core::mods::dependents(&store()?, &instance_id, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_installed(instance_id: String, project_id: String) -> Result<(), String> {
    blockmc_core::mods::remove_installed(&store()?, &instance_id, &project_id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_updates(mr: State<'_, Modrinth>, instance_id: String) -> Result<Vec<UpdateInfo>, String> {
    let (s, inst) = load(&instance_id)?;
    mr.check_updates(&s, &inst).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_project(mr: State<'_, Modrinth>, instance_id: String, project_id: String) -> Result<(), String> {
    let (s, inst) = load(&instance_id)?;
    mr.update(&s, &inst, &project_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn open_instance_folder(id: String) -> Result<(), String> {
    let dir = store()?.instance_dir(&id).map_err(|e| e.to_string())?.join("minecraft");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    open_path(&dir)
}

fn open_path(p: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")] let prog = "explorer";
    #[cfg(target_os = "macos")] let prog = "open";
    #[cfg(all(unix, not(target_os = "macos")))] let prog = "xdg-open";
    Command::new(prog).arg(p).spawn().map(|_| ()).map_err(|e| format!("Could not open the folder: {e}"))
}

pub fn run() {
    tauri::Builder::default()
        .manage(Modrinth::new())
        .invoke_handler(tauri::generate_handler![
            list_instances, create_instance, rename_instance, duplicate_instance, delete_instance, search_mods, install_project,
            search_modpacks, game_versions, data_folder, open_data_folder, install_modpack, repair_dependencies, list_installed, remove_installed, dependents, check_updates, update_project, open_instance_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running BlockMC");
}
