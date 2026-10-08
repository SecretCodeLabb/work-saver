// Estado de la interfaz: configuración y último estado recibido del backend.

import { api, Config, onConfigChanged, onStatus, Status } from "./api";
import { t } from "./i18n";
import { toast } from "./ui/dom";

type Listener = () => void;

class Store {
  config!: Config;
  status: Status | null = null;
  private configListeners = new Set<Listener>();
  private statusListeners = new Set<Listener>();

  async init() {
    this.config = await api.getConfig();
    // Cambios hechos desde la bandeja, el atajo global o al terminar una pausa.
    await onConfigChanged(() => this.reload());
    await onStatus((status) => {
      this.status = status;
      this.statusListeners.forEach((fn) => fn());
    });
  }

  /** Aplica un cambio a una copia de la configuración y la envía al backend. */
  async update(mutate: (config: Config) => void) {
    const next = structuredClone(this.config);
    mutate(next);
    try {
      this.config = await api.setConfig(next);
    } catch (error) {
      console.error(error);
      toast(t("error.save"), "error");
    }
    this.notifyConfig();
  }

  /** Recarga la configuración (cuando cambia desde la bandeja, por ejemplo). */
  async reload() {
    this.config = await api.getConfig();
    this.notifyConfig();
  }

  get paused(): boolean {
    return this.config.enabled && this.config.paused_until !== null;
  }

  onConfig(fn: Listener): () => void {
    this.configListeners.add(fn);
    return () => this.configListeners.delete(fn);
  }

  onStatus(fn: Listener): () => void {
    this.statusListeners.add(fn);
    return () => this.statusListeners.delete(fn);
  }

  private notifyConfig() {
    this.configListeners.forEach((fn) => fn());
  }
}

export const store = new Store();
