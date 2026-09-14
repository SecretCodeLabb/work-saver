import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";

// Elementos del DOM
const toggleSwitch = document.getElementById("toggle-switch") as HTMLInputElement;
const statusLabel = document.getElementById("status-label") as HTMLSpanElement;
const intervalInput = document.getElementById("interval-input") as HTMLInputElement;
const processInput = document.getElementById("process-input") as HTMLInputElement;
const addProcessBtn = document.getElementById("add-process-btn") as HTMLButtonElement;
const processList = document.getElementById("process-list") as HTMLDivElement;
const lastSavedInfo = document.getElementById("last-saved-info") as HTMLParagraphElement;

// Estado local para los procesos
let currentProcesses: string[] = [];

// Interfaz para el estado que viene del backend
interface AppState {
  is_running: boolean;
  interval_minutes: number;
  target_processes: string[];
  last_saved_time: string | null;
}

interface SaveEvent {
  time: string;
  process: string;
}

// Inicializar
async function init() {
  try {
    const state: AppState = await invoke("get_status");

    toggleSwitch.checked = state.is_running;
    updateStatusLabel(state.is_running);
    intervalInput.value = state.interval_minutes.toString();

    currentProcesses = state.target_processes;
    renderProcesses();

    if (state.last_saved_time) {
      lastSavedInfo.textContent = state.last_saved_time;
    }

    // Escuchar eventos de guardado
    await listen<SaveEvent>("saved_event", (event) => {
      const { time, process } = event.payload;
      lastSavedInfo.textContent = `Guardado a las ${time} en ${process}`;
    });
  } catch (error) {
    console.error("Error al obtener estado:", error);
  }
}

// Actualizar UI del status
function updateStatusLabel(isRunning: boolean) {
  if (isRunning) {
    statusLabel.textContent = "Encendido";
    statusLabel.classList.remove("off");
    statusLabel.classList.add("on");
  } else {
    statusLabel.textContent = "Apagado";
    statusLabel.classList.remove("on");
    statusLabel.classList.add("off");
  }
}

// Renderizar chips de procesos
function renderProcesses() {
  processList.innerHTML = "";
  currentProcesses.forEach((proc, index) => {
    const chip = document.createElement("div");
    chip.className = "process-chip";

    const text = document.createElement("span");
    text.textContent = proc;

    const removeBtn = document.createElement("button");
    removeBtn.className = "remove-btn";
    removeBtn.innerHTML = "&times;";
    removeBtn.onclick = () => {
      currentProcesses.splice(index, 1);
      renderProcesses();
      updateBackendStateIfRunning();
    };

    chip.appendChild(text);
    chip.appendChild(removeBtn);
    processList.appendChild(chip);
  });
}

// Sincronizar con el backend si está corriendo
async function updateBackendStateIfRunning() {
  if (toggleSwitch.checked) {
    const interval = parseInt(intervalInput.value) || 60;
    try {
      await invoke("start_autosaver", {
        interval: interval,
        processes: currentProcesses
      });
    } catch (error) {
      console.error("Error al actualizar backend:", error);
    }
  }
}

// Listeners
toggleSwitch.addEventListener("change", async () => {
  const isRunning = toggleSwitch.checked;
  updateStatusLabel(isRunning);

  try {
    if (isRunning) {
      const interval = parseInt(intervalInput.value) || 60;
      await invoke("start_autosaver", {
        interval: interval,
        processes: currentProcesses
      });
      lastSavedInfo.textContent = "Monitoreando...";
    } else {
      await invoke("stop_autosaver");
      lastSavedInfo.textContent = "Pausado";
    }
  } catch (error) {
    console.error("Error al cambiar estado:", error);
    // Revertir UI si falla
    toggleSwitch.checked = !isRunning;
    updateStatusLabel(!isRunning);
  }
});

addProcessBtn.addEventListener("click", () => {
  const newVal = processInput.value.trim().toLowerCase();
  if (newVal && !currentProcesses.includes(newVal)) {
    // Si no termina en .exe, podríamos agregarlo, pero dejemos que el usuario escriba bien
    currentProcesses.push(newVal);
    processInput.value = "";
    renderProcesses();
    updateBackendStateIfRunning();
  }
});

processInput.addEventListener("keypress", (e) => {
  if (e.key === "Enter") {
    addProcessBtn.click();
  }
});

intervalInput.addEventListener("change", () => {
  updateBackendStateIfRunning();
});

// Enlaces del pie de página (abrir en navegador)
document.querySelectorAll(".footer-link").forEach(link => {
  link.addEventListener("click", async (e) => {
    e.preventDefault();
    const url = (e.currentTarget as HTMLAnchorElement).href;
    if (url) {
      try {
        await openUrl(url);
      } catch (err) {
        console.error("Error al abrir URL:", err);
      }
    }
  });
});

// Arrancar la app
init();
