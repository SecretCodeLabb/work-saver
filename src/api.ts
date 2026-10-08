// Tipos y llamadas al backend (Rust).

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Profile {
  id: string;
  name: string;
  exe: string;
  enabled: boolean;
  interval_minutes: number;
  shortcut: string;
}

export interface Config {
  enabled: boolean;
  profiles: Profile[];
}

export interface Preset {
  name: string;
  exe: string;
  shortcut: string;
}

export interface OpenApp {
  exe: string;
  title: string;
}

export interface SaveInfo {
  time: string;
  app: string;
  profile_id: string;
}

export interface ProfileStatus {
  id: string;
  next_save_in: number;
}

export interface Status {
  enabled: boolean;
  foreground: { exe: string; title: string } | null;
  active_profile: string | null;
  profiles: ProfileStatus[];
  last_save: SaveInfo | null;
}

export const api = {
  getConfig: () => invoke<Config>("get_config"),
  setConfig: (config: Config) => invoke<Config>("set_config", { config }),
  getPresets: () => invoke<Preset[]>("get_presets"),
  listOpenApps: () => invoke<OpenApp[]>("list_open_apps"),
};

export function onStatus(callback: (status: Status) => void) {
  return listen<Status>("status", (event) => callback(event.payload));
}

/** Perfil nuevo con valores por defecto (el backend completa el resto). */
export function newProfile(name: string, exe: string, shortcut = "Ctrl+S"): Profile {
  return {
    id: crypto.randomUUID(),
    name,
    exe,
    enabled: true,
    interval_minutes: 5,
    shortcut,
  };
}
