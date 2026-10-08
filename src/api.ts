// Tipos y llamadas al backend (Rust).

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Config {
  enabled: boolean;
  interval_minutes: number;
  processes: string[];
}

export interface SaveInfo {
  time: string;
  app: string;
}

export interface Status {
  enabled: boolean;
  foreground: string | null;
  next_save_in: number | null;
  last_save: SaveInfo | null;
}

export const api = {
  getConfig: () => invoke<Config>("get_config"),
  setConfig: (config: Config) => invoke<Config>("set_config", { config }),
};

export function onStatus(callback: (status: Status) => void) {
  return listen<Status>("status", (event) => callback(event.payload));
}
