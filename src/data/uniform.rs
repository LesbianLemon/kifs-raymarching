use egui_wgpu::wgpu::{self, util::DeviceExt as _};

pub trait UniformBufferData: Copy + Clone {
    type PodData: bytemuck::Pod;

    fn into_pod(self) -> Self::PodData;
    fn from_pod(pod_data: Self::PodData) -> Self;
}

pub struct UniformBufferDescriptor<'a, Data>
where
    Data: UniformBufferData,
{
    pub label: wgpu::Label<'a>,
    pub data: Data,
}

#[derive(Clone, Debug)]
pub struct UniformBuffer<'a> {
    label: wgpu::Label<'a>,
    buffer: wgpu::Buffer,
}

impl<'a> UniformBuffer<'a> {
    pub fn label(&self) -> wgpu::Label<'a> {
        self.label
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn update_buffer<Data>(&mut self, queue: &wgpu::Queue, new_data: Data)
    where
        Data: UniformBufferData,
    {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[new_data.into_pod()]),
        );
    }
}

pub trait UniformBufferInit {
    fn create_uniform_buffer<'a, Data>(
        &self,
        descriptor: &UniformBufferDescriptor<'a, Data>,
    ) -> UniformBuffer<'a>
    where
        Data: UniformBufferData;
}

// implement functionality for foreign type using trait
impl UniformBufferInit for wgpu::Device {
    fn create_uniform_buffer<'a, Data>(
        &self,
        descriptor: &UniformBufferDescriptor<'a, Data>,
    ) -> UniformBuffer<'a>
    where
        Data: UniformBufferData,
    {
        UniformBuffer {
            label: descriptor.label,
            // data: descriptor.data,
            buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: descriptor.label,
                contents: bytemuck::cast_slice(&[descriptor.data.into_pod()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}
