import init, { ShaderRenderer } from "./pkg/shaderand.js";

let renderer;
let isRunning = true;
let showingShaderCode = false;

function showStatus(message, isError = false) {
  const statusDiv = document.getElementById("status");
  statusDiv.textContent = message;
  statusDiv.className = `status ${isError ? "error" : "success"}`;
  statusDiv.style.display = "block";

  setTimeout(() => {
    statusDiv.style.display = "none";
  }, 3000);
}

function updateShaderDisplay() {
  if (showingShaderCode && renderer) {
    const shaderCode = renderer.get_current_shader();
    document.getElementById("shader-code").textContent = shaderCode;
  }
}

function updateShaderDisplay2() {
  if (showingShaderCode && renderer) {
    const shaderCode = renderer.get_current_grammar();
    document.getElementById("shader-code").textContent = shaderCode;
  }
}

async function reloadShader() {
  if (!renderer) return;

  try {
    await renderer.reload_shader();
    showStatus("✅ Shader reloaded successfully!");
    updateShaderDisplay();
  } catch (error) {
    console.error("Failed to reload shader:", error);
    showStatus("❌ Failed to reload shader: " + error.toString(), true);
  }
}

function toggleRendering() {
  if (!renderer) return;

  const toggleBtn = document.getElementById("toggle-btn");

  if (isRunning) {
    renderer.stop_rendering();
    toggleBtn.textContent = "▶️ Resume";
    showStatus("⏸️ Rendering paused");
  } else {
    renderer.start_rendering();
    toggleBtn.textContent = "⏸️ Pause";
    showStatus("▶️ Rendering resumed");
  }

  isRunning = !isRunning;
}

function toggleShaderInfo() {
  const shaderDisplay = document.getElementById("shader-display");
  const btn = document.getElementById("shader-info-btn");

  showingShaderCode = !showingShaderCode;

  if (showingShaderCode) {
    shaderDisplay.style.display = "block";
    btn.textContent = "🙈 Hide Shader Code";
    updateShaderDisplay();
  } else {
    shaderDisplay.style.display = "none";
    btn.textContent = "📋 Show Shader Code";
  }
}

function toggleShaderInfo2() {
  const shaderDisplay = document.getElementById("shader-display");
  const btn = document.getElementById("shader-info-btn2");

  showingShaderCode = !showingShaderCode;

  if (showingShaderCode) {
    shaderDisplay.style.display = "block";
    btn.textContent = "🙈 Hide Shader Grammar";
    updateShaderDisplay2();
  } else {
    shaderDisplay.style.display = "none";
    btn.textContent = "📋 Show Shader Grammar";
  }
}

async function initializeApp() {
  try {
    // Initialize the wasm module
    await init();

    // Create the renderer
    renderer = new ShaderRenderer("shader_canvas");

    // Start rendering
    renderer.start_rendering();

    showStatus("🚀 Shader renderer initialized!");

    // Hide the click hint after a few seconds
    setTimeout(() => {
      const hint = document.querySelector(".click-hint");
      if (hint) {
        hint.style.opacity = "0";
        setTimeout(() => hint.remove(), 500);
      }
    }, 3000);
  } catch (error) {
    console.error("Failed to initialize:", error);
    showStatus("❌ Failed to initialize: " + error.toString(), true);
  }
}

// Event listeners
document.addEventListener("DOMContentLoaded", initializeApp);

// Canvas click handler
document.getElementById("shader_canvas").addEventListener("click", (e) => {
  e.preventDefault();
  reloadShader();
});

// Button handlers
document.getElementById("reload-btn").addEventListener("click", reloadShader);
document.getElementById("toggle-btn").addEventListener(
  "click",
  toggleRendering,
);
document.getElementById("shader-info-btn").addEventListener(
  "click",
  toggleShaderInfo,
);
document.getElementById("shader-info-btn2").addEventListener(
  "click",
  toggleShaderInfo2,
);
document.getElementById("shader-download-btn").addEventListener(
  "click",
  function () {
    // Create a new shader source and download it
    const shaderSource = renderer.get_current_shader();
    const blob = new Blob([shaderSource], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `shader-${Date.now()}.glsl`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    showStatus("📥 Shader source downloaded!");
  },
);

// Page reload handler
globalThis.window.addEventListener("beforeunload", () => {
  if (renderer) {
    renderer.stop_rendering();
  }
});

// Handle page visibility changes
document.addEventListener("visibilitychange", () => {
  if (!renderer) return;

  if (document.hidden) {
    renderer.stop_rendering();
  } else if (isRunning) {
    renderer.start_rendering();
  }
});
