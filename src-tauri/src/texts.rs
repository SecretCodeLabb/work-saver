//! Textos del backend (bandeja y notificaciones) en español e inglés.

use std::sync::atomic::{AtomicBool, Ordering};

static ENGLISH: AtomicBool = AtomicBool::new(false);

/// Aplica el idioma de la configuración ("auto" = idioma de Windows).
pub fn set_language(setting: &str) {
    let english = match setting {
        "es" => false,
        "en" => true,
        _ => !system_is_spanish(),
    };
    ENGLISH.store(english, Ordering::Relaxed);
}

fn system_is_spanish() -> bool {
    const LANG_SPANISH: u16 = 0x0A;
    let lang_id = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
    lang_id & 0x3FF == LANG_SPANISH
}

#[derive(Clone, Copy)]
pub enum Text {
    TrayOpen,
    TrayEnabled,
    TrayPause,
    TrayPause15,
    TrayPause30,
    TrayPause60,
    TrayResume,
    TrayQuit,
    StatusOff,
    StatusActive,
    StatusPausedUntil,
    StatusNextSave,
    NotifySavedTitle,
    NotifySavedBody,
    NotifyWarnTitle,
    NotifyWarnBody,
    NotifyUntitledTitle,
    NotifyUntitledBody,
    NotifyElevatedTitle,
    NotifyElevatedBody,
    NotifyUnverifiedTitle,
    NotifyUnverifiedBody,
    NotifyCrashTitle,
    NotifyCrashLastSave,
    NotifyCrashBackup,
    NotifyCrashNothing,
    NotifyEnabled,
    NotifyDisabled,
    NotifyResumed,
    RestoredSuffix,
}

/// Texto con `{nombre}` sustituido por los valores dados.
pub fn tr(text: Text, vars: &[(&str, &str)]) -> String {
    let template = if ENGLISH.load(Ordering::Relaxed) { en(text) } else { es(text) };
    let mut out = template.to_string();
    for (name, value) in vars {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

fn es(text: Text) -> &'static str {
    use Text::*;
    match text {
        TrayOpen => "Abrir Don't Crash Now",
        TrayEnabled => "Autoguardado activo",
        TrayPause => "Pausar",
        TrayPause15 => "15 minutos",
        TrayPause30 => "30 minutos",
        TrayPause60 => "1 hora",
        TrayResume => "Reanudar ahora",
        TrayQuit => "Salir",
        StatusOff => "Autoguardado desactivado",
        StatusActive => "Protegiendo tu trabajo",
        StatusPausedUntil => "En pausa hasta las {time}",
        StatusNextSave => "{app}: próximo guardado en {minutes} min",
        NotifySavedTitle => "Guardado: {app}",
        NotifySavedBody => "Tu trabajo se guardó a las {time}.",
        NotifyWarnTitle => "Guardando {app} en {seconds} s",
        NotifyWarnBody => "Suelta el lápiz un momento para guardar.",
        NotifyUntitledTitle => "{app}: documento sin nombre",
        NotifyUntitledBody => "Guárdalo una vez manualmente para que el autoguardado pueda protegerlo.",
        NotifyElevatedTitle => "{app} corre como administrador",
        NotifyElevatedBody => "Windows impide enviarle teclas. Ejecuta Don't Crash Now también como administrador.",
        NotifyUnverifiedTitle => "{app}: no se detectó el guardado",
        NotifyUnverifiedBody => "Se envió el atajo pero el archivo no cambió. Revisa el programa.",
        NotifyCrashTitle => "{app} se cerró inesperadamente",
        NotifyCrashLastSave => "Último autoguardado a las {time}. Abre Don't Crash Now para ver tus respaldos.",
        NotifyCrashBackup => "Abre Don't Crash Now para recuperar tu último respaldo.",
        NotifyCrashNothing => "No hubo autoguardados en esta sesión. Revisa la carpeta de recuperación del programa.",
        NotifyEnabled => "Autoguardado activado",
        NotifyDisabled => "Autoguardado desactivado",
        NotifyResumed => "Autoguardado reanudado",
        RestoredSuffix => "restaurado",
    }
}

fn en(text: Text) -> &'static str {
    use Text::*;
    match text {
        TrayOpen => "Open Don't Crash Now",
        TrayEnabled => "Autosave on",
        TrayPause => "Pause",
        TrayPause15 => "15 minutes",
        TrayPause30 => "30 minutes",
        TrayPause60 => "1 hour",
        TrayResume => "Resume now",
        TrayQuit => "Quit",
        StatusOff => "Autosave is off",
        StatusActive => "Protecting your work",
        StatusPausedUntil => "Paused until {time}",
        StatusNextSave => "{app}: next save in {minutes} min",
        NotifySavedTitle => "Saved: {app}",
        NotifySavedBody => "Your work was saved at {time}.",
        NotifyWarnTitle => "Saving {app} in {seconds} s",
        NotifyWarnBody => "Lift your pen for a moment to save.",
        NotifyUntitledTitle => "{app}: untitled document",
        NotifyUntitledBody => "Save it once manually so autosave can protect it.",
        NotifyElevatedTitle => "{app} is running as administrator",
        NotifyElevatedBody => "Windows blocks sending keys to it. Run Don't Crash Now as administrator too.",
        NotifyUnverifiedTitle => "{app}: save not detected",
        NotifyUnverifiedBody => "The shortcut was sent but the file didn't change. Check the program.",
        NotifyCrashTitle => "{app} closed unexpectedly",
        NotifyCrashLastSave => "Last autosave at {time}. Open Don't Crash Now to see your backups.",
        NotifyCrashBackup => "Open Don't Crash Now to recover your latest backup.",
        NotifyCrashNothing => "There were no autosaves in this session. Check the program's recovery folder.",
        NotifyEnabled => "Autosave turned on",
        NotifyDisabled => "Autosave turned off",
        NotifyResumed => "Autosave resumed",
        RestoredSuffix => "restored",
    }
}
