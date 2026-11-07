import { invoke } from "@tauri-apps/api/core";

export type Loader = "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
export type Kind = "mod" | "resourcepack" | "shader";
export interface Instance { id: string; name: string; mc_version: string; loader: Loader; loader_version: string | null }
export interface Hit { project_id: string; title: string; description: string; icon_url: string | null; downloads: number; author: string }
export interface InstalledMod { project_id: string; version_id: string; title: string; version_number: string; kind: Kind; filename: string; sha512: string }
export interface UpdateInfo { project_id: string; title: string; current: string; latest: string }

export const api = {
  list: () => invoke<Instance[]>("list_instances"),
  create: (name: string, mcVersion: string, loader: Loader) => invoke<Instance>("create_instance", { name, mcVersion, loader }),
  remove: (id: string) => invoke<void>("delete_instance", { id }),
  search: (instanceId: string, kind: Kind, query: string) => invoke<Hit[]>("search_mods", { instanceId, kind, query }),
  install: (instanceId: string, projectId: string, kind: Kind) => invoke<InstalledMod[]>("install_project", { instanceId, projectId, kind }),
  installed: (instanceId: string) => invoke<InstalledMod[]>("list_installed", { instanceId }),
  uninstall: (instanceId: string, projectId: string) => invoke<void>("remove_installed", { instanceId, projectId }),
  checkUpdates: (instanceId: string) => invoke<UpdateInfo[]>("check_updates", { instanceId }),
  update: (instanceId: string, projectId: string) => invoke<void>("update_project", { instanceId, projectId }),
  openFolder: (id: string) => invoke<void>("open_instance_folder", { id }),
};
