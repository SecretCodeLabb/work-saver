// Textos de la interfaz.

const es = {
  "nav.home": "Inicio",
  "nav.programs": "Programas",
  "nav.settings": "Ajustes",

  "status.on": "Activo",
  "status.off": "Apagado",

  "home.title": "Inicio",
  "home.subtitle": "El estado de tu autoguardado de un vistazo.",
  "home.on.title": "Protegiendo tu trabajo",
  "home.off.title": "Autoguardado desactivado",
  "home.off.sub": "Actívalo para empezar a guardar automáticamente.",
  "home.next": "Próximo guardado en {time}",
  "home.next.waitingApp": "Guardará en cuanto abras uno de tus programas",
  "home.lastSave": "Último guardado",
  "home.lastSave.none": "Aún no hay guardados en esta sesión.",
  "home.lastSave.value": "{time} · {app}",
  "home.foreground": "Ventana activa",
  "home.foreground.none": "Ninguna",
  "home.foreground.watched": "Vigilado",
  "home.foreground.notWatched": "No vigilado",
  "home.tip.title": "Consejo",
  "home.tip.body": "Guarda tu archivo manualmente una vez antes de activar el autoguardado; si el documento no tiene nombre, el programa abrirá «Guardar como».",

  "programs.title": "Programas",
  "programs.subtitle": "Elige qué programas se guardan automáticamente.",
  "programs.interval": "Intervalo de guardado",
  "programs.interval.hint": "Minutos entre cada guardado.",
  "programs.add": "Añadir",
  "programs.add.placeholder": "ej. blender.exe",
  "programs.list": "Programas vigilados",
  "programs.empty": "No hay programas. Añade el nombre del ejecutable (ej. krita.exe).",
  "programs.remove": "Quitar",

  "settings.title": "Ajustes",
  "settings.subtitle": "Personaliza Don't Crash Now.",
  "settings.about": "Acerca de",
  "settings.about.version": "Versión {version}",
  "settings.about.license": "Software libre y gratuito bajo licencia MIT.",
  "settings.about.source": "Código fuente",
  "settings.about.by": "Desarrollado por DXNX.3D",

  "common.min": "min",
  "common.cancel": "Cancelar",
  "common.save": "Guardar",
  "error.save": "No se pudo guardar la configuración",
};

export type Key = keyof typeof es;

const dict: Record<Key, string> = es;

export function t(key: Key, vars?: Record<string, string | number>): string {
  let text = dict[key] ?? key;
  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.split(`{${name}}`).join(String(value));
    }
  }
  return text;
}

/** 75 → "1:15", 3725 → "1:02:05". */
export function formatCountdown(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = String(s % 60).padStart(2, "0");
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, "0")}:${seconds}` : `${minutes}:${seconds}`;
}
