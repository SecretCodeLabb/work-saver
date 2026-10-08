// Textos de la interfaz.

const es = {
  "nav.home": "Inicio",
  "nav.programs": "Programas",
  "nav.backups": "Respaldos",
  "nav.settings": "Ajustes",

  "status.on": "Activo",
  "status.off": "Apagado",
  "status.paused": "En pausa",

  "home.title": "Inicio",
  "home.subtitle": "El estado de tu autoguardado de un vistazo.",
  "home.on.title": "Protegiendo tu trabajo",
  "home.off.title": "Autoguardado desactivado",
  "home.off.sub": "Actívalo para empezar a guardar automáticamente.",
  "home.paused.title": "En pausa",
  "home.paused.sub": "El autoguardado se reanudará a las {time}.",
  "home.pause": "Pausar {label}",
  "home.resume": "Reanudar",
  "home.pause.15": "15 min",
  "home.pause.30": "30 min",
  "home.pause.60": "1 h",
  "home.next": "Próximo guardado de {app} en {time}",
  "home.saving": "Guardando {app}…",
  "home.waiting": "Esperando a que uses uno de tus programas.",
  "home.noPrograms": "Añade un programa para empezar.",
  "home.lastSave": "Último guardado",
  "home.lastSave.none": "Aún no hay guardados en esta sesión.",
  "home.lastSave.value": "{time} · {app}",
  "home.lastSave.verified": "Verificado: {file}",
  "home.lastSave.unverified": "No se detectó el archivo guardado",
  "home.lastSave.unchecked": "Añade una carpeta de proyecto para verificar los guardados",
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

  "programs.backups": "Respaldos y verificación",
  "programs.extensions": "Extensiones de archivo",
  "programs.extensions.hint": "Tipos de archivo del programa, ej. blend, kra, psd. Separadas por comas.",
  "programs.folders": "Carpetas de proyecto",
  "programs.folders.hint": "Los archivos guardados aquí se respaldan con historial y sirven para verificar cada guardado.",
  "programs.folders.add": "Añadir carpeta",
  "programs.folders.empty": "Sin carpetas: no se harán respaldos de este programa.",
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
  "reason.countdown": "Guardando en unos segundos…",

  "backups.title": "Respaldos",
  "backups.subtitle": "Historial de versiones de tus archivos. Restaurar nunca sobrescribe el original.",
  "backups.openFolder": "Abrir carpeta",
  "backups.empty.title": "Aún no hay respaldos",
  "backups.empty.body": "En Programas, añade las carpetas de proyecto de cada programa. Cada vez que se guarde un archivo, se copiará aquí con fecha y hora.",
  "backups.disabled": "Los respaldos están desactivados en Ajustes.",
  "backups.noFolders": "Ningún programa tiene carpetas de proyecto: no se harán respaldos.",
  "backups.goPrograms": "Ir a Programas",
  "backups.versions": "{count} versiones · {size}",
  "backups.latest": "Última: {date}",
  "backups.restore": "Restaurar",
  "backups.restore.title": "¿Restaurar esta versión?",
  "backups.restore.body": "Se creará una copia junto al archivo original con el nombre «(restaurado …)». El original no se modifica.",
  "backups.restored": "Restaurado en {path}",
  "backups.reveal": "Mostrar en carpeta",
  "backups.delete": "Eliminar",
  "backups.deleteAll": "Eliminar historial",
  "backups.deleteAll.title": "¿Eliminar todas las versiones?",
  "backups.deleteAll.body": "Se borrarán las {count} copias de {name}. El archivo original no se toca.",
  "backups.created": "Respaldo creado: {name}",
  "backups.unverified": "{app}: no se detectó el archivo guardado. Revisa el programa.",

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
  "settings.backups": "Respaldos",
  "settings.backups.enabled": "Crear respaldos",
  "settings.backups.enabled.hint": "Copia cada archivo guardado de tus carpetas de proyecto a un historial.",
  "settings.backups.folder": "Carpeta de respaldos",
  "settings.backups.change": "Cambiar",
  "settings.backups.reset": "Predeterminada",
  "settings.backups.keep": "Versiones por archivo",
  "settings.backups.keep.hint": "Las más antiguas se borran al superar el límite.",
  "settings.backups.gap": "Tiempo mínimo entre copias",
  "settings.backups.gap.hint": "Evita llenar el historial si guardas muy seguido. 0 = copiar siempre.",
  "settings.backups.max": "Espacio máximo",
  "settings.backups.max.hint": "Se conserva siempre la última versión de cada archivo.",
  "settings.notifications": "Notificaciones",
  "settings.notifications.onSave": "Al guardar",
  "settings.notifications.onSave.hint": "Muestra una notificación de Windows en cada autoguardado.",
  "settings.notifications.onProblem": "Problemas",
  "settings.notifications.onProblem.hint": "Documento sin nombre, guardado no verificado o programa como administrador.",
  "settings.notifications.warn": "Avisar antes de guardar",
  "settings.notifications.warn.hint": "Útil en programas que se congelan unos segundos al guardar.",
  "settings.notifications.warnSeconds": "Antelación del aviso",
  "settings.notifications.sound": "Sonido al guardar",
  "settings.notifications.sound.hint": "Un tono breve y discreto.",
  "settings.startup": "Inicio",
  "settings.startup.autostart": "Iniciar con Windows",
  "settings.startup.autostart.hint": "Recomendado: así nunca olvidas activar la protección.",
  "settings.startup.minimized": "Iniciar en la bandeja",
  "settings.startup.minimized.hint": "Al encender el equipo, arranca sin abrir la ventana.",
  "settings.hotkey": "Atajo global",
  "settings.hotkey.label": "Activar / desactivar",
  "settings.hotkey.hint": "Funciona desde cualquier programa.",
  "settings.hotkey.none": "Sin atajo",
  "settings.hotkey.clear": "Quitar",
  "settings.hotkey.default": "Restablecer",
  "settings.about": "Acerca de",
  "settings.about.version": "Versión {version}",
  "settings.about.license": "Software libre y gratuito bajo licencia MIT.",
  "settings.about.source": "Código fuente",
  "settings.about.by": "Desarrollado por DXNX.3D",

  "common.min": "min",
  "common.sec": "s",
  "common.mb": "MB",
  "common.delete": "Eliminar",
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

const DATE_LOCALE = "es";

/** 1536 → "1,5 KB". */
export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toLocaleString(DATE_LOCALE, { maximumFractionDigits: unit ? 1 : 0 })} ${units[unit]}`;
}

export function formatDateTime(ms: number): string {
  return new Date(ms).toLocaleString(DATE_LOCALE, { dateStyle: "medium", timeStyle: "short" });
}
