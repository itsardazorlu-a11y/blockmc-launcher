import { invoke } from "@tauri-apps/api/core";

export type Loader = "vanilla" | "fabric" | "forge" | "neoforge" | "quilt";
export type Kind = "mod" | "resourcepack" | "shader";
export interface Instance { id: string; name: string; mc_version: string; loader: Loader; loader_version: string | null }
export interface Hit { project_id: string; title: string; description: string; icon_url: string | null; downloads: number; follows: number; author: string; categories: string[] }
export interface SearchPage { hits: Hit[]; total_hits: number }
export interface InstalledMod { project_id: string; version_id: string; title: string; version_number: string; kind: Kind; filename: string; sha512: string; requires: string[] }
export interface UpdateInfo { project_id: string; title: string; current: string; latest: string }
export type Fetcher = (query: string, sort: string, offset: number, limit: number) => Promise<SearchPage>;

export const api = {
  list: () => invoke<Instance[]>("list_instances"),
  create: (name: string, mcVersion: string, loader: Loader) => invoke<Instance>("create_instance", { name, mcVersion, loader }),
  rename: (id: string, name: string) => invoke<Instance>("rename_instance", { id, name }),
  duplicate: (id: string) => invoke<Instance>("duplicate_instance", { id }),
  remove: (id: string) => invoke<void>("delete_instance", { id }),
  gameVersions: () => invoke<string[]>("game_versions"),
  search: (instanceId: string, kind: Kind, query: string, sort: string, offset: number, limit: number) =>
    invoke<SearchPage>("search_mods", { instanceId, kind, query, sort, offset, limit }),
  searchModpacks: (query: string, sort: string, offset: number, limit: number, mcVersion: string | null, loader: string | null) =>
    invoke<SearchPage>("search_modpacks", { query, sort, offset, limit, mcVersion, loader }),
  installModpack: (projectId: string) => invoke<Instance>("install_modpack", { projectId }),
  install: (instanceId: string, projectId: string, kind: Kind) => invoke<InstalledMod[]>("install_project", { instanceId, projectId, kind }),
  installed: (instanceId: string) => invoke<InstalledMod[]>("list_installed", { instanceId }),
  dependents: (instanceId: string, projectId: string) => invoke<string[]>("dependents", { instanceId, projectId }),
  uninstall: (instanceId: string, projectId: string) => invoke<void>("remove_installed", { instanceId, projectId }),
  checkUpdates: (instanceId: string) => invoke<UpdateInfo[]>("check_updates", { instanceId }),
  update: (instanceId: string, projectId: string) => invoke<void>("update_project", { instanceId, projectId }),
  openFolder: (id: string) => invoke<void>("open_instance_folder", { id }),
  dataFolder: () => invoke<string>("data_folder"),
  openDataFolder: () => invoke<void>("open_data_folder"),
};

export const LOADERS: { id: Loader; label: string }[] = [
  { id: "vanilla", label: "Vanilla" }, { id: "fabric", label: "Fabric" }, { id: "forge", label: "Forge" },
  { id: "neoforge", label: "NeoForge" }, { id: "quilt", label: "Quilt" },
];
export const loaderLabel = (l: Loader) => LOADERS.find((x) => x.id === l)!.label;
