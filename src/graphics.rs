use egui_wgpu::wgpu;

use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SizeUniform {
    width: u32,
    height: u32,
}

impl SizeUniform {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub struct GraphicState {
    size_uniform: SizeUniform,
    size_uniform_buffer: wgpu::Buffer,
    size_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl GraphicState {
    fn create_size_uniform_buffer(
        device: &wgpu::Device,
        size: PhysicalSize<u32>,
    ) -> (SizeUniform, wgpu::Buffer) {
        let size_uniform = SizeUniform::new(size.width, size.height);

        let size_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Size Uniform Buffer"),
            contents: bytemuck::cast_slice(&[size_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        (size_uniform, size_uniform_buffer)
    }

    fn create_size_uniform_bind(
        device: &wgpu::Device,
        size_uniform_buffer: &wgpu::Buffer,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let size_bind_group_layout =
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
                label: Some("size_bind_group_layout"),
            });

        let size_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &size_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: size_uniform_buffer.as_entire_binding(),
            }],
            label: Some("size_bind_group"),
        });

        (size_bind_group_layout, size_bind_group)
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),

                polygon_mode: wgpu::PolygonMode::Fill,

                unclipped_depth: false,

                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = window.inner_size();

        let (size_uniform, size_uniform_buffer) = Self::create_size_uniform_buffer(device, size);
        let (size_bind_group_layout, size_bind_group) =
            Self::create_size_uniform_bind(device, &size_uniform_buffer);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline =
            Self::create_render_pipeline(device, &[&size_bind_group_layout], config, &shader);

        Self {
            size_uniform,
            size_uniform_buffer,
            size_bind_group,
            render_pipeline,
        }
    }

    pub fn size_uniform(&self) -> (&SizeUniform, &wgpu::Buffer, &wgpu::BindGroup) {
        (
            &self.size_uniform,
            &self.size_uniform_buffer,
            &self.size_bind_group,
        )
    }

    pub fn update_size_uniform(&self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>) {
        let size_uniform = SizeUniform::new(new_size.width, new_size.height);
        queue.write_buffer(
            &self.size_uniform_buffer,
            0,
            bytemuck::cast_slice(&[size_uniform]),
        );
    }

    pub fn prepare_render_pass(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.size_bind_group, &[]);
    }
}
