import { formatCountdown, t } from "../i18n";
import { store } from "../store";
import { button, card, h, initials, switchEl, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import { openAddProgram } from "./add-program";
import type { View } from "./view";

export const homeView: View = {
  id: "home",
  icon: "home",
  label: "nav.home",

  mount(root) {
    const toggle = switchEl(store.config.enabled, (enabled) => store.update((c) => (c.enabled = enabled)), true);
    const heroTitle = h("h2");
    const heroSub = h("p");
    const hero = h(
      "section",
      { class: "card hero" },
      h("div", { class: "hero-icon" }, icon("shieldCheck", 26)),
      h("div", { class: "hero-text" }, heroTitle, heroSub),
      toggle,
    );

    const lastSave = h("div", { class: "list-item-title" });
    const foregroundName = h("div", { class: "list-item-title truncate" });
    const foregroundTitle = h("div", { class: "list-item-sub truncate" });
    const foregroundTag = h("span", { class: "pill" });
    const foregroundIcon = h("div", { class: "app-icon" });
    const programs = h("div", { class: "list" });

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
            h(
              "div",
              { class: "row" },
              foregroundIcon,
              h("div", { class: "list-item-main" }, foregroundName, foregroundTitle),
              foregroundTag,
            ),
          ),
        ),
        card(t("home.programs"), programs),
        h(
          "section",
          { class: "banner" },
          icon("alert"),
          h("div", null, h("strong", null, t("home.tip.title")), h("p", null, t("home.tip.body"))),
        ),
      ),
    );

    function renderHero() {
      const { config, status } = store;
      toggle.input.checked = config.enabled;
      hero.classList.toggle("on", config.enabled);

      if (!config.enabled) {
        heroTitle.textContent = t("home.off.title");
        heroSub.textContent = t("home.off.sub");
        return;
      }
      heroTitle.textContent = t("home.on.title");
      const active = config.profiles.find((p) => p.id === status?.active_profile);
      const activeStatus = status?.profiles.find((p) => p.id === active?.id);
      const remaining = activeStatus?.next_save_in;
      if (!config.profiles.some((p) => p.enabled)) {
        heroSub.textContent = t("home.noPrograms");
      } else if (active && remaining !== undefined) {
        heroSub.textContent =
          remaining > 0
            ? t("home.next", { app: active.name, time: formatCountdown(remaining) })
            : activeStatus?.reason
              ? `${active.name}: ${t(`reason.${activeStatus.reason}`)}`
              : t("home.saving", { app: active.name });
      } else {
        heroSub.textContent = t("home.waiting");
      }
    }

    function renderPrograms() {
      const { config, status } = store;
      const enabled = config.profiles.filter((p) => p.enabled);
      if (!enabled.length) {
        programs.replaceChildren(
          h(
            "div",
            { class: "empty" },
            h("p", { style: "margin-top:0" }, t("home.programs.empty")),
            button(t("programs.add"), openAddProgram, { variant: "primary", icon: "plus" }),
          ),
        );
        return;
      }
      programs.replaceChildren(
        ...enabled.map((profile) => {
          const ps = status?.profiles.find((p) => p.id === profile.id);
          const remaining = ps?.next_save_in ?? 0;
          const total = profile.interval_minutes * 60;
          const progress = config.enabled ? Math.min(100, ((total - remaining) / total) * 100) : 0;
          const isActive = status?.active_profile === profile.id;
          const label = !config.enabled
            ? t("status.off")
            : remaining > 0
              ? formatCountdown(remaining)
              : ps?.reason
                ? t(`reason.${ps.reason}`)
                : t("programs.pending");
          return h(
            "div",
            { class: "list-item" },
            h("div", { class: "app-icon" }, initials(profile.name)),
            h(
              "div",
              { class: "list-item-main stack", style: "gap:.4rem" },
              h(
                "div",
                { class: "row" },
                h("span", { class: "list-item-title truncate" }, profile.name),
                isActive ? h("span", { class: "pill on" }, t("home.inUse")) : null,
                h("span", { class: "spacer" }),
                h("span", { class: "small muted mono" }, label),
              ),
              h(
                "div",
                { class: remaining === 0 && config.enabled ? "progress warn" : isActive ? "progress on" : "progress" },
                h("div", { style: `width:${progress}%` }),
              ),
            ),
          );
        }),
      );
    }

    function renderStatus() {
      renderHero();
      renderPrograms();

      const { config, status } = store;
      const last = status?.last_save;
      lastSave.textContent = last ? t("home.lastSave.value", { time: last.time, app: last.app }) : t("home.lastSave.none");
      lastSave.classList.toggle("muted", !last);

      const fg = status?.foreground ?? null;
      const watched = !!fg && config.profiles.some((p) => p.enabled && p.exe === fg.exe);
      foregroundName.textContent = fg?.exe ?? t("home.foreground.none");
      foregroundTitle.textContent = fg?.title ?? "";
      foregroundIcon.textContent = fg ? initials(fg.exe) : "–";
      foregroundTag.textContent = watched ? t("home.foreground.watched") : t("home.foreground.notWatched");
      foregroundTag.className = watched ? "pill on" : "pill";
      foregroundTag.hidden = !fg;
    }

    renderStatus();
    const offStatus = store.onStatus(renderStatus);
    const offConfig = store.onConfig(renderStatus);
    return () => {
      offStatus();
      offConfig();
    };
  },
};
