use egui_wgpu::{ScreenDescriptor, wgpu};

use std::sync::Arc;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

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

    fn create_config(
        surface: &wgpu::Surface<'static>,
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

    pub async fn new(window: Arc<Window>, options: &RenderStateOptions) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = RenderState::create_adapter(&instance, &surface, options).await;
        let (device, queue) = RenderState::create_device_and_queue(&adapter, options).await;

        let config = RenderState::create_config(&surface, &adapter, size);

        let graphic_state = graphics::GraphicState::new(&window, &device, &config);
        let gui_state = gui::GuiState::new(&window, &device, wgpu::TextureFormat::Rgba8UnormSrgb);

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

            self.graphic_state
                .update_size_uniform(&self.queue, new_size);
        }
    }

    // Processes the input event and schedules a redraw if needed
    pub fn input(&mut self, event: &WindowEvent) {
        let response = self.gui_state.input(&self.window, event);

        if response.repaint {
            self.window.request_redraw();
        }
    }

    pub fn update(&mut self) {
        // todo!()
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

            self.graphic_state.render(&mut render_pass);
            self.gui_state.render(render_pass, &screen_descriptor);
        }

        // Submit the queue to the GPU
        self.queue.submit(std::iter::once(encoder.finish()));
        self.window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }
}
