use egui_wgpu::{
    ScreenDescriptor,
    wgpu::{self, InstanceDescriptor},
};

use std::sync::Arc;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::math::Radians;

pub mod graphics;
pub mod gui;

#[derive(Default)]
pub struct RenderStateOptions {
    power_preference: wgpu::PowerPreference,
    required_features: wgpu::Features,
    required_limits: wgpu::Limits,
}

pub struct RenderState {
    window: Arc<Window>,
    size: PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    graphic_state: graphics::GraphicState,
    gui_state: gui::GuiState,
    pipeline: wgpu::RenderPipeline,
}

impl RenderState {
    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'static>,
        options: &RenderStateOptions,
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
        options: &RenderStateOptions,
    ) -> (wgpu::Device, wgpu::Queue) {
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: options.required_features,
                    required_limits: options.required_limits.clone(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::default(),
                    // This is a change made in wgpu version 25.0.0,
                    // but egui_wgpu is not yet updated to that version
                    // trace: wgpu::Trace::Off,
                },
                None,
            )
            .await
            .unwrap()
    }

    fn create_surface_config(
        surface_capabilities: &wgpu::SurfaceCapabilities,
        surface_format: &wgpu::TextureFormat,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
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
                label: Some("render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
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

    fn surface_format(surface_capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
        // surface_capabilities vector is guaranteed to contain Bgra8UnormSrgb by the library
        surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0])
    }

    pub async fn new(window: Arc<Window>, options: &RenderStateOptions) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..InstanceDescriptor::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = Self::create_adapter(&instance, &surface, options).await;
        let (device, queue) = Self::create_device_and_queue(&adapter, options).await;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = Self::surface_format(&surface_capabilities);
        let config = Self::create_surface_config(&surface_capabilities, &surface_format, size);

        let graphic_state = graphics::GraphicState::new(&window, &device);
        let gui_state = gui::GuiState::new(&window, &device, surface_format);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raymarching_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = Self::create_render_pipeline(
            &device,
            &[
                graphic_state.size_uniform().bind_group_layout(),
                graphic_state.camera_uniform().bind_group_layout(),
                gui_state.gui_uniform().bind_group_layout(),
            ],
            &config,
            &shader,
        );

        // Configure the surface for the first time
        surface.configure(&device, &config);

        Self {
            window,
            size,
            surface,
            device,
            queue,
            config,
            graphic_state,
            gui_state,
            pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.graphic_state.update_size(&self.queue, new_size);
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        let response = self.gui_state.window_event(&self.window, event);
        if self.gui_state.wants_pointer_input() || response.consumed {
            self.window.request_redraw();
            return;
        }

        match event {
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => self.graphic_state.enable_camera_rotation(),
                ElementState::Released => self.graphic_state.disable_camera_rotation(),
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let distance = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => *dy,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: dy, .. }) => {
                        (*dy / 10.) as f32
                    }
                };

                self.graphic_state.zoom_camera(&self.queue, distance);
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    pub fn device_event(&mut self, event: &DeviceEvent) {
        // Right is positive x and down is positive y
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.gui_state.mouse_motion((*dx, *dy));
            if self.graphic_state.is_camera_rotatable() {
                self.graphic_state.rotate_camera(
                    &self.queue,
                    Radians::from_degrees(-(*dx / 10.) as f32),
                    Radians::from_degrees((*dy / 10.) as f32),
                );
                self.window.request_redraw();
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.size.width, self.size.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.gui_state.prerender(
            &self.window,
            &self.device,
            &self.queue,
            &mut encoder,
            &screen_descriptor,
        );

        // Drawing and rendering happens here
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, self.graphic_state.size_uniform().bind_group(), &[]);
            render_pass.set_bind_group(1, self.graphic_state.camera_uniform().bind_group(), &[]);
            render_pass.set_bind_group(2, self.gui_state.gui_uniform().bind_group(), &[]);

            self.graphic_state.render(&mut render_pass);
            // Execute GUI rendering last so it stays on top of our graphics and because it consumes the render_pass
            self.gui_state.render(render_pass, &screen_descriptor);
        }

        // Submit the queue to the GPU
        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        surface_texture.present();

        // This is not what I want to do, but currently have no better solution
        // It might cause a crash if the window is open for too long, who knows...
        self.window.request_redraw();

        Ok(())
    }
}
