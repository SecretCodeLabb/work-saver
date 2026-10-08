// Textos de la interfaz en varios idiomas.

import { en } from "./en";
import { es } from "./es";

export type Key = keyof typeof es;
export type Dict = Record<Key, string>;
export type Language = "es" | "en";

const DICTS: Record<Language, Dict> = { es, en };

let lang: Language = "es";
let dict: Dict = es;

/** "auto" usa el idioma del sistema. */
export function setLanguage(setting: string) {
  const resolved = setting === "auto" || !setting ? (navigator.language.toLowerCase().startsWith("es") ? "es" : "en") : setting;
  lang = resolved === "en" ? "en" : "es";
  dict = DICTS[lang];
  document.documentElement.lang = lang;
}

export function t(key: Key, vars?: Record<string, string | number>): string {
  let text = dict[key] ?? key;
  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.split(`{${name}}`).join(String(value));
    }
  }
  return text;
}

/** 75 → "1:15", 3725 → "1:02:05". */
export function formatCountdown(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = String(s % 60).padStart(2, "0");
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}` : `${minutes}:${seconds}`;
}

/** 1536 → "1,5 KB". */
export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toLocaleString(lang, { maximumFractionDigits: unit ? 1 : 0 })} ${units[unit]}`;
}

export function formatDateTime(ms: number): string {
  return new Date(ms).toLocaleString(lang, { dateStyle: "medium", timeStyle: "short" });
}

/** 4500 → "1 h 15 min", 90 → "1 min", 20 → "20 s". */
export function formatDuration(totalSeconds: number): string {
  const s = Math.floor(totalSeconds);
  if (s < 60) return `${s} s`;
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  return hours ? `${hours} h ${minutes} min` : `${minutes} min`;
}
