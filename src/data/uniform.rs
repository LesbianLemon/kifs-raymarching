use egui_wgpu::wgpu::{self, util::DeviceExt as _};

pub trait UniformData: bytemuck::Pod {}

pub trait UniformDataDescriptor: Copy {
    type Data: UniformData;

    fn into_uniform_data(self) -> Self::Data;
    fn from_uniform_data(data: Self::Data) -> Self;
}

#[derive(Debug)]
pub struct Uniform<Descriptor: UniformDataDescriptor> {
    descriptor: Descriptor,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl<Descriptor: UniformDataDescriptor> Uniform<Descriptor> {
    pub fn descriptor(&self) -> Descriptor {
        self.descriptor
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<Descriptor: UniformDataDescriptor> Uniform<Descriptor> {
    pub fn create_uniform_buffer(
        device: &wgpu::Device,
        data: Descriptor::Data,
        label: Option<&str>,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn create_uniform_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: label
                .map(|label| format!("{label}_bind_group_layout"))
                .as_deref(),
        })
    }

    pub fn create_uniform_bind_group(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
        bind_group_layout: &wgpu::BindGroupLayout,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: label.map(|label| format!("{label}_bind_group")).as_deref(),
        })
    }

    pub fn create_uniform(
        device: &wgpu::Device,
        descriptor: Descriptor,
        label: Option<&str>,
    ) -> Self {
        let buffer = Self::create_uniform_buffer(device, descriptor.into_uniform_data(), label);
        let bind_group_layout = Self::create_uniform_bind_group_layout(device, label);
        let bind_group =
            Self::create_uniform_bind_group(device, &buffer, &bind_group_layout, label);

        Self {
            descriptor,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_uniform(&mut self, descriptor: Descriptor, queue: &wgpu::Queue) {
        self.descriptor = descriptor;

        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.descriptor.into_uniform_data()]),
        );
    }
}
