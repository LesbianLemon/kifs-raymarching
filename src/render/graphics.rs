use egui_wgpu::wgpu;

use winit::{dpi::PhysicalSize, window::Window};

use crate::math::{PI, Radians, Vector2};
use crate::uniform::{
    CameraUniformData, CameraUniformDataDescriptor, SizeUniformData, SizeUniformDataDescriptor,
    Uniform,
};

pub struct GraphicState {
    size_uniform: Uniform<SizeUniformData>,
    camera_uniform: Uniform<CameraUniformData>,
    camera_rotatable: bool,
    render_pipeline: wgpu::RenderPipeline,
}

impl GraphicState {
    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
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

        let size_uniform = Uniform::<SizeUniformData>::create_uniform(
            device,
            SizeUniformDataDescriptor::from(size),
            Some("size_uniform"),
        );
        let camera_uniform = Uniform::<CameraUniformData>::create_uniform(
            device,
            CameraUniformDataDescriptor {
                origin_distance: 15.,
                min_distance: 5.,
                angles: Vector2(Radians::from_degrees(-45.), Radians::from_degrees(20.)),
            },
            Some("camera_uniform"),
        );
        let camera_rotatable = false;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("raymarching_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });

        let render_pipeline = Self::create_render_pipeline(
            device,
            &[
                size_uniform.bind_group_layout(),
                camera_uniform.bind_group_layout(),
            ],
            config,
            &shader,
        );

        Self {
            size_uniform,
            camera_uniform,
            camera_rotatable,
            render_pipeline,
        }
    }

    pub fn update_size(&mut self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>) {
        self.size_uniform
            .update_uniform(SizeUniformDataDescriptor::from(new_size), queue);
    }

    pub fn zoom_camera(&mut self, queue: &wgpu::Queue, distance: f32) {
        let current_distance = self.camera_uniform.origin_distance();
        let min_distance = self.camera_uniform.min_distance();

        self.camera_uniform.update_uniform(
            CameraUniformDataDescriptor {
                origin_distance: f32::max(min_distance, current_distance - distance),
                ..self.camera_uniform.data_descriptor()
            },
            queue,
        );
    }

    pub fn enable_camera_rotation(&mut self) {
        self.camera_rotatable = true;
    }

    pub fn disable_camera_rotation(&mut self) {
        self.camera_rotatable = false;
    }

    pub fn is_camera_rotatable(&self) -> bool {
        self.camera_rotatable
    }

    pub fn rotate_camera(&mut self, queue: &wgpu::Queue, delta_phi: Radians, delta_theta: Radians) {
        let angles = self.camera_uniform.angles();
        let Vector2(new_phi, mut new_theta) = angles + Vector2(delta_phi, delta_theta);

        // Limit theta on [-PI/2, PI/2]
        new_theta = new_theta.clamp(-PI / 2., PI / 2.);

        self.camera_uniform.update_uniform(
            CameraUniformDataDescriptor {
                angles: Vector2(new_phi.standardize(), new_theta),
                ..self.camera_uniform.data_descriptor()
            },
            queue,
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.size_uniform.bind_group(), &[]);
        render_pass.set_bind_group(1, self.camera_uniform.bind_group(), &[]);

        render_pass.draw(0..3, 0..1);
    }
}
