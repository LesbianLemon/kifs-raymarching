use egui_wgpu::wgpu;

use std::{error, fmt, sync::Arc};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::render::{RenderState, RenderStateOptions};

struct UrecoverableError;

impl fmt::Display for UrecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        f.pad("Application came into an unrecoverable error")
    }
}

impl fmt::Debug for UrecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        f.debug_struct("UnrecoverableError").finish()
    }
}

impl error::Error for UrecoverableError {}

struct Application {
    state: Option<RenderState>,
    state_options: RenderStateOptions,
}

impl Application {
    fn new(state_options: RenderStateOptions) -> Self {
        Self {
            state: None,
            state_options,
        }
    }

    fn is_configured(&self) -> bool {
        self.state.is_some()
    }

    fn render(&mut self) -> Result<(), UrecoverableError> {
        if let Some(state) = self.state.as_mut() {
            state.update();

            match state.render() {
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    state.resize(state.size());
                }
                Err(wgpu::SurfaceError::Timeout) => {
                    log::warn!("Surface timed out");
                }
                Err(error) => {
                    log::error!("UnrecoverableError: {error}");
                    return Err(UrecoverableError);
                }
                _ => {}
            }
        };

        Ok(())
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
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
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Self {
            state: Some(state), ..
        } = self
        {
            if window_id != state.window().id() {
                return;
            }

            state.input(&event);

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
                    self.render().unwrap_or_else(|_e| event_loop.exit());
                }
                _ => {}
            }
        } else {
            log::warn!("Cannot process window event due to unconfigured application")
        }
    }
}

pub fn run(options: RenderStateOptions) -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new(options);

    event_loop.run_app(&mut app)
}
