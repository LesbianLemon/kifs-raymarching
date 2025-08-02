use egui_wgpu::wgpu;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};
use winit::error::{EventLoopError, OsError};

macro_rules! impl_error {
    ($Error:ty) => {
        impl Error for $Error {}
    };
}

macro_rules! impl_enum_error_display {
    ($Error:ident$({$(::$ErrorVariant:ident)+})?) => {
        impl Display for $Error {
            fn fmt(&self, f: &mut Formatter) -> Result {
                $(match self {
                    $($Error::$ErrorVariant(error) => error.fmt(f)),+
                })?
            }
        }
    };
}

macro_rules! impl_enum_from {
    ($argument:ident: $($FromErrorwPart:ident)::+ -> $Error:ident::$ErrorVariant:ident($expr:expr)) => {
        impl From<$($FromErrorwPart)::+> for $Error {
            fn from($argument: $($FromErrorwPart)::+) -> Self {
                $Error::$ErrorVariant($expr)
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub struct RenderStateUnconfiguredError;

impl Display for RenderStateUnconfiguredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Render state was not configured before rendering")
    }
}

impl_error!(RenderStateUnconfiguredError);

#[derive(Clone, Copy, Debug)]
pub struct GUIUnconfiguredError;

impl Display for GUIUnconfiguredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "GUI was not configured before rendering")
    }
}

impl_error!(GUIUnconfiguredError);

#[derive(Clone, Debug)]
pub enum RenderError {
    Surface(wgpu::SurfaceError),
    GUIUnconfigured(GUIUnconfiguredError),
}

impl_enum_error_display!(RenderError{ ::Surface ::GUIUnconfigured });
impl_error!(RenderError);

impl_enum_from!(error: wgpu::SurfaceError -> RenderError::Surface(error));
impl_enum_from!(error: GUIUnconfiguredError -> RenderError::GUIUnconfigured(error));

#[derive(Clone, Debug)]
pub enum RenderStateError {
    CreateSurface(wgpu::CreateSurfaceError),
    RequestAdapter(wgpu::RequestAdapterError),
    RequestDevice(wgpu::RequestDeviceError),
}

impl_enum_error_display!(RenderStateError{ ::CreateSurface ::RequestAdapter ::RequestDevice });
impl_error!(RenderStateError);

impl_enum_from!(error: wgpu::CreateSurfaceError -> RenderStateError::CreateSurface(error));
impl_enum_from!(error: wgpu::RequestAdapterError -> RenderStateError::RequestAdapter(error));
impl_enum_from!(error: wgpu::RequestDeviceError -> RenderStateError::RequestDevice(error));

#[derive(Debug)]
pub enum ApplicationError {
    EventLoop(EventLoopError),
    RenderStateUnconfigured(RenderStateUnconfiguredError),
    RenderState(RenderStateError),
    Render(RenderError),
}

impl_enum_error_display!(ApplicationError{ ::EventLoop ::RenderStateUnconfigured ::RenderState ::Render });
impl_error!(ApplicationError);

impl_enum_from!(error: EventLoopError -> ApplicationError::EventLoop(error));
impl_enum_from!(error: OsError -> ApplicationError::EventLoop(error.into()));
impl_enum_from!(error: RenderStateUnconfiguredError -> ApplicationError::RenderStateUnconfigured(error));
impl_enum_from!(error: RenderStateError -> ApplicationError::RenderState(error));
impl_enum_from!(error: wgpu::CreateSurfaceError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: wgpu::RequestAdapterError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: wgpu::RequestDeviceError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: RenderError -> ApplicationError::Render(error));
impl_enum_from!(error: wgpu::SurfaceError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: GUIUnconfiguredError -> ApplicationError::Render(error.into()));
