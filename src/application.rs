use egui_wgpu::wgpu;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::error::ApplicationError;
use crate::render::{RenderState, RenderStateOptions};

pub struct Application {
    state: Option<RenderState>,
    state_options: RenderStateOptions,
}

impl Application {
    pub fn new(state_options: RenderStateOptions) -> Self {
        Self {
            state: None,
            state_options,
        }
    }

    fn is_configured(&self) -> bool {
        self.state.is_some()
    }

    fn render(&mut self) -> Result<(), ApplicationError> {
        if let Some(state) = self.state.as_mut() {
            match state.render() {
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    state.resize(state.size());
                }
                Err(wgpu::SurfaceError::Timeout) => {
                    log::warn!("Surface timed out");
                }
                Err(error) => {
                    log::error!("UnrecoverableError: {error}");
                    return Err(error.into());
                }
                _ => {}
            }
        };

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), ApplicationError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self)?;

        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.is_configured() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );

            let new_state =
                pollster::block_on(RenderState::new(window.clone(), &self.state_options));
            self.state = Some(new_state);

            window.request_redraw();
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
                    // To avoid a segfault, force Rust to drop values before closing
                    self.state = None;

                    event_loop.exit();
                }
                WindowEvent::Resized(new_size) => state.resize(new_size),
                WindowEvent::RedrawRequested => {
                    if let Err(error) = self.render() {
                        log::error!("Application came into an unrecoverable error: {error}");
                        event_loop.exit();
                    }
                }
                _ => {}
            }
        } else {
            log::warn!("Cannot process window event due to unconfigured application")
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
            log::warn!("Cannot process device event due to unconfigured application")
        }
    }
}
