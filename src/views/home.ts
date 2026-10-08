import { formatCountdown, t } from "../i18n";
import { store } from "../store";
import { card, h, initials, switchEl, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import type { View } from "./view";

export const homeView: View = {
  id: "home",
  icon: "home",
  label: "nav.home",

  mount(root) {
    const toggle = switchEl(store.config.enabled, (enabled) => store.update((c) => (c.enabled = enabled)), true);
    const heroTitle = h("h2");
    const heroSub = h("p");
    const heroIcon = h("div", { class: "hero-icon" }, icon("shieldCheck", 26));
    const hero = h(
      "section",
      { class: "card hero" },
      heroIcon,
      h("div", { class: "hero-text" }, heroTitle, heroSub),
      toggle,
    );

    const lastSave = h("div", { class: "list-item-title" });
    const foregroundName = h("div", { class: "list-item-title truncate" });
    const foregroundTag = h("span", { class: "pill" });
    const foregroundIcon = h("div", { class: "app-icon" });

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("home.title"), t("home.subtitle")),
        hero,
        h(
          "div",
          { class: "grid-2" },
          card(t("home.lastSave"), h("div", { class: "row" }, icon("save"), lastSave)),
          card(
            t("home.foreground"),
            h("div", { class: "row" }, foregroundIcon, h("div", { class: "list-item-main" }, foregroundName), foregroundTag),
          ),
        ),
        h(
          "section",
          { class: "banner" },
          icon("alert"),
          h("div", null, h("strong", null, t("home.tip.title")), h("p", null, t("home.tip.body"))),
        ),
      ),
    );

    function render() {
      const config = store.config;
      const status = store.status;
      toggle.input.checked = config.enabled;
      hero.classList.toggle("on", config.enabled);

      if (config.enabled) {
        heroTitle.textContent = t("home.on.title");
        const next = status?.next_save_in;
        heroSub.textContent =
          next === null || next === undefined
            ? ""
            : next > 0
              ? t("home.next", { time: formatCountdown(next) })
              : t("home.next.waitingApp");
      } else {
        heroTitle.textContent = t("home.off.title");
        heroSub.textContent = t("home.off.sub");
      }

      const last = status?.last_save;
      lastSave.textContent = last ? t("home.lastSave.value", { time: last.time, app: last.app }) : t("home.lastSave.none");
      lastSave.classList.toggle("muted", !last);

      const fg = status?.foreground ?? null;
      foregroundName.textContent = fg ?? t("home.foreground.none");
      foregroundIcon.textContent = fg ? initials(fg) : "–";
      const watched = !!fg && config.processes.includes(fg);
      foregroundTag.textContent = watched ? t("home.foreground.watched") : t("home.foreground.notWatched");
      foregroundTag.className = watched ? "pill on" : "pill";
      foregroundTag.hidden = !fg;
    }

    render();
    const offStatus = store.onStatus(render);
    const offConfig = store.onConfig(render);
    return () => {
      offStatus();
      offConfig();
    };
  },
};
