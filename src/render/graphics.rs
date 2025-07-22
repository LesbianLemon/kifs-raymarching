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
}

impl GraphicState {
    pub fn new(window: &Window, device: &wgpu::Device) -> Self {
        let size = window.inner_size();

        let size_uniform = Uniform::<SizeUniformData>::create_uniform(
            device,
            SizeUniformDataDescriptor::from(size),
            Some("size_uniform"),
        );
        let camera_uniform = Uniform::<CameraUniformData>::create_uniform(
            device,
            CameraUniformDataDescriptor {
                origin_distance: 5.,
                min_distance: 2.,
                angles: Vector2(Radians::from_degrees(-45.), Radians::from_degrees(20.)),
            },
            Some("camera_uniform"),
        );
        let camera_rotatable = false;

        Self {
            size_uniform,
            camera_uniform,
            camera_rotatable,
        }
    }

    pub fn size_uniform(&self) -> &Uniform<SizeUniformData> {
        &self.size_uniform
    }

    pub fn camera_uniform(&self) -> &Uniform<CameraUniformData> {
        &self.camera_uniform
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
        render_pass.draw(0..3, 0..1);
    }
}
