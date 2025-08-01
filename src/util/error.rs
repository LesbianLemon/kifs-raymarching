use egui_wgpu::wgpu::{CreateSurfaceError, RequestDeviceError, SurfaceError};
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
    ($argument:ident: $FromError:ident -> $Error:ident::$ErrorVariant:ident($expr:expr)) => {
        impl From<$FromError> for $Error {
            fn from($argument: $FromError) -> Self {
                $Error::$ErrorVariant($expr)
            }
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestAdapterError;

impl Display for RequestAdapterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Adapter request failed, no valid adapters found")
    }
}

impl_error!(RequestAdapterError);

#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderStateUnconfiguredError;

impl Display for RenderStateUnconfiguredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Render state was not configured before rendering")
    }
}

impl_error!(RenderStateUnconfiguredError);

#[derive(Clone, Copy, Debug)]
pub(crate) struct GUIUnconfiguredError;

impl Display for GUIUnconfiguredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "GUI was not configured before rendering")
    }
}

impl_error!(GUIUnconfiguredError);

#[derive(Clone, Debug)]
pub(crate) enum RenderError {
    Surface(SurfaceError),
    GUIUnconfigured(GUIUnconfiguredError),
}

impl_enum_error_display!(RenderError{ ::Surface ::GUIUnconfigured });
impl_error!(RenderError);

impl_enum_from!(error: SurfaceError -> RenderError::Surface(error));
impl_enum_from!(error: GUIUnconfiguredError -> RenderError::GUIUnconfigured(error));

#[derive(Clone, Debug)]
pub(crate) enum RenderStateError {
    CreateSurface(CreateSurfaceError),
    RequestAdapter(RequestAdapterError),
    RequestDevice(RequestDeviceError),
}

impl_enum_error_display!(RenderStateError{ ::CreateSurface ::RequestAdapter ::RequestDevice });
impl_error!(RenderStateError);

impl_enum_from!(error: CreateSurfaceError -> RenderStateError::CreateSurface(error));
impl_enum_from!(error: RequestAdapterError -> RenderStateError::RequestAdapter(error));
impl_enum_from!(error: RequestDeviceError -> RenderStateError::RequestDevice(error));

#[derive(Debug)]
pub(crate) enum ApplicationError {
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
impl_enum_from!(error: CreateSurfaceError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: RequestAdapterError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: RequestDeviceError -> ApplicationError::RenderState(error.into()));
impl_enum_from!(error: RenderError -> ApplicationError::Render(error));
impl_enum_from!(error: SurfaceError -> ApplicationError::Render(error.into()));
impl_enum_from!(error: GUIUnconfiguredError -> ApplicationError::Render(error.into()));
