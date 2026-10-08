// Campo que graba un atajo de teclado al pulsarlo (ej. Ctrl+S).

import { t } from "../i18n";
import { h } from "./dom";

const NAMED: Record<string, string> = {
  Enter: "Enter",
  " ": "Space",
  Tab: "Tab",
  Insert: "Insert",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

function keyName(e: KeyboardEvent): string | null {
  if (/^Key[A-Z]$/.test(e.code)) return e.code.slice(3);
  if (/^Digit\d$/.test(e.code)) return e.code.slice(5);
  if (/^F\d{1,2}$/.test(e.key)) return e.key;
  return NAMED[e.key] ?? null;
}

export function shortcutInput(value: string, onChange: (value: string) => void): HTMLInputElement {
  const input = h("input", {
    class: "input mono",
    readOnly: true,
    value,
    title: t("shortcut.hint"),
    style: "width:10rem;text-align:center;cursor:pointer",
  });
  input.addEventListener("focus", () => (input.value = t("shortcut.press")));
  input.addEventListener("blur", () => (input.value = value));
  input.addEventListener("keydown", (e) => {
    e.preventDefault();
    if (e.key === "Escape") return input.blur();
    const key = keyName(e);
    if (!key) return; // Solo se pulsó un modificador: seguir esperando.
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    if (e.metaKey) parts.push("Win");
    parts.push(key);
    value = parts.join("+");
    onChange(value);
    input.blur();
  });
  return input;
}
