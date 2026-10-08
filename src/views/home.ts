import { formatCountdown, formatDateTime, t } from "../i18n";
import { store } from "../store";
import { api } from "../api";
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
    const heroActions = h("div", { class: "hero-actions" });
    const hero = h(
      "section",
      { class: "card hero" },
      h("div", { class: "hero-icon" }, icon("shieldCheck", 26)),
      h("div", { class: "hero-text" }, heroTitle, heroSub, heroActions),
      toggle,
    );
    const pauseButtons = h(
      "div",
      { class: "row wrap" },
      ([15, 30, 60] as const).map((minutes) =>
        button(t("home.pause", { label: t(`home.pause.${minutes}`) }), () => api.pause(minutes), { small: true, icon: "pause" }),
      ),
    );
    const resumeButton = button(t("home.resume"), () => api.resume(), { small: true, variant: "primary", icon: "play" });

    const lastSave = h("div", { class: "list-item-title" });
    const lastSaveCheck = h("div", { class: "list-item-sub" });
    const foregroundName = h("div", { class: "list-item-title truncate" });
    const foregroundTitle = h("div", { class: "list-item-sub truncate" });
    const foregroundTag = h("span", { class: "pill" });
    const foregroundIcon = h("div", { class: "app-icon" });
    const programs = h("div", { class: "list" });
    const crashBanner = h("div");
    let crashKey = "";

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("home.title"), t("home.subtitle")),
        crashBanner,
        hero,
        h(
          "div",
          { class: "grid-2" },
          card(t("home.lastSave"), h("div", { class: "row" }, icon("save"), h("div", { class: "list-item-main" }, lastSave, lastSaveCheck))),
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
      hero.classList.toggle("on", config.enabled && !store.paused);
      hero.classList.toggle("paused", store.paused);

      const actions = !config.enabled ? null : store.paused ? resumeButton : pauseButtons;
      if (heroActions.firstChild !== actions) heroActions.replaceChildren(actions ?? "");
      heroActions.hidden = !actions;

      if (!config.enabled) {
        heroTitle.textContent = t("home.off.title");
        heroSub.textContent = t("home.off.sub");
        return;
      }
      if (store.paused) {
        const time = new Date(config.paused_until!).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
        heroTitle.textContent = t("home.paused.title");
        heroSub.textContent = t("home.paused.sub", { time });
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
          const running = config.enabled && !store.paused;
          const progress = running ? Math.min(100, ((total - remaining) / total) * 100) : 0;
          const isActive = status?.active_profile === profile.id;
          const label = !config.enabled
            ? t("status.off")
            : store.paused
              ? t("status.paused")
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
                { class: remaining === 0 && running ? "progress warn" : isActive ? "progress on" : "progress" },
                h("div", { style: `width:${progress}%` }),
              ),
            ),
          );
        }),
      );
    }

    function renderCrash() {
      const alert = store.status?.crash_alert ?? null;
      const key = alert ? `${alert.profile_id}${alert.time}${alert.code}` : "";
      if (key === crashKey) return;
      crashKey = key;
      if (!alert) return crashBanner.replaceChildren();
      const lines = [
        t("crash.code", { code: alert.code }),
        alert.last_save ? t("crash.lastSave", { time: alert.last_save.time }) : t("crash.noSave"),
        alert.last_backup ? t("crash.backup", { date: formatDateTime(alert.last_backup.created_ms) }) : null,
        t("crash.recovery"),
      ];
      crashBanner.replaceChildren(
        h(
          "section",
          { class: "banner danger" },
          icon("alert"),
          h(
            "div",
            { class: "list-item-main" },
            h("strong", null, t("crash.title", { app: alert.app, time: alert.time })),
            h("p", null, lines.filter(Boolean).join(" ")),
            h(
              "div",
              { class: "hero-actions" },
              button(t("crash.viewBackups"), () => document.dispatchEvent(new CustomEvent("navigate", { detail: "backups" })), {
                small: true,
                variant: "primary",
                icon: "archive",
              }),
              button(t("crash.dismiss"), () => api.dismissCrashAlert(), { small: true, variant: "ghost" }),
            ),
          ),
        ),
      );
    }

    function renderStatus() {
      renderCrash();
      renderHero();
      renderPrograms();

      const { config, status } = store;
      const last = status?.last_save;
      lastSave.textContent = last ? t("home.lastSave.value", { time: last.time, app: last.app }) : t("home.lastSave.none");
      lastSave.classList.toggle("muted", !last);
      const profile = config.profiles.find((p) => p.id === last?.profile_id);
      lastSaveCheck.textContent = !last
        ? ""
        : last.verified === true
          ? `✓ ${t("home.lastSave.verified", { file: last.file ?? "" })}`
          : last.verified === false
            ? t("home.lastSave.unverified")
            : profile && !profile.watch_folders.length
              ? t("home.lastSave.unchecked")
              : "";
      lastSaveCheck.style.color = last?.verified === true ? "var(--success)" : last?.verified === false ? "var(--warning)" : "";

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
