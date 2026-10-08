import { t } from "../i18n";
import { store } from "../store";
import { button, card, field, h, initials, numberInput, viewHeader } from "../ui/dom";
import { icon } from "../ui/icons";
import type { View } from "./view";

export const programsView: View = {
  id: "programs",
  icon: "apps",
  label: "nav.programs",

  mount(root) {
    const interval = numberInput(store.config.interval_minutes, 1, 240, (value) =>
      store.update((c) => (c.interval_minutes = value)),
    );

    const input = h("input", { class: "input", placeholder: t("programs.add.placeholder") });
    const add = () => {
      const exe = input.value.trim().toLowerCase();
      if (!exe) return;
      input.value = "";
      store.update((c) => c.processes.push(exe));
    };
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") add();
    });

    const list = h("div", { class: "list" });

    root.append(
      h(
        "div",
        { class: "view-inner" },
        viewHeader(t("programs.title"), t("programs.subtitle")),
        card(null, field(t("programs.interval"), t("programs.interval.hint"), interval, h("span", { class: "muted" }, t("common.min")))),
        card(
          t("programs.list"),
          h("div", { class: "row", style: "margin-bottom:1rem" }, input, button(t("programs.add"), add, { variant: "primary", icon: "plus" })),
          list,
        ),
      ),
    );

    function render() {
      interval.value = String(store.config.interval_minutes);
      const processes = store.config.processes;
      list.replaceChildren(
        ...(processes.length
          ? processes.map((exe) =>
              h(
                "div",
                { class: "list-item" },
                h("div", { class: "app-icon" }, initials(exe)),
                h("div", { class: "list-item-main list-item-title" }, exe),
                button(null, () => store.update((c) => (c.processes = c.processes.filter((p) => p !== exe))), {
                  variant: "ghost",
                  icon: "trash",
                  title: t("programs.remove"),
                }),
              ),
            )
          : [h("div", { class: "empty" }, icon("apps", 28), h("p", null, t("programs.empty")))]),
      );
    }

    render();
    return store.onConfig(render);
  },
};
