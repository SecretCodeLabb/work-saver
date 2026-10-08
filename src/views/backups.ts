import { api, BackupEntry, BackupGroup, onBackupCreated } from "../api";
import { formatBytes, formatDateTime, t } from "../i18n";
import { store } from "../store";
import { button, confirmDialog, h, initials, toast, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import type { View } from "./view";

export const backupsView: View = {
  id: "backups",
  icon: "archive",
  label: "nav.backups",

  mount(root) {
    const expanded = new Set<string>();
    const notice = h("div");
    const list = h("div", { class: "stack" });
    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(
          t("backups.title"),
          t("backups.subtitle"),
          button(t("backups.openFolder"), () => api.openBackupRoot().catch(showError), { icon: "folder" }),
        ),
        notice,
        list,
      ),
    );

    function renderNotice() {
      const { config } = store;
      const hasFolders = config.profiles.some((p) => p.enabled && p.watch_folders.length);
      const message = !config.backups.enabled ? t("backups.disabled") : !hasFolders ? t("backups.noFolders") : null;
      notice.replaceChildren(
        message
          ? h(
              "section",
              { class: "banner warn" },
              icon("alert"),
              h("div", { class: "list-item-main" }, h("p", { style: "margin:0;color:var(--text)" }, message)),
              hasFolders ? null : button(t("backups.goPrograms"), () => document.dispatchEvent(new CustomEvent("navigate", { detail: "programs" })), { small: true }),
            )
          : "",
      );
    }

    function versionRow(entry: BackupEntry, isLatest: boolean) {
      const restore = async () => {
        const ok = await confirmDialog(t("backups.restore.title"), t("backups.restore.body"), t("backups.restore"), t("common.cancel"));
        if (!ok) return;
        try {
          const path = await api.restoreBackup(entry.id);
          toast(t("backups.restored", { path }));
        } catch (error) {
          showError(error);
        }
      };
      return h(
        "div",
        { class: "list-item" },
        icon("clock", 16),
        h(
          "div",
          { class: "list-item-main" },
          h("span", { class: "list-item-title" }, formatDateTime(entry.created_ms)),
          isLatest ? h("span", { class: "pill on", style: "margin-left:.5rem" }, "★") : null,
          h("div", { class: "list-item-sub" }, formatBytes(entry.size)),
        ),
        button(t("backups.restore"), restore, { small: true, icon: "restore" }),
        button(null, () => api.revealBackup(entry.id).catch(showError), { variant: "ghost", icon: "folder", title: t("backups.reveal") }),
        button(null, () => api.deleteBackup(entry.id).then(load, showError), { variant: "ghost", icon: "trash", title: t("common.delete") }),
      );
    }

    function groupCard(group: BackupGroup) {
      const details = h("details", { class: "card collapse", open: expanded.has(group.original) });
      details.addEventListener("toggle", () =>
        details.open ? expanded.add(group.original) : expanded.delete(group.original),
      );
      const removeAll = async () => {
        const ok = await confirmDialog(
          t("backups.deleteAll.title"),
          t("backups.deleteAll.body", { count: group.entries.length, name: group.name }),
          t("common.delete"),
          t("common.cancel"),
        );
        if (ok) api.deleteBackupGroup(group.original).then(load, showError);
      };
      details.append(
        h(
          "summary",
          { class: "row" },
          icon("chevron", 16),
          h("div", { class: "app-icon" }, initials(group.app)),
          h(
            "div",
            { class: "list-item-main" },
            h("div", { class: "list-item-title truncate" }, group.name),
            h("div", { class: "list-item-sub truncate", title: group.original }, group.original),
          ),
          h(
            "div",
            { style: "text-align:right" },
            h("div", { class: "small" }, t("backups.versions", { count: group.entries.length, size: formatBytes(group.total_size) })),
            h("div", { class: "small muted" }, t("backups.latest", { date: formatDateTime(group.entries[0].created_ms) })),
          ),
        ),
        h(
          "div",
          { class: "list", style: "margin-top:1rem" },
          group.entries.map((entry, i) => versionRow(entry, i === 0)),
        ),
        h(
          "div",
          { class: "row", style: "margin-top:.75rem" },
          h("div", { class: "spacer" }),
          button(t("backups.deleteAll"), removeAll, { variant: "danger", small: true, icon: "trash" }),
        ),
      );
      return details;
    }

    async function load() {
      renderNotice();
      const groups = await api.listBackups();
      list.replaceChildren(
        ...(groups.length
          ? groups.map(groupCard)
          : [
              h(
                "section",
                { class: "card empty" },
                icon("archive", 28),
                h("strong", { style: "margin-top:.5rem" }, t("backups.empty.title")),
                h("p", null, t("backups.empty.body")),
              ),
            ]),
      );
    }

    load().catch(showError);
    const offConfig = store.onConfig(renderNotice);
    const unlisten = onBackupCreated(() => load().catch(showError));
    return () => {
      offConfig();
      unlisten.then((fn) => fn());
    };
  },
};

function showError(error: unknown) {
  console.error(error);
  toast(String(error), "error");
}
