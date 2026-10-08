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
  untitled_markers: string[];
  dirty_markers: string[];
  extensions: string[];
  watch_folders: string[];
}

export interface BackupConfig {
  enabled: boolean;
  folder: string;
  keep_per_file: number;
  min_minutes_between: number;
  max_total_mb: number;
}

export interface SmartSave {
  wait_for_idle: boolean;
  idle_seconds: number;
  max_wait_seconds: number;
  skip_untitled: boolean;
  skip_dialogs: boolean;
  only_when_dirty: boolean;
}

export interface Notifications {
  on_save: boolean;
  on_problem: boolean;
  warn_before: boolean;
  warn_seconds: number;
  sound: boolean;
}

export interface Config {
  enabled: boolean;
  paused_until: number | null;
  notifications: Notifications;
  toggle_hotkey: string;
  autostart: boolean;
  start_minimized: boolean;
  profiles: Profile[];
  smart: SmartSave;
  backups: BackupConfig;
}

export interface Preset {
  name: string;
  exe: string;
  shortcut: string;
  extensions: string[];
}

export interface OpenApp {
  exe: string;
  title: string;
}

export interface SaveInfo {
  time: string;
  app: string;
  profile_id: string;
  verified: boolean | null;
  file: string | null;
}

export interface SaveCheck {
  profile_id: string;
  app: string;
  file: string | null;
}

export interface BackupEntry {
  id: string;
  original: string;
  path: string;
  created_ms: number;
  size: number;
  app: string;
}

export interface BackupGroup {
  original: string;
  name: string;
  app: string;
  total_size: number;
  entries: BackupEntry[];
}

export type Reason = "busy" | "input_held" | "dialog" | "untitled" | "elevated" | "no_changes" | "countdown";

export interface ProfileStatus {
  id: string;
  next_save_in: number;
  reason: Reason | null;
}

export interface CrashAlert {
  profile_id: string;
  app: string;
  time: string;
  code: string;
  last_save: SaveInfo | null;
  last_backup: BackupEntry | null;
}

export interface Status {
  enabled: boolean;
  foreground: { exe: string; title: string } | null;
  active_profile: string | null;
  profiles: ProfileStatus[];
  last_save: SaveInfo | null;
  crash_alert: CrashAlert | null;
}

export const api = {
  getConfig: () => invoke<Config>("get_config"),
  setConfig: (config: Config) => invoke<Config>("set_config", { config }),
  getPresets: () => invoke<Preset[]>("get_presets"),
  listOpenApps: () => invoke<OpenApp[]>("list_open_apps"),
  dismissCrashAlert: () => invoke<void>("dismiss_crash_alert"),
  pause: (minutes: number) => invoke<void>("pause", { minutes }),
  resume: () => invoke<void>("resume"),
  listBackups: () => invoke<BackupGroup[]>("list_backups"),
  restoreBackup: (id: string) => invoke<string>("restore_backup", { id }),
  revealBackup: (id: string) => invoke<void>("reveal_backup", { id }),
  deleteBackup: (id: string) => invoke<void>("delete_backup", { id }),
  deleteBackupGroup: (original: string) => invoke<void>("delete_backup_group", { original }),
  backupRoot: () => invoke<string>("backup_root"),
  openBackupRoot: () => invoke<void>("open_backup_root"),
};

export const onConfigChanged = (cb: () => void) => listen("config_changed", () => cb());
export const onSaved = (cb: (info: SaveInfo) => void) => listen<SaveInfo>("saved", (e) => cb(e.payload));
export const onSaveVerified = (cb: (check: SaveCheck) => void) => listen<SaveCheck>("save_verified", (e) => cb(e.payload));
export const onSaveUnverified = (cb: (check: SaveCheck) => void) =>
  listen<SaveCheck>("save_unverified", (e) => cb(e.payload));
export const onBackupCreated = (cb: (entry: BackupEntry) => void) =>
  listen<BackupEntry>("backup_created", (e) => cb(e.payload));

export function onStatus(callback: (status: Status) => void) {
  return listen<Status>("status", (event) => callback(event.payload));
}

/** Perfil nuevo; los campos omitidos los completa el backend con sus valores por defecto. */
export function newProfile(name: string, exe: string, shortcut = "Ctrl+S", extensions: string[] = []): Profile {
  return {
    id: crypto.randomUUID(),
    name,
    exe,
    enabled: true,
    interval_minutes: 5,
    shortcut,
    extensions,
    watch_folders: [],
  } as unknown as Profile;
}
