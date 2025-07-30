use egui_wgpu::wgpu;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferGroupLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZeroU32>,
}

pub struct BufferGroupDescriptor<'a, 'b> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'b [&'b wgpu::Buffer],
    pub layout_entry: BufferGroupLayoutEntry,
}

pub struct BufferGroup<'a> {
    label: wgpu::Label<'a>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl<'a> BufferGroup<'a> {
    pub fn label(&self) -> wgpu::Label<'a> {
        self.label
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct BufferGroupBindGroupLayoutDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub layout_entry: BufferGroupLayoutEntry,
}

pub struct BufferGroupBindGroupDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub layout: &'a wgpu::BindGroupLayout,
}

pub trait BufferGroupInit {
    fn create_buffer_group_bind_group_layout(
        &self,
        descriptor: &BufferGroupBindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout;
    fn create_buffer_group_bind_group(
        &self,
        descriptor: &BufferGroupBindGroupDescriptor,
    ) -> wgpu::BindGroup;

    fn create_buffer_group<'a>(
        &self,
        descriptor: &BufferGroupDescriptor<'a, '_>,
    ) -> BufferGroup<'a> {
        let label = descriptor.label;
        let bind_group_layout =
            self.create_buffer_group_bind_group_layout(&BufferGroupBindGroupLayoutDescriptor {
                label: label
                    .map(|label| format!("{label}_bind_group_layout"))
                    .as_deref(),
                buffers: descriptor.buffers,
                layout_entry: descriptor.layout_entry,
            });
        let bind_group = self.create_buffer_group_bind_group(&BufferGroupBindGroupDescriptor {
            label: label.map(|label| format!("{label}_bind_group")).as_deref(),
            buffers: descriptor.buffers,
            layout: &bind_group_layout,
        });

        BufferGroup {
            label,
            bind_group_layout,
            bind_group,
        }
    }
}

// implement functionality for foreign type using trait
impl BufferGroupInit for wgpu::Device {
    fn create_buffer_group_bind_group_layout(
        &self,
        descriptor: &BufferGroupBindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: descriptor.label,
            entries: descriptor
                .buffers
                .iter()
                .zip(0u32..)
                .map(|(_, i)| wgpu::BindGroupLayoutEntry {
                    binding: i,
                    visibility: descriptor.layout_entry.visibility,
                    ty: descriptor.layout_entry.ty,
                    count: descriptor.layout_entry.count,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        })
    }

    fn create_buffer_group_bind_group(
        &self,
        descriptor: &BufferGroupBindGroupDescriptor,
    ) -> wgpu::BindGroup {
        self.create_bind_group(&wgpu::BindGroupDescriptor {
            label: descriptor.label,
            layout: descriptor.layout,
            entries: descriptor
                .buffers
                .iter()
                .zip(0u32..)
                .map(|(buffer, i)| wgpu::BindGroupEntry {
                    binding: i,
                    resource: buffer.as_entire_binding(),
                })
                .collect::<Vec<_>>()
                .as_slice(),
        })
    }
}
