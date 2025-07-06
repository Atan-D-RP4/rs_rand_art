import init, { create_shader_app } from "./pkg/shaderand.js";

async function run() {
	await init();

	try {
		const app = create_shader_app();

		// Initialize the renderer with the canvas ID
		app.initialize("shader_canvas");

		// Set up event listeners
		const canvas = document.getElementById("shader_canvas");
		canvas.addEventListener("click", () => app.handle_canvas_click());

		document.getElementById("toggle-btn").addEventListener(
			"click",
			() => app.toggle_rendering(),
		);
		document.getElementById("shader-info-btn").addEventListener(
			"click",
			() => app.toggle_shader_info(),
		);
		document.getElementById("grammar-info-btn").addEventListener(
			"click",
			() => app.toggle_grammar_info(),
		);
		document.getElementById("grammar-edit-btn").addEventListener(
			"click",
			() => app.show_grammar_editor(),
		);
		document.getElementById("apply-grammar-btn").addEventListener(
			"click",
			() => app.apply_grammar(),
		);
		document.getElementById("cancel-grammar-btn").addEventListener(
			"click",
			() => app.cancel_grammar_edit(),
		);
		document.getElementById("download-shader-btn").addEventListener(
			"click",
			() => app.download_shader(),
		);

		// Handle visibility changes
		document.addEventListener(
			"visibilitychange",
			() => app.handle_visibility_change(),
		);

		// Clean up on window unload
		this.window.addEventListener("unload", () => app.cleanup());
	} catch (error) {
		console.error("Error initializing shader app:", error);
	}
}

run();

