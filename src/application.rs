use egui_wgpu::wgpu;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::util::error::{ApplicationError, RenderStateUnconfiguredError};
use crate::{
    render::{RenderState, RenderStateOptions},
    util::error::RenderError,
};

pub struct Application {
    active: bool,
    state: Option<RenderState>,
    state_options: RenderStateOptions,
}

impl Application {
    #[must_use]
    pub fn new(state_options: RenderStateOptions) -> Self {
        Self {
            active: true,
            state: None,
            state_options,
        }
    }

    /// ## Errors
    /// - `ApplicationError::EventLoop(EventLoopError)` when event loop creation failed or event loop terminated with an error
    pub fn run(&mut self) {
        match EventLoop::new() {
            Ok(event_loop) => {
                event_loop.set_control_flow(ControlFlow::Poll);

                match event_loop.run_app(self) {
                    Ok(()) => {
                        Self::exit_message();
                    }
                    Err(error) => {
                        Self::error_message(&error.into());
                        Self::exit_message();
                    }
                }
            }
            Err(error) => {
                Self::error_message(&error.into());
                Self::exit_message();
            }
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

        match state.render() {
            Err(RenderError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)) => {
                state.resize(state.size());
            }
            Err(RenderError::Surface(wgpu::SurfaceError::Timeout)) => {
                log::warn!("Surface timed out");
            }
            Err(error) => {
                return Err(error.into());
            }
            _ => {}
        }

        Ok(())
    }

    fn deactivate(&mut self) {
        self.active = false;
        // To avoid a segfault, force Rust to drop values before closing
        self.state = None;
    }

    fn error_message(error: &ApplicationError) {
        log::error!("Application came into an unrecoverable error: {error}");
    }

    fn exit_message() {
        log::info!("Exiting application...");
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
                            Self::error_message(&error.into());
                            event_loop.exit();
                        }
                    }
                }
                Err(error) => {
                    Self::error_message(&error.into());
                    event_loop.exit();
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
                        self.deactivate();
                        event_loop.exit();
                    }
                    WindowEvent::Resized(new_size) => state.resize(new_size),
                    WindowEvent::RedrawRequested => {
                        if let Err(error) = self.render() {
                            self.deactivate();
                            Self::error_message(&error);
                            event_loop.exit();
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
