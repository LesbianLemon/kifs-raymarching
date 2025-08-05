use egui_wgpu::wgpu;
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ResourceGroupLayoutEntry {
    pub(crate) visibility: wgpu::ShaderStages,
    pub(crate) ty: wgpu::BindingType,
    pub(crate) count: Option<NonZeroU32>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceGroupDescriptor<'a> {
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) resources: &'a [wgpu::BindingResource<'a>],
    pub(crate) entries: &'a [ResourceGroupLayoutEntry],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FixedEntryResourceGroupDescriptor<'a> {
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) resources: &'a [wgpu::BindingResource<'a>],
    pub(crate) entry: ResourceGroupLayoutEntry,
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceGroup {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl ResourceGroup {
    #[must_use]
    pub(crate) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[must_use]
    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

// Implement extra functionality for Iterator
trait IteratorMapToVec: Iterator + Sized {
    fn map_to_vec<F, T>(self, f: F) -> Vec<T>
    where
        F: FnMut(Self::Item) -> T,
    {
        self.map(f).collect::<Vec<_>>()
    }

    fn enumerate_map_to_vec<F, T>(self, f: F) -> Vec<T>
    where
        F: FnMut((usize, Self::Item)) -> T,
    {
        self.enumerate().map_to_vec(f)
    }
}

impl<T> IteratorMapToVec for T where T: Iterator {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceGroupLayoutDescriptor<'a> {
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) resources: &'a [wgpu::BindingResource<'a>],
    pub(crate) entries: &'a [ResourceGroupLayoutEntry],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResourceGroupBindDescriptor<'a> {
    pub(crate) label: wgpu::Label<'a>,
    pub(crate) resources: &'a [wgpu::BindingResource<'a>],
    pub(crate) layout: &'a wgpu::BindGroupLayout,
}

pub(crate) trait ResourceGroupInit {
    fn create_resource_group_layout(
        &self,
        descriptor: &ResourceGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout;
    fn create_resource_group_bind(
        &self,
        descriptor: &ResourceGroupBindDescriptor,
    ) -> wgpu::BindGroup;

    fn create_resource_group(&self, descriptor: &ResourceGroupDescriptor) -> ResourceGroup {
        let bind_group_layout = self.create_resource_group_layout(&ResourceGroupLayoutDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_buffer_group_layout"))
                .as_deref(),
            resources: descriptor.resources,
            entries: descriptor.entries,
        });
        let bind_group = self.create_resource_group_bind(&ResourceGroupBindDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_buffer_group_bind"))
                .as_deref(),
            resources: descriptor.resources,
            layout: &bind_group_layout,
        });

        ResourceGroup {
            bind_group_layout,
            bind_group,
        }
    }

    // Cannot use self.create_buffer_group to create the buffer due to ownership problems
    fn create_fixed_entry_resource_group(
        &self,
        descriptor: &FixedEntryResourceGroupDescriptor,
    ) -> ResourceGroup {
        let bind_group_layout = self.create_resource_group_layout(&ResourceGroupLayoutDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_bind_group_layout"))
                .as_deref(),
            resources: descriptor.resources,
            entries: descriptor
                .resources
                .iter()
                .map_to_vec(|_| descriptor.entry)
                .as_slice(),
        });
        let bind_group = self.create_resource_group_bind(&ResourceGroupBindDescriptor {
            label: descriptor
                .label
                .map(|label| format!("{label}_bind_group"))
                .as_deref(),
            resources: descriptor.resources,
            layout: &bind_group_layout,
        });

        ResourceGroup {
            bind_group_layout,
            bind_group,
        }
    }
}

// Implement functionality for foreign type using trait
impl ResourceGroupInit for wgpu::Device {
    fn create_resource_group_layout(
        &self,
        descriptor: &ResourceGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: descriptor.label,
            entries: descriptor
                .resources
                .iter()
                .enumerate_map_to_vec(|(i, _resource)| wgpu::BindGroupLayoutEntry {
                    #[allow(clippy::cast_possible_truncation)]
                    binding: i as u32,
                    visibility: descriptor.entries[i].visibility,
                    ty: descriptor.entries[i].ty,
                    count: descriptor.entries[i].count,
                })
                .as_slice(),
        })
    }

    fn create_resource_group_bind(
        &self,
        descriptor: &ResourceGroupBindDescriptor,
    ) -> wgpu::BindGroup {
        self.create_bind_group(&wgpu::BindGroupDescriptor {
            label: descriptor.label,
            layout: descriptor.layout,
            entries: descriptor
                .resources
                .iter()
                .enumerate_map_to_vec(|(i, resource)| wgpu::BindGroupEntry {
                    #[allow(clippy::cast_possible_truncation)]
                    binding: i as u32,
                    // Clone is necessary and non-problematic, since wgpu::BindingResource is small
                    resource: (*resource).clone(),
                })
                .as_slice(),
        })
    }
}
