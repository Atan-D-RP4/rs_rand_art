use js_sys::Date;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    Blob, BlobPropertyBag, Document, HtmlCanvasElement, HtmlElement, HtmlTextAreaElement, Url,
    WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlVertexArrayObject, Window,
};

use crate::bnf_parser::Parser;
use crate::grammar::Grammar;

const FRAGMENT_SHADER_TEMPLATE: &str = r"#version 300 es
precision mediump float;
in vec2 fragTexCoord;
out vec4 finalColor;
uniform float time;

vec4 map_rgb(vec3 rgb) {
    return vec4(rgb + 0.5, 1);
}

void main() {
    float x = fragTexCoord.x;
    float y = fragTexCoord.y;
    float t = tan(time);
    finalColor = map_rgb(%s);
}
";

// Hardcoded vertex shader
const VERTEX_SHADER_SOURCE: &str = r"#version 300 es
precision mediump float;
in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;
uniform mat4 mvp;

void main()
{
    fragTexCoord = vertexTexCoord;
    fragColor = vertexColor;
    gl_Position = mvp*vec4(vertexPosition, 1.0);
}
";

#[wasm_bindgen]
pub struct ShaderRenderer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vao: WebGlVertexArrayObject,
    animation_frame_id: Rc<RefCell<Option<i32>>>,
    source: String,
    grammar: String,
}

#[wasm_bindgen]
impl ShaderRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<ShaderRenderer, JsValue> {
        let window = web_sys::window().ok_or("Failed to create Window")?;
        let document = window
            .document()
            .ok_or("should have a document on window")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("no canvas found")?;
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
        let gl = canvas
            .get_context("webgl2")?
            .ok_or("failed to get WebGL2 context")?
            .dyn_into::<WebGl2RenderingContext>()?;

        // Generate initial fragment shader
        let fragment_shader_source = generate_fragment_shader("")?;

        // Compile shaders
        let vertex_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            VERTEX_SHADER_SOURCE,
        )?;

        let fragment_shader = compile_shader(
            &gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            &fragment_shader_source,
        )?;

        let program = link_program(&gl, &vertex_shader, &fragment_shader)?;
        gl.delete_shader(Some(&vertex_shader));
        gl.delete_shader(Some(&fragment_shader));

        let vao = setup_buffers(&gl);
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        Ok(ShaderRenderer {
            gl,
            program,
            vao,
            animation_frame_id: Rc::new(RefCell::new(None)),
            source: fragment_shader_source,
            grammar: Grammar::default().to_string(),
        })
    }

    #[wasm_bindgen]
    pub fn start_rendering(&mut self) -> Result<(), JsValue> {
        self.stop_rendering()?; // Stop any existing animation loop

        let gl = self.gl.clone();
        let program = self.program.clone();
        let vao = self.vao.clone();
        let animation_frame_id = self.animation_frame_id.clone();

        let f = Rc::new(RefCell::new(
            None::<Closure<dyn std::ops::FnMut() -> Result<(), JsValue>>>,
        ));
        let g = f.clone();

        let initial_time = Date::now() / 1000.0;

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            gl.use_program(Some(&program));

            let mut current_time = Date::now() / 1000.0;
            current_time -= initial_time;

            let time_location = gl.get_uniform_location(&program, "time");
            gl.uniform1f(time_location.as_ref(), current_time as f32);

            let mvp_location = gl.get_uniform_location(&program, "mvp");
            let identity: [f32; 16] = [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ];
            gl.uniform_matrix4fv_with_f32_array(mvp_location.as_ref(), false, &identity);

            gl.clear_color(0.9, 0.3, 0.6, 1.0);
            gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            gl.bind_vertex_array(Some(&vao));
            gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                6,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );

            let window =
                web_sys::window().ok_or_else(|| JsValue::from_str("Failed to get window"))?;
            let animation_id = window
                .request_animation_frame(
                    f.borrow()
                        .as_ref()
                        .ok_or_else(|| JsValue::from_str("Animation frame closure not available"))?
                        .as_ref()
                        .unchecked_ref(),
                )
                .map_err(|_| JsValue::from_str("Failed to request animation frame"))?;

            animation_frame_id.replace(Some(animation_id));
            Ok(())
        })
            as Box<dyn FnMut() -> Result<(), JsValue>>));

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("Failed to get window"))?;
        let animation_id = window
            .request_animation_frame(
                g.borrow()
                    .as_ref()
                    .ok_or_else(|| JsValue::from_str("Animation frame closure not available"))?
                    .as_ref()
                    .unchecked_ref(),
            )
            .map_err(|_| JsValue::from_str("Failed to request animation frame"))?;

        self.animation_frame_id.replace(Some(animation_id));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn stop_rendering(&mut self) -> Result<(), JsValue> {
        if let Some(id) = self.animation_frame_id.take() {
            web_sys::window()
                .ok_or("Failed to get window")?
                .cancel_animation_frame(id)
                .map_err(|_| JsValue::from_str("Failed to cancel animation frame"))
        } else {
            Ok(())
        }
    }

    #[wasm_bindgen]
    pub fn reload_shader(&mut self) -> Result<(), JsValue> {
        self.stop_rendering()?;

        // Generate new fragment shader
        let fragment_shader_source =
            generate_fragment_shader(&self.grammar).map_err(|e| JsValue::from_str(&e))?;

        web_sys::console::log_1(&format!("New shader: {fragment_shader_source}").into());

        // Compile new shaders
        let vertex_shader = compile_shader(
            &self.gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            VERTEX_SHADER_SOURCE,
        )?;

        let fragment_shader = compile_shader(
            &self.gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            &fragment_shader_source,
        )?;

        // Delete old program
        self.gl.delete_program(Some(&self.program));

        // Link new program
        self.program = link_program(&self.gl, &vertex_shader, &fragment_shader)?;

        // Clean up shaders (they're no longer needed after linking)
        self.gl.delete_shader(Some(&vertex_shader));
        self.gl.delete_shader(Some(&fragment_shader));

        // Restart rendering with new shader
        self.source = fragment_shader_source;
        self.start_rendering()?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn reload_grammar(&mut self, new_grammar: &str) -> Result<(), JsValue> {
        // Update the grammar and reload the shader
        let grammar = Parser::new(new_grammar).parse();
        match grammar {
            Ok(g) => self.grammar = g.to_string(),
            Err(e) => {
                web_sys::console::error_1(
                    &format!("{new_grammar} is not a valid grammar: {e:?}").into(),
                );
                return Err(JsValue::from_str(&format!(
                    "Failed to parse grammar: {e:?}\nProvided Grammar: {new_grammar}",
                )));
            }
        }
        web_sys::console::log_1(&format!("New grammar: {}", self.grammar).into());
        self.reload_shader()
    }

    #[wasm_bindgen]
    pub fn get_current_shader(&self) -> String {
        web_sys::console::log_1(
            &format!("Current shader source: {}", self.get_current_grammar()).into(),
        );
        self.source.clone()
    }

    #[wasm_bindgen]
    pub fn get_current_grammar(&self) -> String {
        self.grammar.clone()
    }
}

