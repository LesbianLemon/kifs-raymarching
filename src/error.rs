use std::{error, fmt};

use egui_wgpu::wgpu::SurfaceError;
use winit::error::EventLoopError;

#[derive(Debug)]
pub enum ApplicationError {
    EventLoop(EventLoopError),
    Render(SurfaceError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApplicationError::EventLoop(error) => error.fmt(f),
            ApplicationError::Render(error) => error.fmt(f),
        }
    }
}

impl error::Error for ApplicationError {}

impl From<EventLoopError> for ApplicationError {
    fn from(error: EventLoopError) -> Self {
        ApplicationError::EventLoop(error)
    }
}

impl From<SurfaceError> for ApplicationError {
    fn from(error: SurfaceError) -> Self {
        ApplicationError::Render(error)
    }
}
