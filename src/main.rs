use std::sync::Arc;

use kurbo::{Affine, Vec2};
use wgpu::{
    util::DeviceExt as _, BindGroup, BindGroupLayout, Device, Queue, RenderPipeline, Surface,
    SurfaceConfiguration, TextureFormat,
};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::NamedKey,
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    scale: f32,
    _pad: f32,
    offset: [f32; 2],
    viewport_size: [f32; 2],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            scale: 1.0,
            offset: [0.0, 0.0],
            viewport_size: [800.0, 600.0],
            _pad: 0.0,
        }
    }

    fn create_uniform_buffer(
        device: &Device,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let new = Self::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[new]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
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
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Uniform Bind Group"),
        });

        (uniform_buffer, bind_group_layout, bind_group)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, -1.0] },
    Vertex { position: [ 1.0, -1.0] },
    Vertex { position: [ 1.0,  1.0] },
    Vertex { position: [-1.0, -1.0] },
    Vertex { position: [ 1.0,  1.0] },
    Vertex { position: [-1.0,  1.0] },
];

struct WindowState {
    window: Arc<winit::window::Window>,
    device: Arc<Device>,
    #[allow(unused)]
    queue: Arc<Queue>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: BindGroup,
    num_vertices: u32,
    mouse_down: bool,
    transform: Affine,
    prior_mouse_pos: Option<Vec2>,
}

impl WindowState {
    fn update_uniforms(&self) {
        let translation = self.transform.translation();
        let uniforms = Uniforms {
            scale: self.transform.as_coeffs()[0] as f32,
            offset: [translation.x as f32, translation.y as f32],
            _pad: 0.0,
            viewport_size: [self.config.width as f32, self.config.height as f32],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
        self.window.request_redraw();
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.update_uniforms();
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline); // 2.
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1); // 3.
        }
        self.queue.submit(Some(encoder.finish()));

        frame.present();
        Ok(())
    }
}

#[derive(Default)]
struct App {
    window_state: Option<WindowState>,
}

impl App {
    fn pipeline(
        device: &Device,
        format: TextureFormat,
        uniform_group_layout: &BindGroupLayout,
    ) -> RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[uniform_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",     // 1.
                buffers: &[Vertex::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        })
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_state.is_none() {
            let window = event_loop
                .create_window(Window::default_attributes())
                .unwrap();

            let window = Arc::new(window);

            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
            let surface = instance.create_surface(window.clone()).unwrap();
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }))
                .unwrap();

            let (device, queue) = pollster::block_on(
                adapter.request_device(&wgpu::DeviceDescriptor::default(), None),
            )
            .unwrap();

            let size = window.inner_size();

            let surface_caps = surface.get_capabilities(&adapter);
            let texture_format = surface_caps
                .formats
                .into_iter()
                .find(|it| matches!(it, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
                .unwrap();

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: texture_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);

            let (uniform_buffer, uniform_group_layout, uniform_group) =
                Uniforms::create_uniform_buffer(&device);
            let pipeline = App::pipeline(&device, texture_format, &uniform_group_layout);

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let num_vertices = VERTICES.len() as u32;

            let window_state = WindowState {
                window,
                device: Arc::new(device),
                queue: Arc::new(queue),
                surface,
                config,
                pipeline,
                vertex_buffer,
                num_vertices,
                uniform_buffer,
                uniform_bind_group: uniform_group,
                mouse_down: false,
                prior_mouse_pos: None,
                transform: Affine::IDENTITY,
            };

            self.window_state = Some(window_state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window_state) = &mut self.window_state {
                    match window_state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => window_state
                            .resize((window_state.config.width, window_state.config.height).into()),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(window_state) = &mut self.window_state {
                    window_state.resize(size);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(window_state) = &mut self.window_state {
                    if button == MouseButton::Left {
                        window_state.mouse_down = state == ElementState::Pressed;
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                const BASE: f64 = 1.05;
                const PIXELS_PER_LINE: f64 = 20.0;

                if let Some(window_state) = &mut self.window_state {
                    if let Some(prior_position) = window_state.prior_mouse_pos {
                        let exponent = if let MouseScrollDelta::PixelDelta(delta) = delta {
                            delta.y / PIXELS_PER_LINE
                        } else if let MouseScrollDelta::LineDelta(_, y) = delta {
                            y as f64
                        } else {
                            0.0
                        };
                        window_state.transform = Affine::translate(prior_position)
                            * Affine::scale(BASE.powf(exponent))
                            * Affine::translate(-prior_position)
                            * window_state.transform;
                        window_state.update_uniforms();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(window_state) = &mut self.window_state {
                    window_state.mouse_down = false;
                    window_state.prior_mouse_pos = None;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window_state) = &mut self.window_state {
                    let position = Vec2::new(position.x, position.y);
                    if window_state.mouse_down {
                        if let Some(prior) = window_state.prior_mouse_pos {
                            window_state.transform =
                                Affine::translate(position - prior) * window_state.transform;
                            window_state.update_uniforms();
                        }
                    }
                    window_state.prior_mouse_pos = Some(position);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(window_state) = &mut self.window_state {
                    if let winit::keyboard::Key::Named(NamedKey::Space) = event.logical_key {
                        window_state.transform = Affine::IDENTITY;
                        window_state.update_uniforms();
                    }
                }
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
