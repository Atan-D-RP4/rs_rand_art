import init, { ShaderRenderer } from "./pkg/shaderand.js";

class ShaderApp {
	constructor() {
		this.renderer = null;
		this.isRunning = true;
		this.showingShaderCode = false;
		this.showingGrammarInfo = false;
		this.showingGrammarEditor = false;
		this.elements = {};
		this.statusTimeout = null;

		// Bind methods to preserve context
		this.handleCanvasClick = this.handleCanvasClick.bind(this);
		this.handleVisibilityChange = this.handleVisibilityChange.bind(this);
		this.handleBeforeUnload = this.handleBeforeUnload.bind(this);
	}

	cacheElements() {
		const elementIds = [
			"status",
			"shader-code",
			"shader-display",
			"shader_canvas",
			"toggle-btn",
			"shader-info-btn",
			"grammar-info-btn",
			"reload-btn",
			"shader-download-btn",
			"grammar-editor",
			"grammar-textarea",
			"apply-grammar-btn",
			"cancel-grammar-btn",
			"grammar-display",
			"grammar-code",
			"grammar-edit-btn",
		];

		elementIds.forEach((id) => {
			this.elements[id] = document.getElementById(id);
			if (!this.elements[id]) {
				console.warn(`Element with id '${id}' not found`);
			}
		});
	}

	showStatus(message, isError = false) {
		const statusEl = this.elements.status;
		if (!statusEl) return;

		statusEl.textContent = message;
		statusEl.className = `status ${isError ? "error" : "success"}`;
		statusEl.style.display = "block";
		statusEl.style.opacity = "1";

		if (this.statusTimeout) {
			clearTimeout(this.statusTimeout);
		}

		this.statusTimeout = setTimeout(() => {
			statusEl.style.opacity = "0";
			setTimeout(() => {
				statusEl.style.display = "none";
			}, 500);
		}, 3000);
	}

	updateShaderDisplay() {
		if (!this.showingShaderCode || !this.renderer) return;

		try {
			const content = this.renderer.get_current_shader();
			if (this.elements["shader-code"]) {
				this.elements["shader-code"].textContent = content;
			}
		} catch (error) {
			console.error("Failed to update shader display:", error);
			this.showStatus("❌ Failed to update shader display", true);
		}
	}

	updateGrammarDisplay() {
		if (!this.showingGrammarInfo || !this.renderer) return;

		try {
			const content = this.renderer.get_current_grammar();
			if (this.elements["grammar-code"]) {
				this.elements["grammar-code"].textContent = content;
			}
		} catch (error) {
			console.error("Failed to update grammar display:", error);
			this.showStatus("❌ Failed to update grammar display", true);
		}
	}

