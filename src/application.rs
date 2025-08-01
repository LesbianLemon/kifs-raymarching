use egui_wgpu::wgpu;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::util::error::ApplicationError;
use crate::{
    render::{RenderState, RenderStateOptions},
    util::error::RenderError,
};

pub struct Application {
    state: Option<RenderState>,
    state_options: RenderStateOptions,
}

impl Application {
    #[must_use]
    pub fn new(state_options: RenderStateOptions) -> Self {
        Self {
            state: None,
            state_options,
        }
    }

    /// ## Errors
    /// - `ApplicationError::EventLoop(EventLoopError)` when event loop creation failed or event loop terminated with an error
    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self)?;

        Ok(())
    }

    #[must_use]
    fn is_configured(&self) -> bool {
        self.state.is_some()
    }

    /// ## Errors
    /// - `ApplicationError::Render(RenderError)` when rendering fails
    fn render(&mut self) -> Result<(), ApplicationError> {
        match self.state.as_mut() {
            Some(state) => match state.render() {
                Err(RenderError::Surface(
                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                )) => {
                    state.resize(state.size());
                }
                Err(RenderError::Surface(wgpu::SurfaceError::Timeout)) => {
                    log::warn!("Surface timed out");
                }
                Err(error) => {
                    return Err(error.into());
                }
                _ => {}
            },
            None => {
                log::warn!("Could not render due to unconfigured render state");
            }
        }

        Ok(())
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        // To avoid a segfault, force Rust to drop values before closing
        self.state = None;
        event_loop.exit();
    }

    fn exit_with_error(&mut self, error: &ApplicationError, event_loop: &ActiveEventLoop) {
        log::error!("Application came into an unrecoverable error: {error}");
        self.exit(event_loop);
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Run once when the window is still not created and start the window event loop
        if !self.is_configured() {
            match event_loop.create_window(Window::default_attributes()) {
                Ok(window) => {
                    let window = Arc::new(window);

                    match pollster::block_on(RenderState::new(window.clone(), &self.state_options))
                    {
                        Ok(state) => {
                            self.state = Some(state);
                            window.request_redraw();
                        }
                        Err(error) => self.exit_with_error(&error.into(), event_loop),
                    }
                }
                Err(error) => self.exit_with_error(&error.into(), event_loop),
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Self {
            state: Some(state), ..
        } = self
        {
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
                    self.exit(event_loop);
                }
                WindowEvent::Resized(new_size) => state.resize(new_size),
                WindowEvent::RedrawRequested => {
                    if let Err(error) = self.render() {
                        self.exit_with_error(&error, event_loop);
                    }
                }
                _ => {}
            }
        } else {
            log::warn!("Could not process window event due to unconfigured application");
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Self {
            state: Some(state), ..
        } = self
        {
            state.device_event(&event);
        } else {
            log::warn!("Could not process device event due to unconfigured application");
        }
    }
}
