use egui_wgpu::wgpu;

pub(crate) trait UniformBufferData: Copy + Clone {
    type PodData: bytemuck::Pod;

    fn into_pod(self) -> Self::PodData;
    fn from_pod(pod_data: Self::PodData) -> Self;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UniformBufferDescriptor<'a, Data>
where
    Data: UniformBufferData,
{
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) data: Data,
}

#[derive(Clone, Debug)]
pub(crate) struct UniformBuffer {
    buffer: wgpu::Buffer,
}

impl UniformBuffer {
    #[must_use]
    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn update_buffer<Data>(&mut self, queue: &wgpu::Queue, new_data: Data)
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

pub(crate) trait UniformBufferInit {
    fn create_buffer_init(&self, descriptor: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer;

    fn create_uniform_buffer<Data>(
        &self,
        descriptor: &UniformBufferDescriptor<'_, Data>,
    ) -> UniformBuffer
    where
        Data: UniformBufferData,
    {
        UniformBuffer {
            buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: descriptor.label,
                contents: bytemuck::cast_slice(&[descriptor.data.into_pod()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }
}

// Implement functionality for foreign type using trait
impl UniformBufferInit for wgpu::Device {
    fn create_buffer_init(&self, descriptor: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
        wgpu::util::DeviceExt::create_buffer_init(self, descriptor)
    }
}
