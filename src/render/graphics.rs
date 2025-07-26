use egui_wgpu::wgpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::data::{CameraData, SizeData, uniform::Uniform};
use crate::math::{PI, Radians, Vector2};

pub struct GraphicState {
    size_uniform: Uniform<SizeData>,
    camera_uniform: Uniform<CameraData>,
    camera_rotatable: bool,
}

impl GraphicState {
    pub fn new(window: &Window, device: &wgpu::Device) -> Self {
        let size_uniform =
            Uniform::create_uniform(device, window.inner_size().into(), Some("size_uniform"));
        let camera_uniform = Uniform::create_uniform(
            device,
            CameraData {
                origin_distance: 5.,
                min_distance: 2.,
                ..CameraData::default()
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

    pub fn size_uniform(&self) -> &Uniform<SizeData> {
        &self.size_uniform
    }

    pub fn camera_uniform(&self) -> &Uniform<CameraData> {
        &self.camera_uniform
    }

    pub fn update_size(&mut self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>) {
        self.size_uniform
            .update_uniform(SizeData::from(new_size), queue);
    }

    pub fn zoom_camera(&mut self, queue: &wgpu::Queue, distance: f32) {
        let current_distance = self.camera_uniform.descriptor().origin_distance;
        let min_distance = self.camera_uniform.descriptor().min_distance;

        self.camera_uniform.update_uniform(
            CameraData {
                origin_distance: f32::max(min_distance, current_distance - distance),
                ..self.camera_uniform.descriptor()
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
        let angles = self.camera_uniform.descriptor().angles;
        let Vector2(new_phi, mut new_theta) = angles + Vector2(delta_phi, delta_theta);

        // Limit theta on [-PI/2, PI/2]
        new_theta = new_theta.clamp(-PI / 2., PI / 2.);

        self.camera_uniform.update_uniform(
            CameraData {
                angles: Vector2(new_phi.standardize(), new_theta),
                ..self.camera_uniform.descriptor()
            },
            queue,
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.draw(0..3, 0..1);
    }
}
