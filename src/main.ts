import { getVersion } from "@tauri-apps/api/app";

import { t } from "./i18n";
import { store } from "./store";
import { h } from "./ui/dom";
import { icon } from "./ui/icons";
import { homeView } from "./views/home";
import { programsView } from "./views/programs";
import { settingsView } from "./views/settings";
import type { View } from "./views/view";

const views: View[] = [homeView, programsView, settingsView];

const nav = document.getElementById("nav")!;
const viewRoot = document.getElementById("view")!;
const brandStatus = document.getElementById("brand-status")!;
const sidebarFooter = document.getElementById("sidebar-footer")!;

let currentId = "";
let cleanup: (() => void) | void;

function renderNav() {
  nav.replaceChildren(
    ...views.map((view) =>
      h(
        "button",
        {
          class: view.id === currentId ? "nav-item active" : "nav-item",
          type: "button",
          onclick: () => navigate(view.id),
        },
        icon(view.icon),
        h("span", null, t(view.label)),
      ),
    ),
  );
}

export function navigate(id: string) {
  const view = views.find((v) => v.id === id) ?? views[0];
  cleanup?.();
  currentId = view.id;
  viewRoot.replaceChildren();
  viewRoot.scrollTop = 0;
  cleanup = view.mount(viewRoot);
  renderNav();
}

function renderBrandStatus() {
  const on = store.config.enabled;
  brandStatus.textContent = on ? t("status.on") : t("status.off");
  brandStatus.className = on ? "pill on" : "pill";
}

async function main() {
  await store.init();
  store.onConfig(renderBrandStatus);
  renderBrandStatus();
  navigate("home");
  sidebarFooter.textContent = `v${await getVersion()} · MIT`;
}

main().catch((error) => console.error(error));
