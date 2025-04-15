use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

// struct Uniform<T> {
//     value: T,
//     buffer: Buffer,
//     bind_group: BindGroup,
// }

// struct Uniforms<'a> {
//     uniforms: &'a [Uniform],
// }

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SizeUniform {
    width: u32,
    height: u32,
}

#[derive(Default)]
pub struct GraphicsStateOptions {
    power_preference: wgpu::PowerPreference,
    required_features: wgpu::Features,
    required_limits: wgpu::Limits,
}

struct GraphicsState<'a> {
    size: PhysicalSize<u32>,
    size_uniform: SizeUniform,
    size_uniform_buffer: wgpu::Buffer,
    size_bind_group: wgpu::BindGroup,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
}

impl<'a> GraphicsState<'a> {
    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'a>,
        options: &GraphicsStateOptions,
    ) -> wgpu::Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap()
    }

    async fn create_device_and_queue(
        adapter: &wgpu::Adapter,
        options: &GraphicsStateOptions,
    ) -> (wgpu::Device, wgpu::Queue) {
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: options.required_features,
                    required_limits: options.required_limits.clone(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap()
    }

    fn create_size_uniform_buffer(
        device: &wgpu::Device,
        size: PhysicalSize<u32>,
    ) -> (SizeUniform, wgpu::Buffer) {
        let size_uniform = SizeUniform {
            width: size.width,
            height: size.height,
        };

        let size_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Size Uniform Buffer"),
            contents: bytemuck::cast_slice(&[size_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        (size_uniform, size_uniform_buffer)
    }

    fn create_size_uniform_bind(
        device: &wgpu::Device,
        size_uniform_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let size_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("size_bind_group_layout"),
            });

        let size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: size_uniform_buffer.as_entire_binding(),
            }],
            label: Some("size_bind_group"),
        });

        (size_bind_group_layout, size_bind_group)
    }

    fn create_config(
        surface: &wgpu::Surface<'a>,
        adapter: &wgpu::Adapter,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            cache: None,
        })
    }

    async fn new(window: &'a Window, options: GraphicsStateOptions) -> GraphicsState<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = GraphicsState::create_adapter(&instance, &surface, &options).await;
        let (device, queue) = GraphicsState::create_device_and_queue(&adapter, &options).await;

        let (size_uniform, size_uniform_buffer) =
            GraphicsState::create_size_uniform_buffer(&device, size);
        let (size_bind_group_layout, size_bind_group) =
            GraphicsState::create_size_uniform_bind(&device, &size_uniform_buffer);

        let config = GraphicsState::create_config(&surface, &adapter, size);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline = GraphicsState::create_render_pipeline(
            &device,
            &[&size_bind_group_layout],
            &config,
            &shader,
        );

        Self {
            size,
            size_uniform,
            size_uniform_buffer,
            size_bind_group,
            surface,
            device,
            queue,
            config,
            window,
            render_pipeline,
        }
    }

    fn window(&self) -> &Window {
        self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let size_uniform = SizeUniform {
                width: self.size.width,
                height: self.size.height,
            };
            self.size_uniform = size_uniform;
            self.queue.write_buffer(
                &self.size_uniform_buffer,
                0,
                bytemuck::cast_slice(&[size_uniform]),
            );
        }
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // todo!()
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.size_bind_group, &[]);

            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run(options: GraphicsStateOptions) -> Result<(), EventLoopError> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop)?;

    let mut state = GraphicsState::new(&window, options).await;
    let surface_configured = true;

    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.window().request_redraw();

                        if !surface_configured {
                            return;
                        }

                        state.update();
                        match state.render() {
                            Ok(()) => {}

                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size);
                            }

                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                                log::error!("OutOfMemory");
                                control_flow.exit();
                            }

                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timeout");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    })
}
