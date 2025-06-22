import init, { ShaderRenderer } from "./pkg/shaderand.js";

/**
 * Enhanced Shader Renderer Application
 * Manages WebAssembly-based shader rendering with interactive controls
 */
class ShaderApp {
	constructor() {
		this.renderer = null;
		this.isRunning = true;
		this.showingShaderCode = false;
		this.displayMode = "shader"; // 'shader' or 'grammar'

		// DOM elements cache
		this.elements = {};

		// Status display configuration
		this.statusConfig = {
			displayDuration: 3000,
			fadeOutDuration: 500,
		};

		// Bind methods to preserve context
		this.handleCanvasClick = this.handleCanvasClick.bind(this);
		this.handleVisibilityChange = this.handleVisibilityChange.bind(this);
		this.handleBeforeUnload = this.handleBeforeUnload.bind(this);
	}

	/**
	 * Cache frequently accessed DOM elements
	 */
	cacheElements() {
		const elementIds = [
			"status",
			"shader-code",
			"shader-display",
			"shader_canvas",
			"toggle-btn",
			"shader-info-btn",
			"shader-info-btn2",
			"reload-btn",
			"shader-download-btn",
		];

		elementIds.forEach((id) => {
			this.elements[id] = document.getElementById(id);
			if (!this.elements[id]) {
				console.warn(`Element with id '${id}' not found`);
			}
		});
	}

	/**
	 * Display status message with optional error styling
	 * @param {string} message - Status message to display
	 * @param {boolean} isError - Whether this is an error message
	 */
	showStatus(message, isError = false) {
		const statusEl = this.elements.status;
		if (!statusEl) return;

		statusEl.textContent = message;
		statusEl.className = `status ${isError ? "error" : "success"}`;
		statusEl.style.display = "block";
		statusEl.style.opacity = "1";

		// Clear any existing timeout
		if (this.statusTimeout) {
			clearTimeout(this.statusTimeout);
		}

		// Auto-hide status after configured duration
		this.statusTimeout = setTimeout(() => {
			statusEl.style.opacity = "0";
			setTimeout(() => {
				statusEl.style.display = "none";
			}, this.statusConfig.fadeOutDuration);
		}, this.statusConfig.displayDuration);
	}

	/**
	 * Update shader code display based on current mode
	 */
	updateShaderDisplay() {
		if (!this.showingShaderCode || !this.renderer) return;

		try {
			const content = this.displayMode === "shader"
				? this.renderer.get_current_shader()
				: this.renderer.get_current_grammar();

			if (this.elements["shader-code"]) {
				this.elements["shader-code"].textContent = content;
			}
		} catch (error) {
			console.error("Failed to update shader display:", error);
			this.showStatus("❌ Failed to update display", true);
		}
	}

