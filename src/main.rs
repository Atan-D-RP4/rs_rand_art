use shaderand::bnf_parser;
mod grammar;
mod node;

use gl::types::{GLchar, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint};
use glfw::{Action, Context, Key, Modifiers};
extern crate gl;
extern crate glfw;

use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::str;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTEX_SHADER_SOURCE: &str = r"
#version 330

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

fn compile_shader(source: &str, shader_type: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for shader compile errors
        let mut success = GLint::from(gl::FALSE);
        let mut info_log = vec![0; 512];
        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != GLint::from(gl::TRUE) {
            gl::GetShaderInfoLog(
                shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr().cast::<GLchar>(),
            );
            eprintln!("Shader source: {source:?}");
            eprintln!(
                "ERROR::SHADER::COMPILATION_FAILED\n{}",
                str::from_utf8(&info_log).unwrap()
            );
        }
        shader
    }
}

fn get_random_fs() -> Result<String, String> {
    use crate::grammar::Grammar;
    let inp = "./grammar.bnf";
    let inp = std::fs::read_to_string(inp).map_err(|e| e.to_string())?;
    let mut parser = bnf_parser::Parser::new(&inp);

    let grammar = match parser.parse() {
        Ok(grammar) => grammar,
        Err(e) => {
            return Err(format!("Error parsing BNF: {e:?}"));
        }
    };
    // let grammar = Grammar::default();
    println!("Grammar:");
    println!("{grammar}");

    let Some(mut func) = grammar.gen_from_rule(0, 10) else {
        return Err("Failed to generate function".to_string());
    };
    // println!("Function:");
    // println!("{func}");
    // func.optimize()?;
    // println!("Optimized Function:");
    // println!("{func}");
    let template_fs = String::from(
        r"#version 330

          in vec2 fragTexCoord;
          out vec4 finalColor;
          uniform float time;

          vec4 map_rgb(vec3 rgb) {
              return vec4(rgb + 1/2, 1);
          }

          void main() {
              float x = fragTexCoord.x;
              float y = fragTexCoord.y;
              float t = tan(time);
              finalColor = map_rgb(%s);
          }
        ",
    );
    func.compile_to_glsl_fs(&template_fs)
}

#[allow(non_snake_case)]
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), String> {
    use glfw::fail_on_errors;

    let mut glfw = glfw::init(fail_on_errors!()).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 5));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(
            SCR_WIDTH,
            SCR_HEIGHT,
            "Randow Shader",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    gl::load_with(|symbol| -> *const std::ffi::c_void { window.get_proc_address(symbol).cast() });

    let (shader_program, vao) = unsafe {
        let vertex_shader = compile_shader(VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER);
        let FRAGMENT_SHADER_SOURCE = &get_random_fs()?;
        // println!("{}", FRAGMENT_SHADER_SOURCE);
        let fragment_shader = compile_shader(FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER);
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check for linking errors
        let mut success = GLint::from(gl::FALSE);
        let mut info_log = vec![0; 512];
        info_log.set_len(512 - 1);
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != GLint::from(gl::TRUE) {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr().cast::<GLchar>(),
            );
            println!(
                "ERROR::SHADER::PROGRAM::LINKING_FAILED\n{}",
                str::from_utf8(&info_log).unwrap()
            );
        }

        // Clean up shaders
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // Set up vertex data
        let vertices: [f32; 20] = [
            // positions     // texture coords  // colors
            1.0, 1.0, 0.0, 1.0, 1.0, // top right
            1.0, -1.0, 0.0, 1.0, -1.0, // bottom right
            -1.0, -1.0, 0.0, -1.0, -1.0, // bottom left
            -1.0, 1.0, 0.0, -1.0, 1.0, // top left
        ];
        let indices = [
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];

        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            GLsizeiptr::try_from(vertices.len() * mem::size_of::<GLfloat>()).unwrap(),
            vertices.as_ptr().cast::<c_void>(),
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            GLsizeiptr::try_from(indices.len() * mem::size_of::<GLuint>()).unwrap(),
            indices.as_ptr().cast::<c_void>(),
            gl::STATIC_DRAW,
        );

        // Position attribute
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            5 * GLsizei::try_from(mem::size_of::<GLfloat>()).unwrap(),
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Texture coord attribute
        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            5 * GLsizei::try_from(mem::size_of::<GLfloat>()).unwrap(),
            (3 * mem::size_of::<GLfloat>()) as *const c_void,
        );
        gl::EnableVertexAttribArray(1);
        (shader_program, vao)
    };
    // Get the location of the time uniform
    let time_location = unsafe {
        let time = CString::new("time").unwrap();
        gl::GetUniformLocation(shader_program, time.as_ptr())
    };

    // Store the initial time
    let initial_time = glfw.get_time();

    // Loop until the user closes the window
    while !window.should_close() {
        // Swap front and back buffers
        glfw.poll_events();
        window.swap_buffers();
        // Clear the screen
        unsafe {
            gl::ClearColor(0.9, 0.3, 0.6, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Render
            gl::UseProgram(shader_program);

            // Update time uniform
            let current_time = glfw.get_time() - initial_time;
            gl::Uniform1f(time_location, current_time as f32);

            // Set MVP matrix (identity for now)
            let mvp_location = {
                let mvp = CString::new("mvp").unwrap();
                gl::GetUniformLocation(shader_program, mvp.as_ptr())
            };
            let identity: [f32; 16] = [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ];
            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, identity.as_ptr());

            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        // Poll for and process events
        // event_handler
        event_handler(&mut glfw, &mut window, &events);
    }

    Ok(())
}

fn event_handler(
    glfw: &mut glfw::Glfw,
    window: &mut glfw::Window,
    events: &glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
) {
    for (_, event) in glfw::flush_messages(events) {
        println!("{event:?}");
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                window.restore();
                println!("restored");
            }
            glfw::WindowEvent::Key(Key::Enter, _, Action::Press, Modifiers::Control) => {
                if window.with_window_mode(|mode| matches!(mode, glfw::WindowMode::Windowed)) {
                    glfw.with_primary_monitor(|_, monitor| {
                        let monitor = monitor.unwrap();
                        window.set_monitor(
                            glfw::WindowMode::FullScreen(monitor),
                            0,
                            0,
                            SCR_WIDTH,
                            SCR_HEIGHT,
                            Some(60),
                        );
                    });
                } else {
                    window.set_monitor(
                        glfw::WindowMode::Windowed,
                        0,
                        0,
                        SCR_WIDTH,
                        SCR_HEIGHT,
                        Some(60),
                    );
                }
            }
            glfw::WindowEvent::Key(Key::F, _, Action::Press, _) => {
                // println!("{:?}", modifier);
                if window.with_window_mode(|mode| matches!(mode, glfw::WindowMode::Windowed)) {
                    if window.is_maximized() {
                        window.restore();
                    } else {
                        window.maximize();
                    }
                }
            }
            glfw::WindowEvent::Key(Key::M, _, Action::Press, _) => {
                if window.is_iconified() {
                    window.restore();
                } else {
                    window.iconify();
                }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true);
            }
            _ => {}
        }
    }
}
