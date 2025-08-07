use egui_wgpu::wgpu;
use winit::window::Window;

use crate::data::scene::FractalGroup;
use crate::data::{CameraData, GuiData, OptionsData, ScreenData};
use crate::util::buffer::{
    FixedEntryResourceGroupDescriptor, ResourceGroup, ResourceGroupInit as _,
    ResourceGroupLayoutEntry,
};
use crate::util::math::{PI, Radians, Vector2};
use crate::util::shader::{
    WGSLShaderModuleDescriptor, WGSLShaderModuleInit as _, WGSLShaderSource,
};
use crate::util::uniform::{UniformBuffer, UniformBufferDescriptor, UniformBufferInit as _};

macro_rules! shader_source {
    ($path:expr $(,)?) => {
        WGSLShaderSource(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/shaders/", $path)).into(),
        )
    };
}

#[derive(Clone, Debug)]
pub(crate) struct GraphicState {
    screen_data: ScreenData,
    screen_uniform_buffer: UniformBuffer,
    camera_data: CameraData,
    camera_uniform_buffer: UniformBuffer,
    camera_rotatable: bool,
    options_data: OptionsData,
    options_uniform_buffer: UniformBuffer,
    uniform_group: ResourceGroup,
    kifs_pipeline: wgpu::RenderPipeline,
    julia_pipeline: wgpu::RenderPipeline,
    generalized_julia_pipeline: wgpu::RenderPipeline,
}

