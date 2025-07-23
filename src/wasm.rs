use crate::renderer::ShaderRenderer;
use js_sys::Date;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, Document, HtmlElement, HtmlTextAreaElement, Url, Window};

#[wasm_bindgen]
pub struct ShaderApp {
    renderer: Option<ShaderRenderer>,
    is_running: bool,
    showing_shader_code: bool,
    showing_grammar_info: bool,
    showing_grammar_editor: bool,
    document: Document,
    window: Window,
    status_timeout: Option<i32>,
}

#[wasm_bindgen]
impl ShaderApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ShaderApp, JsValue> {
        let window = web_sys::window().ok_or("Failed to get window")?;
        let document = window.document().ok_or("Failed to get document")?;

        Ok(ShaderApp {
            renderer: None,
            is_running: true,
            showing_shader_code: false,
            showing_grammar_info: false,
            showing_grammar_editor: false,
            document,
            window,
            status_timeout: None,
        })
    }

    #[wasm_bindgen]
    pub fn initialize(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let mut renderer = ShaderRenderer::new(canvas_id)?;
        renderer.start_rendering()?;
        self.renderer = Some(renderer);
        self.show_status("🚀 Shader renderer initialized!", false)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn show_status(&mut self, message: &str, is_error: bool) -> Result<(), JsValue> {
        let status_el = self.document.get_element_by_id("status");
        if let Some(status) = status_el {
            status.set_text_content(Some(message));
            let class_name = if is_error {
                "status error"
            } else {
                "status success"
            };
            status.set_class_name(class_name);
            let status = status.dyn_into::<HtmlElement>()?;

            let style = status.style();
            style.set_property("display", "block")?;
            style.set_property("opacity", "1")?;

            // Clear existing timeout
            if let Some(timeout_id) = self.status_timeout {
                self.window.clear_timeout_with_handle(timeout_id);
            }

            // Set new timeout
            let window = self.window.clone();
            let timeout_callback = Closure::wrap(Box::new(move || {
                let style = status.style();
                let _ = style.set_property("opacity", "0");

                let status_for_hide = status.clone();
                let hide_callback = Closure::wrap(Box::new(move || {
                    let _ = status_for_hide.style().set_property("display", "none");
                }) as Box<dyn FnMut()>);

                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    hide_callback.as_ref().unchecked_ref(),
                    500,
                );
                hide_callback.forget();
            }) as Box<dyn FnMut()>);

            self.status_timeout = Some(
                self.window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        timeout_callback.as_ref().unchecked_ref(),
                        3000,
                    )?,
            );
            timeout_callback.forget();
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reload_shader(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            renderer.reload_shader()?;
            self.show_status("✅ Shader reloaded successfully!", false)?;
            self.update_shader_display()?;
            self.update_grammar_display()?;
        } else {
            self.show_status("❌ Renderer not initialized", true)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn toggle_rendering(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            let toggle_btn = self.document.get_element_by_id("toggle-btn");

            if self.is_running {
                renderer.stop_rendering()?;
                if let Some(btn) = toggle_btn {
                    btn.set_text_content(Some("▶️ Resume"));
                }
                self.show_status("⏸️ Rendering paused", false)?;
            } else {
                renderer.start_rendering()?;
                if let Some(btn) = toggle_btn {
                    btn.set_text_content(Some("⏸️ Pause"));
                }
                self.show_status("▶️ Rendering resumed", false)?;
            }
            self.is_running = !self.is_running;
        } else {
            self.show_status("❌ Renderer not initialized", true)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn toggle_shader_info(&mut self) -> Result<(), JsValue> {
        let shader_display = self.document.get_element_by_id("shader-display");
        let btn = self.document.get_element_by_id("shader-info-btn");

        if let (Some(display), Some(button)) = (shader_display, btn) {
            self.showing_shader_code = !self.showing_shader_code;

            if self.showing_shader_code {
                if let Ok(html_el) = display.dyn_into::<HtmlElement>() {
                    html_el.style().set_property("display", "block")?;
                }
                button.set_text_content(Some("🙈 Hide Shader Code"));
                self.update_shader_display()?;
            } else {
                if let Ok(html_el) = display.dyn_into::<HtmlElement>() {
                    html_el.style().set_property("display", "none")?;
                }
                button.set_text_content(Some("📋 Show Shader Code"));
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn toggle_grammar_info(&mut self) -> Result<(), JsValue> {
        let grammar_display = self.document.get_element_by_id("grammar-display");
        let btn = self.document.get_element_by_id("grammar-info-btn");

        if let (Some(display), Some(button)) = (grammar_display, btn) {
            self.showing_grammar_info = !self.showing_grammar_info;

            if self.showing_grammar_info {
                if let Ok(html_el) = display.dyn_into::<HtmlElement>() {
                    html_el.style().set_property("display", "block")?;
                }
                button.set_text_content(Some("🙈 Hide Grammar"));
                self.update_grammar_display()?;
            } else {
                if let Ok(html_el) = display.dyn_into::<HtmlElement>() {
                    html_el.style().set_property("display", "none")?;
                }
                button.set_text_content(Some("📋 Show Grammar"));
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn show_grammar_editor(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &self.renderer {
            let grammar_textarea = self.document.get_element_by_id("grammar-textarea");
            let grammar_editor = self.document.get_element_by_id("grammar-editor");
            let edit_btn = self.document.get_element_by_id("grammar-edit-btn");

            if let (Some(textarea), Some(editor), Some(btn)) =
                (grammar_textarea, grammar_editor, edit_btn)
            {
                if self.showing_grammar_editor {
                    if let Ok(html_el) = editor.dyn_into::<HtmlElement>() {
                        html_el.style().set_property("display", "none")?;
                    }
                    btn.set_text_content(Some("✏️ Edit Grammar"));
                    self.showing_grammar_editor = false;
                } else {
                    let comments = "# Due to some implementation details, the first rule of the grammar must be a vec3().\n# The First rule also cannot be referenced by any other rule.\n\n";
                    let current_grammar = format!("{}{}", comments, renderer.get_current_grammar());

                    if let Ok(textarea_el) = textarea.clone().dyn_into::<HtmlTextAreaElement>() {
                        textarea_el.set_value(&current_grammar);
                    }

                    if let Ok(html_el) = editor.dyn_into::<HtmlElement>() {
                        html_el.style().set_property("display", "block")?;
                    }
                    btn.set_text_content(Some("🙈 Hide Grammar Editor"));
                    self.showing_grammar_editor = true;

                    if let Ok(textarea_el) = textarea.dyn_into::<HtmlElement>() {
                        textarea_el.focus()?;
                    }
                }
            }
        } else {
            self.show_status("❌ Renderer not initialized", true)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn apply_grammar(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            let grammar_textarea = self.document.get_element_by_id("grammar-textarea");

            if let Some(textarea) = grammar_textarea {
                if let Ok(textarea_el) = textarea.dyn_into::<HtmlTextAreaElement>() {
                    let new_grammar = textarea_el.value();
                    renderer.reload_grammar(&new_grammar)?;
                    self.show_status("✅ Grammar applied successfully!", false)?;
                    self.update_shader_display()?;
                    self.update_grammar_display()?;
                    self.hide_grammar_editor()?;
                }
            }
        } else {
            self.show_status("❌ Renderer not initialized", true)?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn cancel_grammar_edit(&mut self) -> Result<(), JsValue> {
        self.hide_grammar_editor()?;
        self.show_status("Grammar editing cancelled", false)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn hide_grammar_editor(&mut self) -> Result<(), JsValue> {
        let grammar_editor = self.document.get_element_by_id("grammar-editor");
        let edit_btn = self.document.get_element_by_id("grammar-edit-btn");

        if let (Some(editor), Some(btn)) = (grammar_editor, edit_btn) {
            if let Ok(html_el) = editor.dyn_into::<HtmlElement>() {
                html_el.style().set_property("display", "none")?;
            }
            btn.set_text_content(Some("✏️ Edit Grammar"));
            self.showing_grammar_editor = false;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn download_shader(&self) -> Result<(), JsValue> {
        if let Some(renderer) = &self.renderer {
            let shader_source = renderer.get_current_shader();

            let blob_props = BlobPropertyBag::new();
            blob_props.set_type("text/plain");
            let blob = Blob::new_with_str_sequence_and_options(
                &js_sys::Array::from_iter([JsValue::from_str(&shader_source)]),
                &blob_props,
            )?;

            let url = Url::create_object_url_with_blob(&blob)?;

            let link = self
                .document
                .create_element("a")?
                .dyn_into::<HtmlElement>()?;
            link.set_attribute("href", &url)?;
            link.set_attribute("download", &format!("shader-{}.glsl", Date::now() as i64))?;
            link.style().set_property("display", "none")?;

            // self.document.body().unwrap().append_child(&link)?;
            // link.click();
            if let Some(body) = self.document.body() {
                body.append_child(&link)?;
                link.click();
                body.remove_child(&link)?;
            } else {
                return Err(JsValue::from_str("Failed to get document body"));
            }
            Url::revoke_object_url(&url)?;

            // Note: Can't call show_status here due to &self, would need &mut self
            web_sys::console::log_1(&"📥 Shader source downloaded!".into());
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn update_shader_display(&self) -> Result<(), JsValue> {
        if !self.showing_shader_code {
            return Ok(());
        }

        if let Some(renderer) = &self.renderer {
            let content = renderer.get_current_shader();
            if let Some(shader_code) = self.document.get_element_by_id("shader-code") {
                shader_code.set_text_content(Some(&content));
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn update_grammar_display(&self) -> Result<(), JsValue> {
        if !self.showing_grammar_info {
            return Ok(());
        }

        if let Some(renderer) = &self.renderer {
            let content = renderer.get_current_grammar();
            if let Some(grammar_code) = self.document.get_element_by_id("grammar-code") {
                grammar_code.set_text_content(Some(&content));
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn handle_canvas_click(&mut self) -> Result<(), JsValue> {
        self.reload_shader()
    }

    #[wasm_bindgen]
    pub fn handle_visibility_change(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            if self.document.hidden() {
                renderer.stop_rendering()?;
            } else if self.is_running {
                renderer.start_rendering()?;
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn cleanup(&mut self) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            renderer.stop_rendering()?;
        }

        if let Some(timeout_id) = self.status_timeout {
            self.window.clear_timeout_with_handle(timeout_id);
            self.status_timeout = None;
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub fn create_shader_app() -> Result<ShaderApp, JsValue> {
    ShaderApp::new()
}
