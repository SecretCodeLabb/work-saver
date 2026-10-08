import { getVersion } from "@tauri-apps/api/app";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";

import { api, Config } from "../api";
import logoUrl from "../assets/DCN.svg";
import { t } from "../i18n";
import { store } from "../store";
import { button, card, field, h, numberInput, switchEl, viewHeader } from "../ui/dom";
import type { IconName } from "../ui/icons";
import type { View } from "./view";

const LINKS: { icon: IconName; title: string; url: string }[] = [
  { icon: "github", title: "GitHub", url: "https://github.com/SecretCodeLabb/work-saver" },
  { icon: "instagram", title: "Instagram", url: "https://www.instagram.com/dxnx.3d/" },
  { icon: "artstation", title: "ArtStation", url: "https://www.artstation.com/danimation21" },
];

const set = (mutate: (config: Config) => void) => store.update(mutate);

export const settingsView: View = {
  id: "settings",
  icon: "settings",
  label: "nav.settings",

  mount(root) {
    const content = h("div", { class: "stack", style: "gap:1rem" });
    root.append(h("div", { class: "view-inner" }, viewHeader(t("settings.title"), t("settings.subtitle")), content));

    // Se redibuja al cambiar la configuración (ej. desde la bandeja).
    const render = () => content.replaceChildren(smartCard(store.config), backupsCard(store.config), aboutCard());
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
      h(
        "div",
        { class: "links" },
        LINKS.map((link) => button(null, () => openUrl(link.url), { variant: "ghost", icon: link.icon, title: link.title })),
      ),
    ),
  );
}
