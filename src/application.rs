use egui_wgpu::wgpu;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::error::{
    ApplicationError, RenderError, RenderStateUnconfiguredError, SurfaceMissizedError,
};
use crate::render::{RenderState, RenderStateOptions};

pub struct Application {
    active: bool,
    state: Option<RenderState>,
    state_options: RenderStateOptions,
    // Needs to be Option<_> due to requiring .take() later, since ApplicationError is not Clone nor Copy
    exit_error: Option<ApplicationError>,
}

impl Application {
    #[must_use]
    pub fn new(state_options: RenderStateOptions) -> Self {
        Self {
            active: true,
            state: None,
            state_options,
            exit_error: None,
        }
    }

    /// ## Errors
    /// - `ApplicationError::EventLoop(EventLoopError)` when event loop creation failed or event loop terminated with an error
    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self)?;

        log::info!("Exiting application...");
        match self.exit_error.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    #[must_use]
    fn is_configured(&self) -> bool {
        self.state.is_some()
    }

    /// ## Errors
    /// - `ApplicationError::RenderStateUnconfigured(RenderStateUnconfiguredError)` when tried to render with unconfigured render state
    /// - `ApplicationError::Render(RenderError)` when rendering failed
    fn render(&mut self) -> Result<(), ApplicationError> {
        let state = self
            .state
            .as_mut()
            .ok_or(ApplicationError::RenderStateUnconfigured(
                RenderStateUnconfiguredError,
            ))?;

        let render_result = state.render();
        match render_result {
            Err(RenderError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)) => {
                state.resize(state.size());
            }
            Err(
                RenderError::Surface(wgpu::SurfaceError::Timeout)
                | RenderError::SurfaceMissized(SurfaceMissizedError),
            ) => {
                log::warn!("{}", render_result.err().unwrap());
            }
            Err(error) => {
                return Err(error.into());
            }
            _ => {}
        }

        Ok(())
    }

    fn exit(&mut self, exit_error: Option<ApplicationError>) {
        self.active = false;
        // To avoid a segfault, force Rust to drop values before closing
        self.state = None;
        self.exit_error = exit_error;
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Run once when the window is still not created and start the window event loop
        if !self.is_configured() && self.active {
            match event_loop.create_window(Window::default_attributes()) {
                Ok(window) => {
                    let window = Arc::new(window);

                    match pollster::block_on(RenderState::new(window.clone(), &self.state_options))
                    {
                        Ok(state) => {
                            self.state = Some(state);
                            window.request_redraw();
                        }
                        Err(error) => {
                            event_loop.exit();
                            self.exit(Some(error.into()));
                        }
                    }
                }
                Err(error) => {
                    event_loop.exit();
                    self.exit(Some(error.into()));
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match self {
            Self {
                active: true,
                state: Some(state),
                ..
            } => {
                if window_id != state.window().id() {
                    return;
                }

                state.window_event(&event);

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
                        event_loop.exit();
                        self.exit(None);
                    }
                    WindowEvent::Resized(new_size) => state.resize(new_size),
                    WindowEvent::RedrawRequested => {
                        if let Err(error) = self.render() {
                            event_loop.exit();
                            self.exit(Some(error));
                        }
                    }
                    _ => {}
                }
            }
            Self {
                active: true,
                state: None,
                ..
            } => {
                log::warn!(
                    "Could not process window event due to unconfigured application. Continuing..."
                );
            }
            Self { active: false, .. } => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match self {
            Self {
                active: true,
                state: Some(state),
                ..
            } => {
                state.device_event(&event);
            }
            Self {
                active: true,
                state: None,
                ..
            } => {
                log::warn!(
                    "Could not process device event due to unconfigured application. Continuing..."
                );
            }
            Self { active: false, .. } => {}
        }
    }
}
