import type { Profile } from "../api";
import { formatCountdown, t } from "../i18n";
import { store } from "../store";
import { button, confirmDialog, field, h, initials, numberInput, switchEl, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import { shortcutInput } from "../ui/shortcut-input";
import { openAddProgram } from "./add-program";
import type { View } from "./view";

export function updateProfile(id: string, mutate: (profile: Profile) => void) {
  return store.update((c) => {
    const profile = c.profiles.find((p) => p.id === id);
    if (profile) mutate(profile);
  });
}

function textInput(value: string, onChange: (value: string) => void) {
  const input = h("input", { class: "input", value, style: "width:14rem" });
  input.addEventListener("change", () => onChange(input.value));
  return input;
}

export const programsView: View = {
  id: "programs",
  icon: "apps",
  label: "nav.programs",

  mount(root) {
    const expanded = new Set<string>();
    const countdowns = new Map<string, HTMLElement>();
    const list = h("div", { class: "stack" });

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("programs.title"), t("programs.subtitle"), button(t("programs.add"), openAddProgram, { variant: "primary", icon: "plus" })),
        list,
      ),
    );

    function profileCard(profile: Profile) {
      const countdown = h("span", { class: "pill" });
      countdowns.set(profile.id, countdown);

      const toggle = switchEl(profile.enabled, (enabled) => updateProfile(profile.id, (p) => (p.enabled = enabled)));
      const details = h("details", { class: "card collapse", open: expanded.has(profile.id) });
      details.addEventListener("toggle", () => (details.open ? expanded.add(profile.id) : expanded.delete(profile.id)));

      const remove = async () => {
        const ok = await confirmDialog(
          t("programs.remove.title"),
          t("programs.remove.body", { name: profile.name }),
          t("programs.remove"),
          t("common.cancel"),
        );
        if (ok) store.update((c) => (c.profiles = c.profiles.filter((p) => p.id !== profile.id)));
      };

      details.append(
        h(
          "summary",
          { class: "row" },
          icon("chevron", 16),
          h("div", { class: "app-icon" }, initials(profile.name)),
          h(
            "div",
            { class: "list-item-main" },
            h("div", { class: "list-item-title truncate" }, profile.name),
            h(
              "div",
              { class: "list-item-sub truncate" },
              t("programs.summary", { exe: profile.exe, minutes: profile.interval_minutes, shortcut: profile.shortcut }),
            ),
          ),
          countdown,
          h("span", { onclick: (e: Event) => e.stopPropagation() }, toggle),
        ),
        h(
          "div",
          { style: "margin-top:1rem" },
          field(t("programs.name"), null, textInput(profile.name, (v) => updateProfile(profile.id, (p) => (p.name = v)))),
          field(
            t("programs.exe"),
            t("programs.exe.hint"),
            textInput(profile.exe, (v) => updateProfile(profile.id, (p) => (p.exe = v))),
          ),
          field(
            t("programs.interval"),
            t("programs.interval.hint"),
            numberInput(profile.interval_minutes, 1, 240, (v) => updateProfile(profile.id, (p) => (p.interval_minutes = v))),
            h("span", { class: "muted" }, t("common.min")),
          ),
          field(
            t("programs.shortcut"),
            t("programs.shortcut.hint"),
            shortcutInput(profile.shortcut, (v) => updateProfile(profile.id, (p) => (p.shortcut = v))),
          ),
          h(
            "div",
            { class: "row", style: "margin-top:.75rem" },
            h("div", { class: "spacer" }),
            button(t("programs.remove"), remove, { variant: "danger", small: true, icon: "trash" }),
          ),
        ),
      );
      return details;
    }

    function render() {
      countdowns.clear();
      const profiles = store.config.profiles;
      list.replaceChildren(
        ...(profiles.length
          ? profiles.map(profileCard)
          : [
              h(
                "section",
                { class: "card empty" },
                h("strong", null, t("programs.empty.title")),
                h("p", null, t("programs.empty.body")),
                button(t("programs.add"), openAddProgram, { variant: "primary", icon: "plus" }),
              ),
            ]),
      );
      renderStatus();
    }

    function renderStatus() {
      const status = store.status;
      for (const profile of store.config.profiles) {
        const pill = countdowns.get(profile.id);
        if (!pill) continue;
        const remaining = status?.profiles.find((p) => p.id === profile.id)?.next_save_in;
        if (!profile.enabled) {
          pill.textContent = t("programs.disabled");
          pill.className = "pill";
        } else if (!store.config.enabled || remaining === undefined) {
          pill.textContent = t("status.off");
          pill.className = "pill";
        } else if (remaining > 0) {
          pill.textContent = formatCountdown(remaining);
          pill.className = status?.active_profile === profile.id ? "pill on" : "pill";
        } else {
          pill.textContent = t("programs.pending");
          pill.className = "pill warn";
        }
      }
    }

    render();
    const offConfig = store.onConfig(render);
    const offStatus = store.onStatus(renderStatus);
    return () => {
      offConfig();
      offStatus();
    };
  },
};
