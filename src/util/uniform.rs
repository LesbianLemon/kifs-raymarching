use egui_wgpu::wgpu;
use std::ops::Deref;

pub(crate) trait BufferDataDescriptor: Clone + Copy {
    type BufferData: bytemuck::Pod;

    fn into_buffer_data(self) -> Self::BufferData;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UniformBufferDescriptor<'a, Descriptor>
where
    Descriptor: BufferDataDescriptor,
{
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) data_descriptor: Descriptor,
}

#[derive(Clone, Debug)]
pub(crate) struct UniformBuffer(wgpu::Buffer);

impl UniformBuffer {
    pub(crate) fn update_buffer<Descriptor>(
        &mut self,
        queue: &wgpu::Queue,
        new_data_init: Descriptor,
    ) where
        Descriptor: BufferDataDescriptor,
    {
        queue.write_buffer(
            &self.0,
            0,
            bytemuck::cast_slice(&[new_data_init.into_buffer_data()]),
        );
    }
}

// Implicitly implement all methods of wgpu::Buffer on UniformBuffer
impl Deref for UniformBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) trait UniformBufferInit {
    fn create_buffer_init(&self, descriptor: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer;

    fn create_uniform_buffer<Descriptor>(
        &self,
        descriptor: &UniformBufferDescriptor<'_, Descriptor>,
    ) -> UniformBuffer
    where
        Descriptor: BufferDataDescriptor,
    {
        UniformBuffer(self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: descriptor.label,
            contents: bytemuck::cast_slice(&[descriptor.data_descriptor.into_buffer_data()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }))
    }
}

// Implement functionality for foreign type using trait
impl UniformBufferInit for wgpu::Device {
    fn create_buffer_init(&self, descriptor: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
        wgpu::util::DeviceExt::create_buffer_init(self, descriptor)
    }
}
