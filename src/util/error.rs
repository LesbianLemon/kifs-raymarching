use egui_wgpu::wgpu::{CreateSurfaceError, RequestDeviceError, SurfaceError};
use std::{
    error::Error,
    fmt::{self, Display},
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
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(match self {
                    $($Error::$ErrorVariant(error) => error.fmt(f)),+
                })?
            }
        }
    };
}

macro_rules! impl_enum_from {
    ($argument:ident: $FromError:ident -> $Error:ident::$ErrorVariant:ident($expr:expr)) => {
        impl From<$FromError> for $Error {
            fn from($argument: $FromError) -> Self {
                $Error::$ErrorVariant($expr)
            }
        }
    };
}

#[derive(Clone, Debug)]
pub struct RequestAdapterError;

impl Display for RequestAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Adapter request failed. No valid adapters found")
    }
}

impl_error!(RequestAdapterError);

#[derive(Clone, Debug)]
pub struct GUINotConfiguredError;

impl Display for GUINotConfiguredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GUI was not configured before rendering")
    }
}

impl_error!(GUINotConfiguredError);

#[derive(Clone, Debug)]
pub enum RenderError {
    CreateSurface(CreateSurfaceError),
    Surface(SurfaceError),
    RequestAdapter(RequestAdapterError),
    RequestDevice(RequestDeviceError),
    GUINotConfigured(GUINotConfiguredError),
}

impl_enum_error_display!(RenderError{ ::CreateSurface ::Surface ::RequestAdapter ::RequestDevice ::GUINotConfigured });
impl_error!(RenderError);

impl_enum_from!(error: CreateSurfaceError -> RenderError::CreateSurface(error));
impl_enum_from!(error: SurfaceError -> RenderError::Surface(error));
impl_enum_from!(error: RequestAdapterError -> RenderError::RequestAdapter(error));
impl_enum_from!(error: RequestDeviceError -> RenderError::RequestDevice(error));
impl_enum_from!(error: GUINotConfiguredError -> RenderError::GUINotConfigured(error));

#[derive(Debug)]
pub enum ApplicationError {
    EventLoop(EventLoopError),
    Render(RenderError),
}

impl_enum_error_display!(ApplicationError{ ::EventLoop ::Render });
impl_error!(ApplicationError);

impl_enum_from!(error: EventLoopError -> ApplicationError::EventLoop(error));
impl_enum_from!(error: OsError -> ApplicationError::EventLoop(error.into()));
impl_enum_from!(error: RenderError -> ApplicationError::Render(error));
impl_enum_from!(error: CreateSurfaceError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: SurfaceError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: RequestAdapterError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: RequestDeviceError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: GUINotConfiguredError -> ApplicationError::Render(error.into()));
