import { open as openDialog } from "@tauri-apps/plugin-dialog";

import type { Profile } from "../api";
import { formatCountdown, t } from "../i18n";
import { store } from "../store";
import { button, confirmDialog, field, h, initials, numberInput, switchEl, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import { shortcutInput } from "../ui/shortcut-input";
import { openAddProgram } from "./add-program";
import type { View } from "./view";

export function updateProfile(id: string, mutate: (profile: Profile) => void) {
  return store.update((c) => {
    const profile = c.profiles.find((p) => p.id === id);
    if (profile) mutate(profile);
  });
}

function textInput(value: string, onChange: (value: string) => void) {
  const input = h("input", { class: "input", value, style: "width:14rem" });
  input.addEventListener("change", () => onChange(input.value));
  return input;
}

/** Lista de carpetas con botones para añadir y quitar. */
function foldersEditor(profile: Profile) {
  const add = async () => {
    const folder = await openDialog({ directory: true, multiple: false, title: t("programs.folders.add") });
    if (typeof folder === "string") updateProfile(profile.id, (p) => p.watch_folders.push(folder));
  };
  return h(
    "div",
    { class: "stack", style: "gap:.25rem" },
    profile.watch_folders.length
      ? profile.watch_folders.map((folder) =>
          h(
            "div",
            { class: "row" },
            icon("folder", 16),
            h("span", { class: "list-item-main truncate small", title: folder }, folder),
            button(null, () => updateProfile(profile.id, (p) => (p.watch_folders = p.watch_folders.filter((f) => f !== folder))), {
              variant: "ghost",
              icon: "x",
              small: true,
              title: t("programs.remove"),
            }),
          ),
        )
      : h("div", { class: "small faint" }, t("programs.folders.empty")),
    h("div", null, button(t("programs.folders.add"), add, { small: true, icon: "plus" })),
  );
}

/** Lista editable como texto separado por comas. */
function listInput(values: string[], onChange: (values: string[]) => void) {
  return textInput(values.join(", "), (v) => onChange(v.split(",").map((x) => x.trim()).filter(Boolean)));
}

export const programsView: View = {
  id: "programs",
  icon: "apps",
  label: "nav.programs",

  mount(root) {
    const expanded = new Set<string>();
    const countdowns = new Map<string, HTMLElement>();
    const list = h("div", { class: "stack" });

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("programs.title"), t("programs.subtitle"), button(t("programs.add"), openAddProgram, { variant: "primary", icon: "plus" })),
        list,
      ),
    );

    function profileCard(profile: Profile) {
      const countdown = h("span", { class: "pill" });
      countdowns.set(profile.id, countdown);

      const toggle = switchEl(profile.enabled, (enabled) => updateProfile(profile.id, (p) => (p.enabled = enabled)));
      const details = h("details", { class: "card collapse", open: expanded.has(profile.id) });
      details.addEventListener("toggle", () => (details.open ? expanded.add(profile.id) : expanded.delete(profile.id)));

      const remove = async () => {
        const ok = await confirmDialog(
          t("programs.remove.title"),
          t("programs.remove.body", { name: profile.name }),
          t("programs.remove"),
          t("common.cancel"),
        );
        if (ok) store.update((c) => (c.profiles = c.profiles.filter((p) => p.id !== profile.id)));
      };

      details.append(
        h(
          "summary",
          { class: "row" },
          icon("chevron", 16),
          h("div", { class: "app-icon" }, initials(profile.name)),
          h(
            "div",
            { class: "list-item-main" },
            h("div", { class: "list-item-title truncate" }, profile.name),
            h(
              "div",
              { class: "list-item-sub truncate" },
              t("programs.summary", { exe: profile.exe, minutes: profile.interval_minutes, shortcut: profile.shortcut }),
            ),
          ),
          countdown,
          h("span", { onclick: (e: Event) => e.stopPropagation() }, toggle),
        ),
        h(
          "div",
          { style: "margin-top:1rem" },
          field(t("programs.name"), null, textInput(profile.name, (v) => updateProfile(profile.id, (p) => (p.name = v)))),
          field(
            t("programs.exe"),
            t("programs.exe.hint"),
            textInput(profile.exe, (v) => updateProfile(profile.id, (p) => (p.exe = v))),
          ),
          field(
            t("programs.interval"),
            t("programs.interval.hint"),
            numberInput(profile.interval_minutes, 1, 240, (v) => updateProfile(profile.id, (p) => (p.interval_minutes = v))),
            h("span", { class: "muted" }, t("common.min")),
          ),
          field(
            t("programs.shortcut"),
            t("programs.shortcut.hint"),
            shortcutInput(profile.shortcut, (v) => updateProfile(profile.id, (p) => (p.shortcut = v))),
          ),
          h("h3", { class: "card-title", style: "margin:1.25rem 0 .25rem" }, t("programs.backups")),
          field(
            t("programs.extensions"),
            t("programs.extensions.hint"),
            listInput(profile.extensions, (v) => updateProfile(profile.id, (p) => (p.extensions = v))),
          ),
          h(
            "div",
            { class: "field column" },
            h("div", { class: "field-label" }, h("span", null, t("programs.folders")), h("span", null, t("programs.folders.hint"))),
            foldersEditor(profile),
          ),
          h("h3", { class: "card-title", style: "margin:1.25rem 0 .25rem" }, t("programs.advanced")),
          field(
            t("programs.untitled"),
            t("programs.untitled.hint"),
            listInput(profile.untitled_markers, (v) => updateProfile(profile.id, (p) => (p.untitled_markers = v))),
          ),
          field(
            t("programs.dirty"),
            t("programs.dirty.hint"),
            listInput(profile.dirty_markers, (v) => updateProfile(profile.id, (p) => (p.dirty_markers = v))),
          ),
          h(
            "div",
            { class: "row", style: "margin-top:.75rem" },
            h("div", { class: "spacer" }),
            button(t("programs.remove"), remove, { variant: "danger", small: true, icon: "trash" }),
          ),
        ),
      );
      return details;
    }

    function render() {
      countdowns.clear();
      const profiles = store.config.profiles;
      list.replaceChildren(
        ...(profiles.length
          ? profiles.map(profileCard)
          : [
              h(
                "section",
                { class: "card empty" },
                h("strong", null, t("programs.empty.title")),
                h("p", null, t("programs.empty.body")),
                button(t("programs.add"), openAddProgram, { variant: "primary", icon: "plus" }),
              ),
            ]),
      );
      renderStatus();
    }

    function renderStatus() {
      const status = store.status;
      for (const profile of store.config.profiles) {
        const pill = countdowns.get(profile.id);
        if (!pill) continue;
        const ps = status?.profiles.find((p) => p.id === profile.id);
        const remaining = ps?.next_save_in;
        pill.title = ps?.reason ? t(`reason.${ps.reason}`) : "";
        if (!profile.enabled) {
          pill.textContent = t("programs.disabled");
          pill.className = "pill";
        } else if (!store.config.enabled || remaining === undefined) {
          pill.textContent = t("status.off");
          pill.className = "pill";
        } else if (remaining > 0) {
          pill.textContent = formatCountdown(remaining);
          pill.className = status?.active_profile === profile.id ? "pill on" : "pill";
        } else {
          pill.textContent = ps?.reason ? t(`reason.${ps.reason}`) : t("programs.pending");
          pill.className = ps?.reason === "untitled" || ps?.reason === "elevated" ? "pill danger" : "pill warn";
        }
      }
    }

    render();
    const offConfig = store.onConfig(render);
    const offStatus = store.onStatus(renderStatus);
    return () => {
      offConfig();
      offStatus();
    };
  },
};
