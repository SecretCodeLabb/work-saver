import type { Key } from "../i18n";
import type { IconName } from "../ui/icons";

export interface View {
  id: string;
  icon: IconName;
  label: Key;
  /** Dibuja la vista en `root`. Devuelve una función de limpieza opcional. */
  mount(root: HTMLElement): (() => void) | void;
}
