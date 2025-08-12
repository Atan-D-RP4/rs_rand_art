use js_sys::Date;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

use crate::bnf_parser::Parser;
use crate::grammar::Grammar;

// WGSL Fragment shader template (converted from GLSL)
const FRAGMENT_SHADER_TEMPLATE: &str = r"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    time: f32,
    _padding: vec3<f32>, // Ensure 16-byte alignment
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn map_rgb(rgb: vec3<f32>) -> vec4<f32> {
    return vec4<f32>(rgb + 0.5, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = in.tex_coords.x;
    let y = in.tex_coords.y;
    let t = tan(uniforms.time);
    return map_rgb(%s);
}
";

// WGSL Vertex shader (converted from GLSL)
const VERTEX_SHADER_SOURCE: &str = r"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    time: f32,
    _padding: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}
";

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x2
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    _padding: [f32; 3], // Ensure 16-byte alignment
}

struct WgpuState<'state> {
    surface: wgpu::Surface<'state>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

#[wasm_bindgen]
pub struct ShaderRenderer {
    state: Option<WgpuState<'static>>,
    canvas: HtmlCanvasElement,
    animation_frame_id: Rc<RefCell<Option<i32>>>,
    source: String,
    grammar: String,
    start_time: f64,
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

        // Generate initial fragment shader
        let fragment_shader_source = generate_fragment_shader("")?;

        Ok(ShaderRenderer {
            state: None,
            canvas,
            animation_frame_id: Rc::new(RefCell::new(None)),
            source: fragment_shader_source,
            grammar: Grammar::default().to_string(),
            start_time: Date::now() / 1000.0,
        })
    }

    #[wasm_bindgen]
    pub async fn initialize(&mut self) -> Result<(), JsValue> {
        // Set up WGPU
        // let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        //     backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
        //     dx12_shader_compiler: Default::default(),
        //     flags: wgpu::InstanceFlags::default(),
        //     gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        // });
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(self.canvas.clone()))
            .map_err(|e| JsValue::from_str(&format!("Failed to create surface: {:?}", e)))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {:?}", e)))?;

        let size = (self.canvas.width(), self.canvas.height());
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create render pipeline
        let render_pipeline = self.create_render_pipeline(&device, &config)?;

        // Create vertex buffer
        let vertices = [
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 1.0],
            }, // top right
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 0.0],
            }, // bottom right
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 0.0],
            }, // bottom left
            Vertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 1.0],
            }, // top left
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let indices: [u16; 6] = [0, 1, 3, 1, 2, 3];
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create uniform buffer
        let uniforms = Uniforms {
            time: 0.0,
            _padding: [0.0; 3],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        self.state = Some(WgpuState {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
        });

        Ok(())
    }

    fn create_render_pipeline(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<wgpu::RenderPipeline, JsValue> {
        // Create shaders
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER_SOURCE.into()),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(self.source.as_str().into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &vertex_shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(render_pipeline)
    }

    fn render(&self) -> Result<(), JsValue> {
        let state = self.state.as_ref().ok_or("WGPU not initialized")?;

        // Update uniform buffer
        let current_time = (Date::now() / 1000.0 - self.start_time) as f32;
        let uniforms = Uniforms {
            time: current_time,
            _padding: [0.0; 3],
        };
        state
            .queue
            .write_buffer(&state.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let output = state
            .surface
            .get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("Failed to get surface texture: {:?}", e)))?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.9,
                            g: 0.3,
                            b: 0.6,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&state.render_pipeline);
            render_pass.set_bind_group(0, &state.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
            render_pass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1);
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    #[wasm_bindgen]
    pub fn start_rendering(&mut self) -> Result<(), JsValue> {
        self.stop_rendering()?; // Stop any existing animation loop

        let animation_frame_id = self.animation_frame_id.clone();

        let f = Rc::new(RefCell::new(
            None::<Closure<dyn std::ops::FnMut() -> Result<(), JsValue>>>,
        ));
        let g = f.clone();

        // Reset start time
        self.start_time = Date::now() / 1000.0;

        *g.borrow_mut() = Some(Closure::wrap(Box::new({
            let renderer_ptr = self as *mut ShaderRenderer;
            move || -> Result<(), JsValue> {
                // Safety: We ensure the renderer lives as long as the animation loop
                let renderer = unsafe { &*renderer_ptr };

                renderer.render()?;

                let window =
                    web_sys::window().ok_or_else(|| JsValue::from_str("Failed to get window"))?;
                let animation_id = window
                    .request_animation_frame(
                        f.borrow()
                            .as_ref()
                            .ok_or_else(|| {
                                JsValue::from_str("Animation frame closure not available")
                            })?
                            .as_ref()
                            .unchecked_ref(),
                    )
                    .map_err(|_| JsValue::from_str("Failed to request animation frame"))?;

                animation_frame_id.replace(Some(animation_id));
                Ok(())
            }
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

        if let Some(state) = &mut self.state {
            // Create new render pipeline with updated shader
            self.source = fragment_shader_source;
            let (device, config) = (state.device.clone(), state.config.clone());
            state.render_pipeline = self.create_render_pipeline(&device, &config).map_err(|e| {
                web_sys::console::error_1(
                    &format!("Failed to create render pipeline: {e:?}").into(),
                );
                e
            })?;
        }

        // Restart rendering with new shader
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

    // Convert to WGSL instead of GLSL
    func.compile_to_glsl_fs(FRAGMENT_SHADER_TEMPLATE)
        .map_err(|e| format!("Failed to compile function to WGSL: {e:?}"))
}