	reloadShader() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		try {
			this.renderer.reload_shader();
			this.showStatus("✅ Shader reloaded successfully!");
			this.updateShaderDisplay();
			this.updateGrammarDisplay();
		} catch (error) {
			console.error("Failed to reload shader:", error);
			this.showStatus(
				`❌ Failed to reload shader: ${error.message || error}`,
				true,
			);
		}
	}

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

	toggleShaderInfo() {
		const shaderDisplay = this.elements["shader-display"];
		const btn = this.elements["shader-info-btn"];
		if (!shaderDisplay || !btn) return;

		this.showingShaderCode = !this.showingShaderCode;
		if (this.showingShaderCode) {
			shaderDisplay.style.display = "block";
			btn.textContent = "🙈 Hide Shader Code";
			this.updateShaderDisplay();
		} else {
			shaderDisplay.style.display = "none";
			btn.textContent = "📋 Show Shader Code";
		}
	}

	toggleGrammarInfo() {
		const grammarDisplay = this.elements["grammar-display"];
		const btn = this.elements["grammar-info-btn"];
		if (!grammarDisplay || !btn) return;

		this.showingGrammarInfo = !this.showingGrammarInfo;
		if (this.showingGrammarInfo) {
			grammarDisplay.style.display = "block";
			btn.textContent = "🙈 Hide Grammar";
			this.updateGrammarDisplay();
		} else {
			grammarDisplay.style.display = "none";
			btn.textContent = "📋 Show Grammar";
		}
	}

	showGrammarEditor() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		try {
			const grammarTextarea = this.elements["grammar-textarea"];
			const grammarEditor = this.elements["grammar-editor"];
			const editBtn = this.elements["grammar-edit-btn"];

			if (!grammarTextarea || !grammarEditor || !editBtn) return;

			if (this.showingGrammarEditor) {
				grammarEditor.style.display = "none";
				editBtn.textContent = "✏️ Edit Grammar";
				this.showingGrammarEditor = false;
			} else {
				const comments =
					"# Due to some implementation details, the first rule of the grammar must be a vec3().\n" +
					"# The First rule also cannot be referenced by any other rule.\n\n";

				const currentGrammar = comments +
					this.renderer.get_current_grammar();
				grammarTextarea.value = currentGrammar;
				grammarEditor.style.display = "block";
				editBtn.textContent = "🙈 Hide Grammar Editor";
				this.showingGrammarEditor = true;
				grammarTextarea.focus();
			}
		} catch (error) {
			console.error("Failed to show grammar editor:", error);
			this.showStatus("❌ Failed to load grammar editor", true);
		}
	}

	applyGrammar() {
		if (!this.renderer) {
			this.showStatus("❌ Renderer not initialized", true);
			return;
		}

		try {
			const grammarTextarea = this.elements["grammar-textarea"];
			if (!grammarTextarea) return;
			const newGrammar = grammarTextarea.value;
			this.renderer.reload_grammar(newGrammar);
			this.showStatus("✅ Grammar applied successfully!");
			this.updateShaderDisplay();
			this.updateGrammarDisplay();
			this.hideGrammarEditor();
		} catch (error) {
			console.error("Failed to apply grammar:", error);
			this.showStatus("❌ Failed to apply grammar", true);
		}
	}

	cancelGrammarEdit() {
		this.hideGrammarEditor();
		this.showStatus("Grammar editing cancelled");
	}

	hideGrammarEditor() {
		const grammarEditor = this.elements["grammar-editor"];
		const editBtn = this.elements["grammar-edit-btn"];

		if (grammarEditor && editBtn) {
			grammarEditor.style.display = "none";
			editBtn.textContent = "✏️ Edit Grammar";
			this.showingGrammarEditor = false;
		}
	}

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

	handleCanvasClick(event) {
		event.preventDefault();
		this.reloadShader();
	}

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

	handleBeforeUnload() {
		this.cleanup();
	}

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
			this.statusTimeout = null;
		}
	}

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

	setupEventListeners() {
		const canvas = this.elements.shader_canvas;
		if (canvas) {
			canvas.addEventListener("click", this.handleCanvasClick);
		}

		const buttonHandlers = {
			"reload-btn": () => this.reloadShader(),
			"toggle-btn": () => this.toggleRendering(),
			"shader-info-btn": () => this.toggleShaderInfo(),
			"grammar-info-btn": () => this.toggleGrammarInfo(),
			"shader-download-btn": () => this.downloadShader(),
			"grammar-edit-btn": () => this.showGrammarEditor(),
			"apply-grammar-btn": () => this.applyGrammar(),
			"cancel-grammar-btn": () => this.cancelGrammarEdit(),
		};

		Object.entries(buttonHandlers).forEach(([btnId, handler]) => {
			const btn = this.elements[btnId];
			if (btn) {
				btn.addEventListener("click", handler);
			}
		});

		window.addEventListener("beforeunload", this.handleBeforeUnload);
		document.addEventListener(
			"visibilitychange",
			this.handleVisibilityChange,
		);
	}

	async initialize() {
		try {
			this.cacheElements();
			await init();
			this.renderer = new ShaderRenderer("shader_canvas");
			this.renderer.start_rendering();
			this.setupEventListeners();
			this.showStatus("🚀 Shader renderer initialized!");

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

const shaderApp = new ShaderApp();

if (document.readyState === "loading") {
	document.addEventListener("DOMContentLoaded", () => shaderApp.initialize());
} else {
	shaderApp.initialize();
}

