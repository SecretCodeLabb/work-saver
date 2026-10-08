// Utilidades mínimas para construir la interfaz sin framework.

import { icon, IconName } from "./icons";

export type Child = Node | string | number | null | undefined | false;
type Props = Record<string, unknown>;

/** Crea un elemento: h("div", { class: "card", onclick: fn }, hijos...). */
export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Props | null = null,
  ...children: (Child | Child[])[]
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  if (props) {
    for (const [key, value] of Object.entries(props)) {
      if (value === undefined || value === null || value === false) continue;
      if (key === "class") {
        el.className = String(value);
      } else if (key.startsWith("on") && typeof value === "function") {
        el.addEventListener(key.slice(2).toLowerCase(), value as EventListener);
      } else if (key in el) {
        (el as unknown as Record<string, unknown>)[key] = value;
      } else {
        el.setAttribute(key, value === true ? "" : String(value));
      }
    }
  }
  append(el, children);
  return el;
}

export function append(el: Element, children: (Child | Child[])[]) {
  for (const child of children.flat()) {
    if (child === null || child === undefined || child === false) continue;
    el.append(child instanceof Node ? child : String(child));
  }
}

export function button(
  label: Child,
  onClick: (e: MouseEvent) => void,
  opts: { variant?: "primary" | "ghost" | "danger"; small?: boolean; icon?: IconName; title?: string } = {},
): HTMLButtonElement {
  const classes = ["btn"];
  if (opts.variant) classes.push(`btn-${opts.variant}`);
  if (opts.small) classes.push("btn-sm");
  if (!label) classes.push("btn-icon");
  return h(
    "button",
    { class: classes.join(" "), type: "button", title: opts.title, onclick: onClick },
    opts.icon ? icon(opts.icon, opts.small ? 14 : 16) : null,
    label,
  );
}

export type Switch = HTMLLabelElement & { input: HTMLInputElement };

export function switchEl(checked: boolean, onChange: (value: boolean) => void, large = false): Switch {
  const input = h("input", { type: "checkbox", checked });
  input.addEventListener("change", () => onChange(input.checked));
  const label = h("label", { class: large ? "switch lg" : "switch" }, input, h("span")) as Switch;
  label.input = input;
  return label;
}

/** Fila de ajuste: etiqueta + descripción a la izquierda, control a la derecha. */
export function field(label: string, hint: string | null, ...controls: Child[]): HTMLDivElement {
  return h(
    "div",
    { class: "field" },
    h("div", { class: "field-label" }, h("span", null, label), hint ? h("span", null, hint) : null),
    h("div", { class: "field-control" }, ...controls),
  );
}

export function numberInput(value: number, min: number, max: number, onChange: (value: number) => void) {
  const input = h("input", { class: "input", type: "number", min, max, value: String(value) });
  input.addEventListener("change", () => {
    const parsed = Math.round(Number(input.value));
    const clamped = Number.isFinite(parsed) ? Math.min(max, Math.max(min, parsed)) : value;
    input.value = String(clamped);
    onChange(clamped);
  });
  return input;
}

export function card(title: Child, ...children: Child[]): HTMLElement {
  return h("section", { class: "card" }, title ? h("h3", { class: "card-title" }, title) : null, ...children);
}

export function viewHeader(title: string, subtitle?: string, ...actions: Child[]): HTMLElement {
  return h(
    "header",
    { class: "view-header" },
    h("div", null, h("h1", null, title), subtitle ? h("p", null, subtitle) : null),
    actions.length ? h("div", { class: "row" }, ...actions) : null,
  );
}

export function toast(message: string, kind: "info" | "error" = "info") {
  const root = document.getElementById("toast-root");
  if (!root) return;
  const el = h("div", { class: `toast ${kind}` }, message);
  root.append(el);
  setTimeout(() => el.remove(), 3200);
}

export interface ModalHandle {
  close: () => void;
  body: HTMLElement;
  footer: HTMLElement;
}

export function modal(title: string, onClose?: () => void): ModalHandle {
  const body = h("div", { class: "modal-body" });
  const footer = h("div", { class: "modal-footer" });
  const backdrop = h("div", { class: "modal-backdrop" });
  const close = () => {
    backdrop.remove();
    document.removeEventListener("keydown", onKey);
    onClose?.();
  };
  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape") close();
  };
  document.addEventListener("keydown", onKey);
  backdrop.addEventListener("mousedown", (e) => {
    if (e.target === backdrop) close();
  });
  backdrop.append(
    h(
      "div",
      { class: "modal", role: "dialog" },
      h("div", { class: "modal-header" }, h("h3", null, title), button(null, close, { variant: "ghost", icon: "x" })),
      body,
      footer,
    ),
  );
  document.body.append(backdrop);
  return { close, body, footer };
}

/** Pide confirmación con un modal. */
export function confirmDialog(title: string, message: string, confirmLabel: string, cancelLabel: string): Promise<boolean> {
  return new Promise((resolve) => {
    let answered = false;
    const m = modal(title, () => {
      if (!answered) resolve(false);
    });
    m.body.append(h("p", { class: "muted", style: "margin:0" }, message));
    m.footer.append(
      button(cancelLabel, () => m.close(), { variant: "ghost" }),
      button(confirmLabel, () => {
        answered = true;
        m.close();
        resolve(true);
      }, { variant: "primary" }),
    );
  });
}

/** Iniciales para el icono de un programa ("blender.exe" → "BL"). */
export function initials(name: string): string {
  return name.replace(/\.exe$/i, "").replace(/[^a-z0-9]/gi, "").slice(0, 2).toUpperCase() || "?";
}
