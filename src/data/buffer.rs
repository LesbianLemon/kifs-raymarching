use egui_wgpu::wgpu;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferGroupLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZeroU32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferGroupDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub entries: &'a [BufferGroupLayoutEntry],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FixedEntryBufferGroupDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub entry: BufferGroupLayoutEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferGroup {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl BufferGroup {
    #[must_use]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[must_use]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

// Implement extra functionality for Iterator
trait IteratorMapToVec {
    fn map_to_vec<T, F>(self, f: F) -> Vec<T>
    where
        Self: Iterator + Sized,
        F: FnMut(Self::Item) -> T,
    {
        self.map(f).collect::<Vec<_>>()
    }
}

impl<T> IteratorMapToVec for T where T: Iterator {}

// Implement extra functionality for Iterator
trait IteratorEnumerateMapToVec {
    fn enumerate_map_to_vec<T, F>(self, f: F) -> Vec<T>
    where
        Self: Iterator + Sized,
        F: FnMut((usize, Self::Item)) -> T,
    {
        self.enumerate().map(f).collect::<Vec<_>>()
    }
}

impl<T> IteratorEnumerateMapToVec for T where T: Iterator {}

pub struct BufferGroupLayoutDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub entries: &'a [BufferGroupLayoutEntry],
}

pub struct BufferGroupBindDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub buffers: &'a [&'a wgpu::Buffer],
    pub layout: &'a wgpu::BindGroupLayout,
}

pub trait BufferGroupInit {
    fn create_buffer_group_layout(
        &self,
        descriptor: &BufferGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout;
    fn create_buffer_group_bind(&self, descriptor: &BufferGroupBindDescriptor) -> wgpu::BindGroup;

    fn create_buffer_group(&self, descriptor: &BufferGroupDescriptor) -> BufferGroup {
        let bind_group_layout = self.create_buffer_group_layout(&BufferGroupLayoutDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_buffer_group_layout"))
                .as_deref(),
            buffers: descriptor.buffers,
            entries: descriptor.entries,
        });
        let bind_group = self.create_buffer_group_bind(&BufferGroupBindDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_buffer_group_bind"))
                .as_deref(),
            buffers: descriptor.buffers,
            layout: &bind_group_layout,
        });

        BufferGroup {
            bind_group_layout,
            bind_group,
        }
    }

    // Cannot use self.create_buffer_group to create the buffer due to ownership problems
    fn create_fixed_entry_buffer_group(
        &self,
        descriptor: &FixedEntryBufferGroupDescriptor,
    ) -> BufferGroup {
        let bind_group_layout = self.create_buffer_group_layout(&BufferGroupLayoutDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_bind_group_layout"))
                .as_deref(),
            buffers: descriptor.buffers,
            entries: descriptor
                .buffers
                .iter()
                .map_to_vec(|_| descriptor.entry)
                .as_slice(),
        });
        let bind_group = self.create_buffer_group_bind(&BufferGroupBindDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_bind_group"))
                .as_deref(),
            buffers: descriptor.buffers,
            layout: &bind_group_layout,
        });

        BufferGroup {
            bind_group_layout,
            bind_group,
        }
    }
}

// Implement functionality for foreign type using trait
impl BufferGroupInit for wgpu::Device {
    fn create_buffer_group_layout(
        &self,
        descriptor: &BufferGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: descriptor.label,
            entries: descriptor
                .buffers
                .iter()
                .enumerate_map_to_vec(|(i, _)| wgpu::BindGroupLayoutEntry {
                    #[allow(clippy::cast_possible_truncation)]
                    binding: i as u32,
                    visibility: descriptor.entries[i].visibility,
                    ty: descriptor.entries[i].ty,
                    count: descriptor.entries[i].count,
                })
                .as_slice(),
        })
    }

    fn create_buffer_group_bind(&self, descriptor: &BufferGroupBindDescriptor) -> wgpu::BindGroup {
        self.create_bind_group(&wgpu::BindGroupDescriptor {
            label: descriptor.label,
            layout: descriptor.layout,
            entries: descriptor
                .buffers
                .iter()
                .enumerate_map_to_vec(|(i, buffer)| wgpu::BindGroupEntry {
                    #[allow(clippy::cast_possible_truncation)]
                    binding: i as u32,
                    resource: buffer.as_entire_binding(),
                })
                .as_slice(),
        })
    }
}