#[allow(clippy::similar_names)]
fn setup_buffers(gl: &WebGl2RenderingContext) -> WebGlVertexArrayObject {
    let vertices: [f32; 20] = [
        1.0, 1.0, 0.0, 1.0, 1.0, // top right
        1.0, -1.0, 0.0, 1.0, -1.0, // bottom right
        -1.0, -1.0, 0.0, -1.0, -1.0, // bottom left
        -1.0, 1.0, 0.0, -1.0, 1.0, // top left
    ];
    let indices: [u16; 6] = [0, 1, 3, 1, 2, 3];

    let vao = gl.create_vertex_array().unwrap(); // VAO
    gl.bind_vertex_array(Some(&vao));

    let vbo = gl.create_buffer().unwrap(); // VBO
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let ebo = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ebo));
    unsafe {
        let index_array = js_sys::Uint16Array::view(&indices);
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    // Position attribute
    gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 5 * 4, 0);
    gl.enable_vertex_attrib_array(0);

    // Texture coord attribute
    gl.vertex_attrib_pointer_with_i32(1, 2, WebGl2RenderingContext::FLOAT, false, 5 * 4, 3 * 4);
    gl.enable_vertex_attrib_array(1);

    vao
}

// Compile a shader
fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("unable to create shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .is_falsy()
    {
        let log = gl.get_shader_info_log(&shader).unwrap_or_default();
        gl.delete_shader(Some(&shader));
        return Err(JsValue::from_str(&format!(
            "Shader compilation failed: {log}"
        )));
    }
    Ok(shader)
}

// Link a shader program
fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, JsValue> {
    let program = gl
        .create_program()
        .ok_or_else(|| JsValue::from_str("unable to create program"))?;
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .is_falsy()
    {
        let log = gl.get_program_info_log(&program).unwrap_or_default();
        gl.delete_program(Some(&program));
        return Err(JsValue::from_str(&format!("Program linking failed: {log}")));
    }
    Ok(program)
}

#[wasm_bindgen]
pub fn generate_fragment_shader(inp: &str) -> Result<String, String> {
    let grammar = if inp.is_empty() {
        // Use a default grammar if no input is provided
        Grammar::default()
    } else {
        let mut parser = Parser::new(inp);
        parser
            .parse()
            .map_err(|e| format!("Failed to parse grammar: {e:?}\nGrammar:\n{inp}"))?
    };

    let mut func = grammar
        .gen_from_rule(0, 10)
        .ok_or("Failed to generate function".to_string())?;

    func.compile_to_glsl_fs(FRAGMENT_SHADER_TEMPLATE)
        .map_err(|e| format!("Failed to compile function to GLSL: {e:?}"))
}

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

            self.document.body().unwrap().append_child(&link)?;
            link.click();
            self.document.body().unwrap().remove_child(&link)?;
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