impl GraphicState {
    #[must_use]
    fn create_uniform_group(
        device: &wgpu::Device,
        resources: &[wgpu::BindingResource],
    ) -> ResourceGroup {
        device.create_fixed_entry_resource_group(&FixedEntryResourceGroupDescriptor {
            label: Some("uniform_buffer_group"),
            resources,
            entry: ResourceGroupLayoutEntry {
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        })
    }

    #[must_use]
    fn create_render_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
        shader: &wgpu::ShaderModule,
        label: wgpu::Label,
    ) -> wgpu::RenderPipeline {
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
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

    #[must_use]
    fn create_pipelines(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        config: &wgpu::SurfaceConfiguration,
    ) -> (
        wgpu::RenderPipeline,
        wgpu::RenderPipeline,
        wgpu::RenderPipeline,
    ) {
        let kifs_shader = device.create_wgsl_shader_module(WGSLShaderModuleDescriptor {
            label: Some("kifs_shader"),
            main: shader_source!("kifs.wgsl"),
            dependencies: &[
                shader_source!("dependencies/bindings.wgsl"),
                shader_source!("dependencies/entry.wgsl"),
                shader_source!("dependencies/quaternions.wgsl"),
            ],
        });
        let kifs_pipeline = Self::create_render_pipeline(
            device,
            bind_group_layouts,
            config,
            &kifs_shader,
            Some("kifs_render_pipeline"),
        );

        let julia_shader = device.create_wgsl_shader_module(WGSLShaderModuleDescriptor {
            label: Some("julia_shader"),
            main: shader_source!("julia.wgsl"),
            dependencies: &[
                shader_source!("dependencies/bindings.wgsl"),
                shader_source!("dependencies/entry.wgsl"),
                shader_source!("dependencies/quaternions.wgsl"),
            ],
        });
        let julia_pipeline = Self::create_render_pipeline(
            device,
            bind_group_layouts,
            config,
            &julia_shader,
            Some("julia_render_pipeline"),
        );

        let generalized_julia_shader =
            device.create_wgsl_shader_module(WGSLShaderModuleDescriptor {
                label: Some("generalized_julia_shader"),
                main: shader_source!("gen_julia.wgsl"),
                dependencies: &[
                    shader_source!("dependencies/bindings.wgsl"),
                    shader_source!("dependencies/entry.wgsl"),
                    shader_source!("dependencies/quaternions.wgsl"),
                ],
            });
        let generalized_julia_pipeline = Self::create_render_pipeline(
            device,
            bind_group_layouts,
            config,
            &generalized_julia_shader,
            Some("generalized_julia_render_pipeline"),
        );

        (kifs_pipeline, julia_pipeline, generalized_julia_pipeline)
    }

    #[must_use]
    pub(crate) fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let screen_data = window.inner_size().into();
        let screen_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("size_uniform_buffer"),
            data_descriptor: screen_data,
        });

        let camera_data = CameraData::default();
        let camera_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("camera_uniform_buffer"),
            data_descriptor: camera_data,
        });
        let camera_rotatable = false;

        let options_data = GuiData::default().into();
        let options_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("gui_uniform_buffer"),
            data_descriptor: options_data,
        });

        let uniform_group = Self::create_uniform_group(
            device,
            &[
                screen_uniform_buffer.as_entire_binding(),
                camera_uniform_buffer.as_entire_binding(),
                options_uniform_buffer.as_entire_binding(),
            ],
        );

        let (kifs_pipeline, julia_pipeline, generalized_julia_pipeline) =
            Self::create_pipelines(device, &[uniform_group.bind_group_layout()], config);

        Self {
            screen_data,
            screen_uniform_buffer,
            camera_data,
            camera_uniform_buffer,
            camera_rotatable,
            options_data,
            options_uniform_buffer,
            uniform_group,
            kifs_pipeline,
            julia_pipeline,
            generalized_julia_pipeline,
        }
    }

    #[must_use]
    pub(crate) fn screen_data(&self) -> ScreenData {
        self.screen_data
    }

    #[must_use]
    pub(crate) fn camera_data(&self) -> CameraData {
        self.camera_data
    }

    #[must_use]
    pub(crate) fn is_camera_rotatable(&self) -> bool {
        self.camera_rotatable
    }

    pub(crate) fn enable_camera_rotation(&mut self) {
        self.camera_rotatable = true;
    }

    pub(crate) fn disable_camera_rotation(&mut self) {
        self.camera_rotatable = false;
    }

    #[must_use]
    pub(crate) fn options_data(&self) -> OptionsData {
        self.options_data
    }

    pub(crate) fn update_screen_data(&mut self, queue: &wgpu::Queue, new_screen_data: ScreenData) {
        self.screen_data = new_screen_data;
        self.screen_uniform_buffer
            .update_buffer(queue, self.screen_data);
    }

    pub(crate) fn zoom_camera(&mut self, queue: &wgpu::Queue, distance: f32) {
        let current_distance = self.camera_data.origin_distance;
        let min_distance = self.camera_data.min_distance;

        self.camera_data = CameraData {
            origin_distance: f32::max(min_distance, current_distance - distance),
            ..self.camera_data
        };
        self.camera_uniform_buffer
            .update_buffer(queue, self.camera_data);
    }

    pub(crate) fn rotate_camera(
        &mut self,
        queue: &wgpu::Queue,
        delta_phi: Radians,
        delta_theta: Radians,
    ) {
        if !self.camera_rotatable {
            return;
        }

        let angles = self.camera_data.angles;
        let Vector2(new_phi, mut new_theta) = angles + Vector2(delta_phi, delta_theta);

        // Limit theta on [-PI/2, PI/2]
        new_theta = new_theta.clamp(-PI / 2., PI / 2.);

        self.camera_data = CameraData {
            angles: Vector2(new_phi.standardize(), new_theta),
            ..self.camera_data
        };
        self.camera_uniform_buffer
            .update_buffer(queue, self.camera_data);
    }

    pub(crate) fn update_options(&mut self, queue: &wgpu::Queue, new_options_data: OptionsData) {
        self.options_data = new_options_data;
        self.options_uniform_buffer
            .update_buffer(queue, self.options_data);
    }

    pub(crate) fn render(&self, render_pass: &mut wgpu::RenderPass) {
        match self.options_data.fractal_group {
            FractalGroup::KaleidoscopicIFS => {
                render_pass.set_pipeline(&self.kifs_pipeline);
            }
            FractalGroup::JuliaSet => {
                render_pass.set_pipeline(&self.julia_pipeline);
            }
            FractalGroup::GeneralizedJuliaSet => {
                render_pass.set_pipeline(&self.generalized_julia_pipeline);
            }
        }
        render_pass.set_bind_group(0, self.uniform_group.bind_group(), &[]);

        render_pass.draw(0..3, 0..2);
    }
}
