// Modal para añadir un programa: ventanas abiertas, lista de populares o manual.

import { api, newProfile, OpenApp, Preset } from "../api";
import { t } from "../i18n";
import { store } from "../store";
import { button, h, initials, modal, toast } from "../ui/dom";
import { icon } from "../ui/icons";

type Tab = "open" | "presets" | "manual";

let presetsCache: Preset[] | null = null;

export async function presets(): Promise<Preset[]> {
  presetsCache ??= await api.getPresets();
  return presetsCache;
}

function isAdded(exe: string) {
  return store.config.profiles.some((p) => p.exe === exe.toLowerCase());
}

export function openAddProgram() {
  const m = modal(t("add.title"));
  const content = h("div");
  const tabs: Record<Tab, HTMLButtonElement> = {
    open: h("button", { class: "tab", type: "button" }, t("add.tab.open")),
    presets: h("button", { class: "tab", type: "button" }, t("add.tab.presets")),
    manual: h("button", { class: "tab", type: "button" }, t("add.tab.manual")),
  };
  for (const [name, tab] of Object.entries(tabs) as [Tab, HTMLButtonElement][]) {
    tab.addEventListener("click", () => show(name));
  }
  m.body.append(h("div", { class: "tabs" }, Object.values(tabs)), content);
  m.footer.remove();

  function add(preset: Pick<Preset, "name" | "exe"> & Partial<Preset>) {
    store.update((c) => c.profiles.push(newProfile(preset.name, preset.exe, preset.shortcut)));
    toast(t("add.added", { name: preset.name || preset.exe }));
    m.close();
  }

  function pickItem(title: string, subtitle: string, exe: string, onPick: () => void) {
    const added = isAdded(exe);
    return h(
      "button",
      { class: "pick-item", type: "button", disabled: added, onclick: onPick },
      h("div", { class: "app-icon" }, initials(title)),
      h(
        "div",
        { class: "list-item-main" },
        h("div", { class: "list-item-title truncate" }, title),
        h("div", { class: "list-item-sub truncate" }, subtitle),
      ),
      added ? h("span", { class: "pill on" }, t("add.already")) : icon("plus", 16),
    );
  }

  async function showOpen() {
    content.replaceChildren(h("div", { class: "empty" }, t("common.loading")));
    const [apps, known] = await Promise.all([api.listOpenApps(), presets()]);
    const presetFor = (app: OpenApp): Preset =>
      known.find((p) => p.exe === app.exe) ?? {
        name: app.exe.replace(/\.exe$/, "").replace(/^./, (c) => c.toUpperCase()),
        exe: app.exe,
        shortcut: "Ctrl+S",
      };
    content.replaceChildren(
      h("p", { class: "small muted", style: "margin:0 0 .75rem" }, t("add.open.hint")),
      h(
        "div",
        { class: "stack", style: "gap:2px" },
        apps.length
          ? apps.map((app) => {
              const preset = presetFor(app);
              return pickItem(preset.name, `${app.exe} · ${app.title}`, app.exe, () => add(preset));
            })
          : h("div", { class: "empty" }, t("add.open.empty")),
      ),
      h("div", { class: "row", style: "margin-top:.75rem" }, button(t("add.refresh"), () => showOpen(), { small: true, icon: "restore" })),
    );
  }

  async function showPresets() {
    const all = await presets();
    const search = h("input", { class: "input", placeholder: t("add.search") });
    const list = h("div", { class: "stack", style: "gap:2px;margin-top:.75rem" });
    const render = () => {
      const q = search.value.trim().toLowerCase();
      const matches = all.filter((p) => p.name.toLowerCase().includes(q) || p.exe.includes(q));
      list.replaceChildren(
        ...(matches.length
          ? matches.map((p) => pickItem(p.name, p.exe, p.exe, () => add(p)))
          : [h("div", { class: "empty" }, t("add.presets.empty"))]),
      );
    };
    search.addEventListener("input", render);
    content.replaceChildren(search, list);
    render();
    search.focus();
  }

  function showManual() {
    const name = h("input", { class: "input", placeholder: "Krita" });
    const exe = h("input", { class: "input", placeholder: "krita.exe" });
    const submit = () => {
      let exeValue = exe.value.trim().toLowerCase();
      if (!exeValue) return exe.focus();
      if (!exeValue.endsWith(".exe")) exeValue += ".exe";
      if (isAdded(exeValue)) return toast(t("add.duplicate"), "error");
      add({ name: name.value.trim(), exe: exeValue });
    };
    exe.addEventListener("keydown", (e) => {
      if (e.key === "Enter") submit();
    });
    content.replaceChildren(
      h(
        "div",
        { class: "stack" },
        h("label", { class: "stack", style: "gap:.35rem" }, h("span", null, t("add.manual.name")), name),
        h("label", { class: "stack", style: "gap:.35rem" }, h("span", null, t("add.manual.exe")), exe),
        h("p", { class: "small muted", style: "margin:0" }, t("add.manual.hint")),
        h("div", { class: "row" }, h("div", { class: "spacer" }), button(t("add.submit"), submit, { variant: "primary", icon: "plus" })),
      ),
    );
    name.focus();
  }

  function show(tab: Tab) {
    for (const [name, el] of Object.entries(tabs)) el.classList.toggle("active", name === tab);
    if (tab === "open") showOpen();
    else if (tab === "presets") showPresets();
    else showManual();
  }

  show("open");
}
