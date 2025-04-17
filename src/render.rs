use egui_wgpu::wgpu;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::graphics::GraphicState;

#[derive(Default)]
pub struct RenderStateOptions {
    power_preference: wgpu::PowerPreference,
    required_features: wgpu::Features,
    required_limits: wgpu::Limits,
}

struct RenderState {
    size: PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    graphic_state: GraphicState,
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

    async fn new(window: &Arc<Window>, options: &RenderStateOptions) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(window)).unwrap();

        let adapter = RenderState::create_adapter(&instance, &surface, options).await;
        let (device, queue) = RenderState::create_device_and_queue(&adapter, options).await;

        let config = RenderState::create_config(&surface, &adapter, size);

        let graphic_state = GraphicState::new(window, &device, &config);
        // let gui_state = GuiState::new(window);

        Self {
            size,
            surface,
            device,
            queue,
            config,
            graphic_state,
            // gui_state,
        }
    }

    fn size(&self) -> PhysicalSize<u32> {
        self.size
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.graphic_state
                .update_size_uniform(&self.queue, new_size);
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

            self.graphic_state.prepare_render_pass(&mut render_pass);

            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
struct Application {
    window: Option<Arc<Window>>,
    state: Option<RenderState>,
    state_options: RenderStateOptions,
}

impl Application {
    fn set_options(&mut self, options: RenderStateOptions) {
        self.state_options = options;
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let new_window = event_loop
                .create_window(Window::default_attributes())
                .unwrap();
            let window = self.window.insert(Arc::new(new_window));

            let new_state = pollster::block_on(RenderState::new(window, &self.state_options));
            let _ = self.state.insert(new_state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match self {
            Self {
                window: Some(window),
                state: Some(state),
                ..
            } => {
                if window_id != window.id() {
                    return;
                }

                if state.input(&event) {
                    return;
                }

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
                    } => {
                        // To avoid a segfault, force Rust to drop values before closing
                        self.window = None;
                        self.state = None;

                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),
                    WindowEvent::RedrawRequested => {
                        window.request_redraw();

                        state.update();
                        match state.render() {
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size());
                            }
                            Err(wgpu::SurfaceError::OutOfMemory | wgpu::SurfaceError::Other) => {
                                log::error!("OutOfMemory");
                                event_loop.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                log::warn!("Surface timed out");
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                log::warn!("Cannot process window event with unconfigured state and window");
            }
        }
    }
}

pub fn run(options: RenderStateOptions) -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;

    let mut app = Application::default();
    app.set_options(options);

    event_loop.run_app(&mut app)
}
