use egui_wgpu::{ScreenDescriptor, wgpu};
use limited_queue::LimitedQueue;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use wgpu::wgt::SamplerDescriptor;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    window::Window,
};

use crate::error::{RenderError, RenderStateError};
use crate::util::math::Radians;

pub(crate) mod graphics;
pub(crate) mod gui;

use graphics::GraphicState;
use gui::GuiState;

#[derive(Clone, Debug, Default)]
pub struct RenderStateOptions {
    pub power_preference: wgpu::PowerPreference,
    pub required_features: wgpu::Features,
    pub required_limits: wgpu::Limits,
}

pub(crate) struct RenderState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    graphic_state: GraphicState,
    gui_state: GuiState,
    frametimes: LimitedQueue<Duration>,
}

impl RenderState {
    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'static>,
        options: &RenderStateOptions,
    ) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: options.power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
    }

    async fn create_device_and_queue(
        adapter: &wgpu::Adapter,
        options: &RenderStateOptions,
    ) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: options.required_features,
                required_limits: options.required_limits.clone(),
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
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
    fn create_render_texture(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: PhysicalSize<u32>,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render_texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }

    #[must_use]
    fn create_render_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&SamplerDescriptor {
            label: Some("render_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.,
            lod_max_clamp: 32.,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        })
    }

    /// ## Errors
    /// - `RenderStateError::CreateSurface(CreateSurfaceError)` when surface creation failed
    /// - `RenderStateError::RequestAdapter(RequestAdapterError)` when adapter request failed
    /// - `RenderStateError::RequestDevice(RequestDeviceError)` when device request failed
    pub(crate) async fn new(
        window: Arc<Window>,
        options: &RenderStateOptions,
    ) -> Result<Self, RenderStateError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::default()
        });

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone())?;

        let adapter = Self::create_adapter(&instance, &surface, options).await?;
        let (device, queue) = Self::create_device_and_queue(&adapter, options).await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        // Surface is guaranteed compatible with adapter on adapter initialisation, meaning surface_format and alpha_mode are well defined
        let surface_format = Self::surface_format(&surface_capabilities);
        let alpha_mode = Self::alpha_mode(&surface_capabilities);
        let config = Self::create_surface_config(surface_format, alpha_mode, size);

        let graphic_state = GraphicState::new(&window, &device, &config);
        let gui_state = GuiState::new(&window, &device, surface_format);

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
            frametimes: LimitedQueue::with_capacity(5),
        })
    }

    #[must_use]
    pub(crate) fn window(&self) -> &Window {
        &self.window
    }

    #[must_use]
    pub(crate) fn size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub(crate) fn drop_window(self) {
        drop(self.window);
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.graphic_state
                .update_screen_data(&self.queue, new_size.into());
            self.window.request_redraw();
        }
    }

    pub(crate) fn window_event(&mut self, event: &WindowEvent) {
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

    pub(crate) fn device_event(&mut self, event: &DeviceEvent) {
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
    /// - `RenderError::GUINotConfigured(GUINotConfiguredError)` when tried to render unconfigured GUI
    pub(crate) fn render(&mut self) -> Result<(), RenderError> {
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
                label: Some("render_encoder"),
            });

        // Prepare everything for render
        self.gui_state.update_gui(
            &self.window,
            &self.device,
            &self.queue,
            &mut encoder,
            &screen_descriptor,
        );
        self.graphic_state
            .update_options(&self.queue, self.gui_state.gui_data().into());

        // Drawing and rendering calls happen here
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

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
