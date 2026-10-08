import { getVersion } from "@tauri-apps/api/app";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

import { api, Config } from "../api";
import logoUrl from "../assets/DCN.svg";
import { t } from "../i18n";
import { store } from "../store";
import { button, card, field, h, numberInput, switchEl, viewHeader } from "../ui/dom";
import { shortcutInput } from "../ui/shortcut-input";
import { socialLinks } from "../ui/social";
import type { View } from "./view";

const set = (mutate: (config: Config) => void) => store.update(mutate);

export const settingsView: View = {
  id: "settings",
  icon: "settings",
  label: "nav.settings",

  mount(root) {
    const content = h("div", { class: "stack", style: "gap:1rem" });
    root.append(h("div", { class: "view-inner" }, viewHeader(t("settings.title"), t("settings.subtitle")), content));

    // Se redibuja al cambiar la configuración (ej. desde la bandeja).
    const render = () =>
      content.replaceChildren(
        smartCard(store.config),
        backupsCard(store.config),
        notificationsCard(store.config),
        startupCard(store.config),
        hotkeyCard(store.config),
        languageCard(store.config),
        aboutCard(),
      );
    render();
    return store.onConfig(render);
  },
};

function smartCard(config: Config) {
  const smart = config.smart;
  return card(
    t("settings.smart"),
    field(
      t("settings.smart.idle"),
      t("settings.smart.idle.hint"),
      switchEl(smart.wait_for_idle, (v) => set((c) => (c.smart.wait_for_idle = v))),
    ),
    smart.wait_for_idle
      ? field(
          t("settings.smart.idleSeconds"),
          t("settings.smart.idleSeconds.hint"),
          numberInput(smart.idle_seconds, 1, 30, (v) => set((c) => (c.smart.idle_seconds = v))),
          h("span", { class: "muted" }, t("common.sec")),
        )
      : null,
    smart.wait_for_idle
      ? field(
          t("settings.smart.maxWait"),
          t("settings.smart.maxWait.hint"),
          numberInput(smart.max_wait_seconds, 10, 900, (v) => set((c) => (c.smart.max_wait_seconds = v))),
          h("span", { class: "muted" }, t("common.sec")),
        )
      : null,
    field(
      t("settings.smart.dirty"),
      t("settings.smart.dirty.hint"),
      switchEl(smart.only_when_dirty, (v) => set((c) => (c.smart.only_when_dirty = v))),
    ),
    field(
      t("settings.smart.untitled"),
      t("settings.smart.untitled.hint"),
      switchEl(smart.skip_untitled, (v) => set((c) => (c.smart.skip_untitled = v))),
    ),
    field(
      t("settings.smart.dialogs"),
      t("settings.smart.dialogs.hint"),
      switchEl(smart.skip_dialogs, (v) => set((c) => (c.smart.skip_dialogs = v))),
    ),
  );
}

function backupsCard(config: Config) {
  const backups = config.backups;
  const folder = h("span", { class: "small muted truncate", style: "max-width:18rem" });
  api.backupRoot().then((path) => {
    folder.textContent = path;
    folder.title = path;
  });
  const change = async () => {
    const picked = await openDialog({ directory: true, multiple: false, title: t("settings.backups.folder") });
    if (typeof picked === "string") set((c) => (c.backups.folder = picked));
  };
  return card(
    t("settings.backups"),
    field(
      t("settings.backups.enabled"),
      t("settings.backups.enabled.hint"),
      switchEl(backups.enabled, (v) => set((c) => (c.backups.enabled = v))),
    ),
    field(
      t("settings.backups.folder"),
      null,
      folder,
      button(t("settings.backups.change"), change, { small: true, icon: "folder" }),
      backups.folder ? button(t("settings.backups.reset"), () => set((c) => (c.backups.folder = "")), { small: true, variant: "ghost" }) : null,
    ),
    field(
      t("settings.backups.keep"),
      t("settings.backups.keep.hint"),
      numberInput(backups.keep_per_file, 1, 500, (v) => set((c) => (c.backups.keep_per_file = v))),
    ),
    field(
      t("settings.backups.gap"),
      t("settings.backups.gap.hint"),
      numberInput(backups.min_minutes_between, 0, 240, (v) => set((c) => (c.backups.min_minutes_between = v))),
      h("span", { class: "muted" }, t("common.min")),
    ),
    field(
      t("settings.backups.max"),
      t("settings.backups.max.hint"),
      numberInput(backups.max_total_mb, 50, 1_000_000, (v) => set((c) => (c.backups.max_total_mb = v))),
      h("span", { class: "muted" }, t("common.mb")),
    ),
  );
}

