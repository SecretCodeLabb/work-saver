//! Textos del backend (bandeja y notificaciones).

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
}

/// Texto con `{nombre}` sustituido por los valores dados.
pub fn tr(text: Text, vars: &[(&str, &str)]) -> String {
    let mut out = es(text).to_string();
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
    }
}
