import { getVersion } from "@tauri-apps/api/app";

import { onSaved, onSaveUnverified } from "./api";
import { setLanguage, t } from "./i18n";
import { store } from "./store";
import { h, toast } from "./ui/dom";
import { icon } from "./ui/icons";
import { socialLinks } from "./ui/social";
import { activityView } from "./views/activity";
import { backupsView } from "./views/backups";
import { homeView } from "./views/home";
import { programsView } from "./views/programs";
import { settingsView } from "./views/settings";
import type { View } from "./views/view";

const views: View[] = [homeView, programsView, backupsView, activityView, settingsView];

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
  renderCrashBadge();
}

/** Punto rojo en «Inicio» mientras haya un cierre inesperado sin revisar. */
function renderCrashBadge() {
  const homeItem = nav.querySelector(".nav-item");
  const hasAlert = !!store.status?.crash_alert;
  const badge = homeItem?.querySelector(".badge");
  if (hasAlert && homeItem && !badge) homeItem.append(h("span", { class: "badge" }, "!"));
  if (!hasAlert) badge?.remove();
}

function renderBrandStatus() {
  const { enabled } = store.config;
  brandStatus.textContent = !enabled ? t("status.off") : store.paused ? t("status.paused") : t("status.on");
  brandStatus.className = !enabled ? "pill" : store.paused ? "pill warn" : "pill on";
}

/** Tono breve de confirmación (dos notas suaves). */
function chime() {
  const ctx = new AudioContext();
  [660, 880].forEach((freq, i) => {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    const start = ctx.currentTime + i * 0.12;
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(0.0001, start);
    gain.gain.exponentialRampToValueAtTime(0.08, start + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, start + 0.25);
    osc.connect(gain).connect(ctx.destination);
    osc.start(start);
    osc.stop(start + 0.3);
  });
  setTimeout(() => ctx.close(), 800);
}

async function main() {
  await store.init();
  setLanguage(store.config.language);
  const language = store.config.language;
  // Cambiar de idioma redibuja toda la interfaz.
  store.onConfig(() => {
    if (store.config.language !== language) location.reload();
  });
  store.onConfig(renderBrandStatus);
  store.onStatus(renderCrashBadge);
  renderBrandStatus();
  navigate("home");
  document.addEventListener("navigate", (e) => navigate((e as CustomEvent<string>).detail));
  await onSaved(() => {
    if (store.config.notifications.sound) chime();
  });
  await onSaveUnverified((check) => toast(t("backups.unverified", { app: check.app }), "error"));
  document.getElementById("sidebar-social")!.replaceWith(socialLinks("sidebar-social"));
  sidebarFooter.textContent = `v${await getVersion()} · MIT · DXNX.3D`;
}

main().catch((error) => console.error(error));