function notificationsCard(config: Config) {
  const n = config.notifications;
  return card(
    t("settings.notifications"),
    field(
      t("settings.notifications.onProblem"),
      t("settings.notifications.onProblem.hint"),
      switchEl(n.on_problem, (v) => set((c) => (c.notifications.on_problem = v))),
    ),
    field(
      t("settings.notifications.onSave"),
      t("settings.notifications.onSave.hint"),
      switchEl(n.on_save, (v) => set((c) => (c.notifications.on_save = v))),
    ),
    field(
      t("settings.notifications.warn"),
      t("settings.notifications.warn.hint"),
      switchEl(n.warn_before, (v) => set((c) => (c.notifications.warn_before = v))),
    ),
    n.warn_before
      ? field(
          t("settings.notifications.warnSeconds"),
          null,
          numberInput(n.warn_seconds, 1, 60, (v) => set((c) => (c.notifications.warn_seconds = v))),
          h("span", { class: "muted" }, t("common.sec")),
        )
      : null,
    field(
      t("settings.notifications.sound"),
      t("settings.notifications.sound.hint"),
      switchEl(n.sound, (v) => set((c) => (c.notifications.sound = v))),
    ),
  );
}

function startupCard(config: Config) {
  return card(
    t("settings.startup"),
    field(
      t("settings.startup.autostart"),
      t("settings.startup.autostart.hint"),
      switchEl(config.autostart, (v) => set((c) => (c.autostart = v))),
    ),
    config.autostart
      ? field(
          t("settings.startup.minimized"),
          t("settings.startup.minimized.hint"),
          switchEl(config.start_minimized, (v) => set((c) => (c.start_minimized = v))),
        )
      : null,
  );
}

const DEFAULT_HOTKEY = "Ctrl+Shift+Alt+F9";

function hotkeyCard(config: Config) {
  const input = shortcutInput(config.toggle_hotkey || t("settings.hotkey.none"), (v) => set((c) => (c.toggle_hotkey = v)));
  return card(
    t("settings.hotkey"),
    field(
      t("settings.hotkey.label"),
      t("settings.hotkey.hint"),
      input,
      config.toggle_hotkey
        ? button(t("settings.hotkey.clear"), () => set((c) => (c.toggle_hotkey = "")), { small: true, variant: "ghost" })
        : button(t("settings.hotkey.default"), () => set((c) => (c.toggle_hotkey = DEFAULT_HOTKEY)), { small: true, variant: "ghost" }),
    ),
  );
}

function languageCard(config: Config) {
  const select = h(
    "select",
    { class: "select", style: "width:14rem" },
    h("option", { value: "auto" }, t("settings.language.auto")),
    h("option", { value: "es" }, "Español"),
    h("option", { value: "en" }, "English"),
  );
  select.value = config.language;
  select.addEventListener("change", () => set((c) => (c.language = select.value)));
  return card(t("settings.language"), field(t("settings.language"), t("settings.language.hint"), select));
}

function aboutCard() {
  const version = h("span", { class: "muted" });
  getVersion().then((v) => (version.textContent = t("settings.about.version", { version: v })));
  return card(
    t("settings.about"),
    h(
      "div",
      { class: "row" },
      h("img", { src: logoUrl, width: 40, height: 40, alt: "" }),
      h(
        "div",
        { class: "list-item-main" },
        h("div", { class: "list-item-title" }, "Don't Crash Now!"),
        version,
        h("div", { class: "small muted" }, t("settings.about.license")),
        h("div", { class: "small faint" }, t("settings.about.by")),
      ),
      socialLinks(),
    ),
  );
}
