import { listen } from "@tauri-apps/api/event";

import { ActivityKind, ActivitySummary, api, DayStats } from "../api";
import { formatDateTime, formatDuration, t } from "../i18n";
import { button, card, confirmDialog, h, toast, viewHeader } from "../ui/dom";
import { icon, IconName } from "../ui/icons";
import type { View } from "./view";

const KIND_ICON: Record<ActivityKind, IconName> = {
  save: "save",
  skip: "check",
  backup: "archive",
  unverified: "alert",
  problem: "alert",
  crash: "alert",
};

const KIND_COLOR: Partial<Record<ActivityKind, string>> = {
  save: "var(--success)",
  unverified: "var(--warning)",
  problem: "var(--warning)",
  crash: "var(--danger)",
};

/** Fila de cifras: guardados, respaldos, tiempo protegido y cierres. */
export function statTiles(stats: DayStats) {
  const tile = (value: string, label: string) =>
    h("div", { class: "stat" }, h("span", { class: "stat-value" }, value), h("span", { class: "stat-label" }, label));
  return h(
    "div",
    { class: "grid-4" },
    tile(String(stats.saves), t("activity.saves")),
    tile(String(stats.backups), t("activity.backups")),
    tile(formatDuration(stats.protected_seconds), t("activity.protected")),
    tile(String(stats.crashes), t("activity.crashes")),
  );
}

export const activityView: View = {
  id: "activity",
  icon: "activity",
  label: "nav.activity",

  mount(root) {
    const content = h("div", { class: "stack", style: "gap:1rem" });
    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(
          t("activity.title"),
          t("activity.subtitle"),
          button(t("activity.openLog"), () => api.openLogFolder().catch((e) => toast(String(e), "error")), { icon: "file" }),
        ),
        content,
      ),
    );

    const clear = async () => {
      const ok = await confirmDialog(t("activity.clear.title"), t("activity.clear.body"), t("common.delete"), t("common.cancel"));
      if (ok) api.clearActivity().then(load);
    };

    function render(summary: ActivitySummary) {
      content.replaceChildren(
        card(t("activity.today"), statTiles(summary.today)),
        card(t("activity.week"), statTiles(summary.week)),
        card(t("activity.total"), statTiles(summary.total)),
        card(
          h("span", { class: "row", style: "width:100%;justify-content:space-between" }, t("activity.log"), summary.events.length ? button(t("activity.clear"), clear, { small: true, variant: "ghost" }) : null),
          summary.events.length
            ? h(
                "div",
                { class: "list" },
                summary.events.map((event) =>
                  h(
                    "div",
                    { class: "list-item" },
                    h("span", { style: `color:${KIND_COLOR[event.kind] ?? "var(--text-muted)"}` }, icon(KIND_ICON[event.kind], 16)),
                    h(
                      "div",
                      { class: "list-item-main" },
                      h("div", { class: "list-item-title truncate" }, `${t(`activity.kind.${event.kind}`)} · ${event.app}`),
                      event.detail ? h("div", { class: "list-item-sub truncate", title: event.detail }, event.detail) : null,
                    ),
                    h("span", { class: "small muted" }, formatDateTime(event.time_ms)),
                  ),
                ),
              )
            : h("div", { class: "empty" }, t("activity.empty")),
        ),
      );
    }

    const load = () => api.getActivity().then(render);
    load();
    const unlisten = listen("activity", load);
    return () => {
      unlisten.then((fn) => fn());
    };
  },
};