	/**
	 * Reload the current shader
	 */
	async reloadShader() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		try {
			await this.renderer.reload_shader();
			this.showStatus("✅ Shader reloaded successfully!");
			this.updateShaderDisplay();
		} catch (error) {
			console.error("Failed to reload shader:", error);
			this.showStatus(
				`❌ Failed to reload shader: ${error.message || error}`,
				true,
			);
		}
	}

	/**
	 * Toggle rendering state (play/pause)
	 */
	toggleRendering() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		const toggleBtn = this.elements["toggle-btn"];
		if (!toggleBtn) return;

		try {
			if (this.isRunning) {
				this.renderer.stop_rendering();
				toggleBtn.textContent = "▶️ Resume";
				this.showStatus("⏸️ Rendering paused");
			} else {
				this.renderer.start_rendering();
				toggleBtn.textContent = "⏸️ Pause";
				this.showStatus("▶️ Rendering resumed");
			}

			this.isRunning = !this.isRunning;
		} catch (error) {
			console.error("Failed to toggle rendering:", error);
			this.showStatus("❌ Failed to toggle rendering", true);
		}
	}

	/**
	 * Toggle shader information display
	 * @param {string} mode - Display mode: 'shader' or 'grammar'
	 */
	toggleShaderInfo(mode = "shader") {
		const shaderDisplay = this.elements["shader-display"];
		const btnKey = mode === "shader"
			? "shader-info-btn"
			: "shader-info-btn2";
		const btn = this.elements[btnKey];

		if (!shaderDisplay || !btn) return;

		this.displayMode = mode;
		this.showingShaderCode = !this.showingShaderCode;

		if (this.showingShaderCode) {
			shaderDisplay.style.display = "block";
			btn.textContent = mode === "shader"
				? "🙈 Hide Shader Code"
				: "🙈 Hide Shader Grammar";
			this.updateShaderDisplay();
		} else {
			shaderDisplay.style.display = "none";
			btn.textContent = mode === "shader"
				? "📋 Show Shader Code"
				: "📋 Show Shader Grammar";
		}
	}

	/**
	 * Download current shader source code
	 */
	downloadShader() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		try {
			const shaderSource = this.renderer.get_current_shader();
			const blob = new Blob([shaderSource], { type: "text/plain" });
			const url = URL.createObjectURL(blob);

			const link = document.createElement("a");
			link.href = url;
			link.download = `shader-${Date.now()}.glsl`;
			link.style.display = "none";

			document.body.appendChild(link);
			link.click();
			document.body.removeChild(link);

			URL.revokeObjectURL(url);
			this.showStatus("📥 Shader source downloaded!");
		} catch (error) {
			console.error("Failed to download shader:", error);
			this.showStatus("❌ Failed to download shader", true);
		}
	}

	/**
	 * Handle canvas click events
	 * @param {Event} event - Click event
	 */
	handleCanvasClick(event) {
		event.preventDefault();
		this.reloadShader();
	}

	/**
	 * Handle page visibility changes to optimize performance
	 */
	handleVisibilityChange() {
		if (!this.renderer) return;

		try {
			if (document.hidden) {
				this.renderer.stop_rendering();
			} else if (this.isRunning) {
				this.renderer.start_rendering();
			}
		} catch (error) {
			console.error("Failed to handle visibility change:", error);
		}
	}

	/**
	 * Handle page unload to cleanup resources
	 */
	handleBeforeUnload() {
		this.cleanup();
	}

	/**
	 * Clean up resources
	 */
	cleanup() {
		if (this.renderer) {
			try {
				this.renderer.stop_rendering();
			} catch (error) {
				console.error(
					"Failed to stop rendering during cleanup:",
					error,
				);
			}
		}

		if (this.statusTimeout) {
			clearTimeout(this.statusTimeout);
		}
	}

	/**
	 * Hide the initial click hint with smooth animation
	 */
	hideClickHint() {
		const hint = document.querySelector(".click-hint");
		if (!hint) return;

		hint.style.transition = "opacity 0.5s ease-out";
		hint.style.opacity = "0";

		setTimeout(() => {
			if (hint.parentNode) {
				hint.parentNode.removeChild(hint);
			}
		}, 500);
	}

	/**
	 * Set up all event listeners
	 */
	setupEventListeners() {
		// Canvas interaction
		const canvas = this.elements.shader_canvas;
		if (canvas) {
			canvas.addEventListener("click", this.handleCanvasClick);
		}

		// Control buttons
		const buttonHandlers = {
			"reload-btn": () => this.reloadShader(),
			"toggle-btn": () => this.toggleRendering(),
			"shader-info-btn": () => this.toggleShaderInfo("shader"),
			"shader-info-btn2": () => this.toggleShaderInfo("grammar"),
			"shader-download-btn": () => this.downloadShader(),
		};

		Object.entries(buttonHandlers).forEach(([btnId, handler]) => {
			const btn = this.elements[btnId];
			if (btn) {
				btn.addEventListener("click", handler);
			}
		});

		// Window events
		window.addEventListener("beforeunload", this.handleBeforeUnload);
		document.addEventListener(
			"visibilitychange",
			this.handleVisibilityChange,
		);
	}

	/**
	 * Initialize the shader renderer application
	 */
	async initialize() {
		try {
			// Cache DOM elements first
			this.cacheElements();

			// Initialize WebAssembly module
			await init();

			// Create renderer instance
			this.renderer = new ShaderRenderer("shader_canvas");

			// Start rendering
			this.renderer.start_rendering();

			// Set up event listeners
			this.setupEventListeners();

			this.showStatus("🚀 Shader renderer initialized!");

			// Hide click hint after initialization
			setTimeout(() => {
				this.hideClickHint();
			}, 3000);
		} catch (error) {
			console.error("Failed to initialize shader app:", error);
			this.showStatus(
				`❌ Failed to initialize: ${error.message || error}`,
				true,
			);
		}
	}
}

// Application instance
const shaderApp = new ShaderApp();

// Initialize when DOM is ready
if (document.readyState === "loading") {
	document.addEventListener("DOMContentLoaded", () => shaderApp.initialize());
} else {
	// DOM already loaded
	shaderApp.initialize();
}
