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
  "home.next": "Próximo guardado de {app} en {time}",
  "home.saving": "Guardando {app}…",
  "home.waiting": "Esperando a que uses uno de tus programas.",
  "home.noPrograms": "Añade un programa para empezar.",
  "home.lastSave": "Último guardado",
  "home.lastSave.none": "Aún no hay guardados en esta sesión.",
  "home.lastSave.value": "{time} · {app}",
  "home.foreground": "Ventana activa",
  "home.foreground.none": "Ninguna",
  "home.foreground.watched": "Vigilado",
  "home.foreground.notWatched": "No vigilado",
  "home.programs": "Programas vigilados",
  "home.programs.empty": "Todavía no vigilas ningún programa.",
  "home.inUse": "En uso",
  "home.tip.title": "Consejo",
  "home.tip.body": "Guarda tu archivo manualmente una vez antes de activar el autoguardado; si el documento no tiene nombre, el programa abrirá «Guardar como».",

  "programs.title": "Programas",
  "programs.subtitle": "Cada programa tiene su propio intervalo y atajo de guardado.",
  "programs.add": "Añadir programa",
  "programs.summary": "{exe} · cada {minutes} min · {shortcut}",
  "programs.name": "Nombre",
  "programs.exe": "Ejecutable",
  "programs.exe.hint": "Nombre del proceso, como aparece en el Administrador de tareas.",
  "programs.interval": "Intervalo",
  "programs.interval.hint": "Minutos entre cada guardado.",
  "programs.shortcut": "Atajo de guardado",
  "programs.shortcut.hint": "Haz clic y pulsa la combinación. Casi siempre es Ctrl+S.",
  "programs.disabled": "Desactivado",
  "programs.pending": "Pendiente",
  "programs.remove": "Quitar",
  "programs.remove.title": "¿Quitar programa?",
  "programs.remove.body": "{name} dejará de guardarse automáticamente.",
  "programs.empty.title": "Sin programas",
  "programs.empty.body": "Añade los programas que quieres proteger.",

  "programs.advanced": "Detección del título",
  "programs.untitled": "Marcas de documento sin nombre",
  "programs.untitled.hint": "Si el título contiene alguna, no se guarda (abriría «Guardar como»). Separadas por comas.",
  "programs.dirty": "Marcas de cambios sin guardar",
  "programs.dirty.hint": "Ej. el «*» que muchos programas añaden al título. Separadas por comas.",

  "reason.busy": "Esperando una pausa",
  "reason.input_held": "Esperando a que sueltes el ratón o las teclas",
  "reason.dialog": "Hay un diálogo abierto",
  "reason.untitled": "El documento no tiene nombre: guárdalo una vez",
  "reason.elevated": "El programa corre como administrador",
  "reason.no_changes": "Sin cambios: guardado omitido",

  "add.title": "Añadir programa",
  "add.tab.open": "Abiertos ahora",
  "add.tab.presets": "Populares",
  "add.tab.manual": "Manual",
  "add.open.hint": "Programas con ventanas abiertas en este momento.",
  "add.open.empty": "No se encontraron ventanas.",
  "add.refresh": "Actualizar",
  "add.search": "Buscar programa…",
  "add.presets.empty": "Sin resultados. Prueba en «Manual».",
  "add.manual.name": "Nombre",
  "add.manual.exe": "Ejecutable",
  "add.manual.hint": "Encuéntralo en Administrador de tareas → Detalles.",
  "add.submit": "Añadir",
  "add.already": "Añadido",
  "add.added": "{name} añadido",
  "add.duplicate": "Ese programa ya está en la lista",

  "shortcut.press": "Pulsa el atajo…",
  "shortcut.hint": "Haz clic y pulsa la combinación de teclas",

  "settings.title": "Ajustes",
  "settings.subtitle": "Personaliza Don't Crash Now.",
  "settings.smart": "Guardado inteligente",
  "settings.smart.idle": "Esperar una pausa",
  "settings.smart.idle.hint": "No guarda mientras dibujas o escribes; espera a que dejes de usar el teclado, ratón o lápiz.",
  "settings.smart.idleSeconds": "Duración de la pausa",
  "settings.smart.idleSeconds.hint": "Segundos sin actividad antes de guardar.",
  "settings.smart.maxWait": "Espera máxima",
  "settings.smart.maxWait.hint": "Pasado este tiempo guarda en la siguiente pausa breve, aunque sigas trabajando.",
  "settings.smart.untitled": "Omitir documentos sin nombre",
  "settings.smart.untitled.hint": "Evita que aparezca «Guardar como» cada vez.",
  "settings.smart.dialogs": "Omitir si hay un diálogo abierto",
  "settings.smart.dialogs.hint": "No envía el atajo a ventanas de exportar, preferencias, etc.",
  "settings.smart.dirty": "Guardar solo si hay cambios",
  "settings.smart.dirty.hint": "Usa la marca del título (ej. «*»). Si el programa no la muestra, guarda siempre.",
  "settings.about": "Acerca de",
  "settings.about.version": "Versión {version}",
  "settings.about.license": "Software libre y gratuito bajo licencia MIT.",
  "settings.about.source": "Código fuente",
  "settings.about.by": "Desarrollado por DXNX.3D",

  "common.min": "min",
  "common.sec": "s",
  "common.cancel": "Cancelar",
  "common.save": "Guardar",
  "common.loading": "Cargando…",
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
