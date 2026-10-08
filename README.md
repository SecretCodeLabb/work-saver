<p align="center">
  <img src="src/assets/DCN.svg" alt="Don't Crash Now" width="96">
</p>

<h1 align="center">Don't Crash Now!</h1>

<p align="center"><em>No vuelvas a perder 4 horas de trabajo... otra vez.</em></p>

**Don't Crash Now** (DCN) es una pequeña aplicación de escritorio para Windows que **guarda automáticamente tu trabajo** en programas de diseño, arte digital y 3D. Cada cierto intervalo de tiempo, si la ventana activa pertenece a uno de los programas que registraste, DCN envía el atajo **`Ctrl + S`** por ti.

Está pensada para software que se cuelga con frecuencia, que no tiene autoguardado nativo o cuyo autoguardado es poco fiable (Blender, Krita, Photoshop, Clip Studio, Aseprite, etc.).

---

## Índice

- [Cómo funciona](#cómo-funciona)
- [Características](#características)
- [Requisitos](#requisitos)
- [Instalación y desarrollo](#instalación-y-desarrollo)
- [Uso](#uso)
- [Arquitectura](#arquitectura)
- [Estructura del proyecto](#estructura-del-proyecto)
- [API interna (IPC)](#api-interna-ipc)
- [Limitaciones conocidas](#limitaciones-conocidas)
- [Créditos](#créditos)

---

## Cómo funciona

1. Activas el interruptor y defines un **intervalo** (en minutos) y una lista de **ejecutables** a vigilar (ej. `blender.exe`).
2. Un hilo en segundo plano revisa cada segundo si ya pasó el intervalo.
3. Cuando el intervalo se cumple, consulta a Windows qué proceso es dueño de la **ventana en primer plano**.
4. Si ese proceso está en tu lista, simula la pulsación `Ctrl + S` mediante la API `SendInput` de Win32 y reinicia el contador.
5. Si la ventana activa **no** es uno de tus programas, no hace nada y sigue esperando: el guardado se dispara en cuanto vuelvas a enfocar uno de ellos.
6. La interfaz muestra la hora y el programa del último guardado.

> DCN solo envía teclas a la ventana que tienes enfocada; nunca escribe en tus archivos directamente.

## Características

- Autoguardado por intervalo configurable (mínimo 1 minuto).
- Lista de programas vigilados editable (añadir / quitar chips).
- Solo actúa sobre la ventana activa, así que no interfiere con otras aplicaciones.
- Se minimiza a la **bandeja del sistema** al cerrar la ventana; sigue funcionando en segundo plano.
- Menú de bandeja con *Abrir UI* y *Salir*; clic izquierdo en el icono reabre la ventana.
- Indicador del último guardado en tiempo real (evento desde el backend).
- Binario ligero (Tauri + Rust, perfil release optimizado con LTO y `strip`).

## Requisitos

| Herramienta | Versión |
|---|---|
| Windows | 10 / 11 |
| [Node.js](https://nodejs.org/) | 18+ |
| [Rust](https://rustup.rs/) | stable (toolchain MSVC) |
| WebView2 | incluido en Windows 11; en Windows 10 lo instala el instalador de Tauri |
| Prerrequisitos de Tauri | ver [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) |

## Instalación y desarrollo

```bash
# Instalar dependencias del frontend
npm install

# Ejecutar en modo desarrollo (Vite en http://localhost:1420 + ventana Tauri)
npm run tauri dev

# Generar el instalador de producción (.msi / .exe en src-tauri/target/release/bundle)
npm run tauri build
```

Scripts disponibles en `package.json`:

| Script | Descripción |
|---|---|
| `npm run dev` | Solo el servidor de Vite (frontend) |
| `npm run build` | Compila TypeScript y empaqueta el frontend en `dist/` |
| `npm run tauri <cmd>` | CLI de Tauri (`dev`, `build`, `icon`, ...) |

## Uso

1. Abre **Don't Crash Now**.
2. En *Intervalo (minutos)* elige cada cuánto quieres guardar.
3. En *Agregar Software* escribe el nombre exacto del ejecutable y pulsa **Añadir** (o `Enter`).
   - Por defecto vienen `blender.exe` y `krita.exe`.
   - El nombre se compara en minúsculas. Puedes encontrarlo en el *Administrador de tareas → Detalles*.
4. Activa el interruptor. El estado cambia a **Encendido** y verás *Monitoreando...*.
5. Cierra la ventana: la app queda en la bandeja del sistema. Para salir del todo usa *Salir* en el menú de la bandeja.

**Importante:** guarda tu archivo manualmente al menos una vez antes de activar DCN. Si el documento no tiene nombre, `Ctrl + S` abrirá el diálogo *Guardar como* en cada intervalo.

Nombres de ejecutable habituales:

| Programa | Ejecutable |
|---|---|
| Blender | `blender.exe` |
| Krita | `krita.exe` |
| Adobe Photoshop | `photoshop.exe` |
| Adobe Illustrator | `illustrator.exe` |
| Clip Studio Paint | `clipstudiopaint.exe` |
| Aseprite | `aseprite.exe` |
| GIMP | `gimp-2.10.exe` (varía según versión) |
| ZBrush | `zbrush.exe` |

## Arquitectura

DCN es una app [Tauri 2](https://tauri.app/): un backend en **Rust** que hace el trabajo con la API de Windows y un frontend en **TypeScript + Vite** (sin framework) que sirve de panel de control.

```
┌──────────────────────────── Frontend (WebView) ────────────────────────────┐
│ index.html + src/main.ts + src/styles.css                                  │
│  - Interruptor ON/OFF, intervalo, lista de procesos                        │
│  - invoke("start_autosaver" | "stop_autosaver" | "get_status")             │
│  - listen("saved_event")  → actualiza "Guardado a las HH:MM:SS en X"        │
└──────────────────────────────────┬──────────────────────────────────────────┘
                                   │ IPC de Tauri
┌──────────────────────────────────▼──────────── Backend (Rust) ─────────────┐
│ src-tauri/src/lib.rs                                                       │
│  - AppState: Arc<Mutex<AppStateData>> compartido                           │
│  - Comandos IPC que leen/escriben el estado                                │
│  - Icono y menú de bandeja; cerrar ventana = ocultar                       │
│  - Hilo de temporizador (tick de 1 s):                                     │
│       si is_running y pasó el intervalo                                    │
│         y proceso activo ∈ target_processes                                │
│           → win32::send_ctrl_s() → emit("saved_event")                     │
│                                                                            │
│ src-tauri/src/win32.rs                                                     │
│  - get_active_process_name(): GetForegroundWindow → PID → OpenProcess      │
│                               → GetModuleFileNameExW → "blender.exe"       │
│  - send_ctrl_s(): SendInput con Ctrl↓ S↓ S↑ Ctrl↑                           │
└────────────────────────────────────────────────────────────────────────────┘
```

### Estado compartido

```rust
pub struct AppStateData {
    pub is_running: bool,             // interruptor ON/OFF
    pub interval_minutes: u64,        // intervalo de guardado
    pub target_processes: Vec<String>,// ejecutables vigilados (minúsculas)
    pub last_saved_time: Option<String>,
}
```

El estado vive solo en memoria: al reiniciar la app vuelve a los valores por defecto (`1` minuto, `blender.exe` y `krita.exe`, apagado).

### Temporizador

- Hay **un único contador global**, no uno por programa.
- Mientras la app está en pausa, el contador se reinicia en cada tick, así que al activarla no guarda de inmediato.
- Si el intervalo vence mientras otra aplicación está enfocada, el guardado queda pendiente y se dispara en el primer segundo en que se enfoca un programa vigilado.

## Estructura del proyecto

```
work-saver/
├── index.html                  # Estructura de la UI
├── src/
│   ├── main.ts                 # Lógica de la UI e IPC con el backend
│   ├── styles.css              # Tema oscuro monocromo
│   └── assets/DCN.svg          # Logo
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # Punto de entrada (oculta la consola en release)
│   │   ├── lib.rs              # Estado, comandos, bandeja y temporizador
│   │   └── win32.rs            # Detección de ventana activa y envío de Ctrl+S
│   ├── capabilities/default.json  # Permisos de Tauri (core, opener)
│   ├── tauri.conf.json         # Ventana, bundle e iconos
│   ├── Cargo.toml              # Dependencias Rust (tauri, windows, chrono, serde)
│   └── icons/                  # Iconos de la app
├── package.json                # Dependencias y scripts del frontend
├── vite.config.ts              # Configuración de Vite para Tauri
└── tsconfig.json
```

## API interna (IPC)

### Comandos (frontend → backend)

| Comando | Parámetros | Devuelve | Efecto |
|---|---|---|---|
| `start_autosaver` | `interval: u64`, `processes: string[]` | `()` | Activa el autoguardado y reemplaza intervalo y lista. También se usa para actualizar la configuración en caliente. |
| `stop_autosaver` | — | `()` | Pone `is_running = false`. |
| `get_status` | — | `AppStateData` | Estado actual; la UI lo usa al arrancar. |

### Eventos (backend → frontend)

| Evento | Payload | Cuándo |
|---|---|---|
| `saved_event` | `{ time: "HH:MM:SS", process: "blender.exe" }` | Justo después de enviar `Ctrl + S`. |

## Limitaciones conocidas

- **Solo Windows.** `win32.rs` usa la API Win32; el proyecto no compila en macOS/Linux.
- **La configuración no se guarda** entre sesiones.
- **Untitled / Guardar como:** si el documento nunca se guardó, cada intervalo abrirá el diálogo de guardado.
- **Teclas pulsadas:** si estás manteniendo `Shift` o `Alt` en el momento del guardado, la combinación resultante puede ser otra (ej. `Ctrl + Shift + S` = *Guardar como*). Si mantienes `Ctrl`, el `Ctrl↑` simulado lo suelta.
- **Guardado a mitad de trazo:** el atajo se envía aunque estés dibujando o arrastrando con el ratón/lápiz.
- **Programas con privilegios de administrador:** Windows (UIPI) bloquea `SendInput` hacia ventanas con mayor nivel de integridad. Si el programa corre como administrador, DCN también debe hacerlo.
- **No verifica** que el guardado haya ocurrido realmente; solo confirma que envió las teclas.
- Un contador global: guardar en un programa reinicia el tiempo de todos.

## Créditos

Desarrollado por **DXNX.3D**

- GitHub: [SecretCodeLabb](https://github.com/SecretCodeLabb)
- Instagram: [@dxnx.3d](https://www.instagram.com/dxnx.3d/)
- ArtStation: [danimation21](https://www.artstation.com/danimation21)

Construido con [Tauri](https://tauri.app/), [Rust](https://www.rust-lang.org/), [TypeScript](https://www.typescriptlang.org/) y [Vite](https://vitejs.dev/).

## Licencia

Don't Crash Now es **software libre y gratuito** bajo licencia [MIT](LICENSE).
