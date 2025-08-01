use egui_wgpu::{ScreenDescriptor, wgpu};
use limited_queue::LimitedQueue;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::{
    data::buffer::{
        BufferGroup, BufferGroupInit, BufferGroupLayoutEntry, FixedEntryBufferGroupDescriptor,
    },
    util::{
        error::{RenderError, RequestAdapterError},
        math::Radians,
    },
};

pub mod graphics;
pub mod gui;

use graphics::GraphicState;
use gui::GuiState;

#[derive(Default)]
pub struct RenderStateOptions {
    pub power_preference: wgpu::PowerPreference,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
}

pub struct RenderState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    graphic_state: GraphicState,
    gui_state: GuiState,
    uniform_group: BufferGroup,
    pipeline: wgpu::RenderPipeline,
    frametimes: LimitedQueue<Duration>,
}

impl RenderState {
    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'static>,
        options: &RenderStateOptions,
    ) -> Result<wgpu::Adapter, RequestAdapterError> {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RequestAdapterError)
    }

    async fn create_device_and_queue(
        adapter: &wgpu::Adapter,
        options: &RenderStateOptions,
    ) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
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
    }

    #[must_use]
    fn surface_format(surface_capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
        // Find first preferred SRGB format, otherwise use the general preferred one
        surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0])
    }

    #[must_use]
    fn alpha_mode(surface_capabilities: &wgpu::SurfaceCapabilities) -> wgpu::CompositeAlphaMode {
        if surface_capabilities
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Auto)
        {
            wgpu::CompositeAlphaMode::Auto
        } else {
            surface_capabilities.alpha_modes[0]
        }
    }

    #[must_use]
    fn create_surface_config(
        format: wgpu::TextureFormat,
        alpha_mode: wgpu::CompositeAlphaMode,
        size: PhysicalSize<u32>,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync, // This is supported on all platforms if adapter is compatible with the surface
            alpha_mode,
            view_formats: vec![], // View formats of the surface format are always allowed, even when specifying an empty vector
            desired_maximum_frame_latency: 2,
        }
    }

    #[must_use]
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

    /// ## Errors
    /// - `RenderError::CreateSurface(CreateSurfaceError)` when surface creation failed
    /// - `RenderError::RequestAdapter(RequestAdapterError)` when adapter request failed
    /// - `RenderError::RequestDevice(RequestDeviceError)` when device request failed
    pub async fn new(
        window: Arc<Window>,
        options: &RenderStateOptions,
    ) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = Self::create_adapter(&instance, &surface, options).await?;
        let (device, queue) = Self::create_device_and_queue(&adapter, options).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Surface is guaranteed compatible with adapter on adapter initialisation,
        // meaning surface_format and alpha_mode are well defined
        let surface_format = Self::surface_format(&surface_capabilities);
        let alpha_mode = Self::alpha_mode(&surface_capabilities);
        let config = Self::create_surface_config(surface_format, alpha_mode, window.inner_size());

        let graphic_state = GraphicState::new(&window, &device);
        let gui_state = GuiState::new(&window, &device, surface_format);
        let uniform_group =
            device.create_fixed_entry_buffer_group(&FixedEntryBufferGroupDescriptor {
                label: Some("uniform_buffer_group"),
                buffers: &[
                    graphic_state.size_uniform_buffer().buffer(),
                    graphic_state.camera_uniform_buffer().buffer(),
                    gui_state.gui_uniform_buffer().buffer(),
                ],
                entry: BufferGroupLayoutEntry {
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raymarching_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let pipeline = Self::create_render_pipeline(
            &device,
            &[uniform_group.bind_group_layout()],
            &config,
            &shader,
        );

        // Configure the surface for the first time
        surface.configure(&device, &config);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            graphic_state,
            gui_state,
            uniform_group,
            pipeline,
            frametimes: LimitedQueue::with_capacity(5),
        })
    }

    #[must_use]
    pub fn window(&self) -> &Window {
        &self.window
    }

    #[must_use]
    pub fn size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.graphic_state.update_size(&self.queue, new_size);
            self.window.request_redraw();
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        // Check if event was for the GUI
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
                    #[allow(clippy::cast_possible_truncation)]
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
                    #[allow(clippy::cast_possible_truncation)]
                    Radians::from_degrees(-(*dx / 10.) as f32),
                    #[allow(clippy::cast_possible_truncation)]
                    Radians::from_degrees((*dy / 10.) as f32),
                );
                self.window.request_redraw();
            }
        }
    }

    /// ## Errors
    /// - `RenderError::Surface(SurfaceError)` when getting current surface texture failed
    /// - `RenderError::GUINotConfigured(GUINotConfiguredError)` when tried to render GUI prior to configuring it
    pub fn render(&mut self) -> Result<(), RenderError> {
        let start_time = Instant::now();

        let surface_texture = self.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.size().width, self.size().height],
            #[allow(clippy::cast_possible_truncation)]
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

        // Drawing and rendering calls happen here
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
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, self.uniform_group.bind_group(), &[]);

            self.graphic_state.render(&mut render_pass);
            // Execute GUI rendering last so it stays on top of our graphics and because it consumes the render_pass
            self.gui_state.render(render_pass, &screen_descriptor)?;
        }

        // Submit the queue to the GPU and present the changed surface
        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        surface_texture.present();

        self.frametimes
            .push(Instant::now().duration_since(start_time));
        println!(
            "Frame time: {}ms",
            self.frametimes.iter().sum::<Duration>().as_millis()
        );

        // Request a window redraw
        // This is not what I want to do, but currently have no better solution
        // It might cause a crash if the window is open for too long, who knows...
        self.window.request_redraw();

        Ok(())
    }
}
