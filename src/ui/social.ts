// Redes sociales del autor (barra lateral y Ajustes → Acerca de).

import { openUrl } from "@tauri-apps/plugin-opener";

import { button, h } from "./dom";
import type { IconName } from "./icons";

const LINKS: { icon: IconName; title: string; url: string }[] = [
  { icon: "github", title: "GitHub", url: "https://github.com/SecretCodeLabb/work-saver" },
  { icon: "instagram", title: "Instagram", url: "https://www.instagram.com/dxnx.3d/" },
  { icon: "artstation", title: "ArtStation", url: "https://www.artstation.com/danimation21" },
];

/** Botones de icono que abren cada red en el navegador. */
export function socialLinks(className = "links"): HTMLElement {
  return h(
    "div",
    { class: className },
    LINKS.map((link) =>
      button(null, () => openUrl(link.url), { variant: "ghost", icon: link.icon, title: link.title }),
    ),
  );
}
