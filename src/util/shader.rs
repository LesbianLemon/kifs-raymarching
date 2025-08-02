use egui_wgpu::wgpu;
use std::{
    borrow::Cow,
    iter::{self, Sum},
    ops::{Add, Deref},
};

#[derive(Clone, Debug, Default)]
pub(crate) struct WGSLShaderSource<'a>(pub(crate) Cow<'a, str>);

impl<'a> From<WGSLShaderSource<'a>> for wgpu::ShaderSource<'a> {
    fn from(value: WGSLShaderSource<'a>) -> Self {
        Self::Wgsl(value.0)
    }
}

// Implicitly implement all methods of Cow<'a, str> on WGSLShaderSource<'a>
impl<'a> Deref for WGSLShaderSource<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Add<Self> for WGSLShaderSource<'a> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // Cannot avoid cloning, since Cow<'a, str> does not implement Copy
        WGSLShaderSource((*self).clone() + (*rhs).clone())
    }
}

impl<'a> Sum<Self> for WGSLShaderSource<'a> {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self::default(), Add::add)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WGSLShaderModuleDescriptor<'a> {
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) main: WGSLShaderSource<'a>,
    pub(crate) dependencies: &'a [WGSLShaderSource<'a>],
}

#[derive(Clone, Debug)]
pub(crate) struct WGSLShaderModule(wgpu::ShaderModule);

// Implicitly implement all methods of wgpu::ShaderModule on WGSLShaderModule
impl Deref for WGSLShaderModule {
    type Target = wgpu::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) trait WGSLShaderModuleInit {
    fn create_shader_module(&self, descriptor: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule;

    fn create_wgsl_shader_module(&self, descriptor: WGSLShaderModuleDescriptor) -> WGSLShaderModule {
        WGSLShaderModule(
            self.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: descriptor.label,
                source: descriptor
                    .dependencies
                    .iter()
                    // Cannot avoid cloning, since Cow<'a, str> does not implement Copy
                    .map(|wgsl_source| (*wgsl_source).clone())
                    .chain(iter::once(descriptor.main))
                    .sum::<WGSLShaderSource<'_>>()
                    .into(),
            }),
        )
    }
}

impl WGSLShaderModuleInit for wgpu::Device {
    fn create_shader_module(&self, descriptor: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.create_shader_module(descriptor)
    }
}