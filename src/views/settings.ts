import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

import logoUrl from "../assets/DCN.svg";
import { t } from "../i18n";
import { button, card, h, viewHeader } from "../ui/dom";
import type { IconName } from "../ui/icons";
import type { View } from "./view";

const LINKS: { icon: IconName; title: string; url: string }[] = [
  { icon: "github", title: "GitHub", url: "https://github.com/SecretCodeLabb/work-saver" },
  { icon: "instagram", title: "Instagram", url: "https://www.instagram.com/dxnx.3d/" },
  { icon: "artstation", title: "ArtStation", url: "https://www.artstation.com/danimation21" },
];

export const settingsView: View = {
  id: "settings",
  icon: "settings",
  label: "nav.settings",

  mount(root) {
    const version = h("span", { class: "muted" });
    getVersion().then((v) => (version.textContent = t("settings.about.version", { version: v })));

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("settings.title"), t("settings.subtitle")),
        aboutCard(version),
      ),
    );
  },
};

function aboutCard(version: HTMLElement) {
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
