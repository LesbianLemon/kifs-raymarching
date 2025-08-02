use egui_wgpu::wgpu;
use winit::{dpi::PhysicalSize, window::Window};

use crate::data::{CameraData, SizeData};
use crate::util::math::{PI, Radians, Vector2};
use crate::util::uniform::{UniformBuffer, UniformBufferDescriptor, UniformBufferInit as _};

#[derive(Clone, Debug)]
pub(crate) struct GraphicState {
    size_data: SizeData,
    size_uniform_buffer: UniformBuffer,
    camera_data: CameraData,
    camera_uniform_buffer: UniformBuffer,
    camera_rotatable: bool,
}

impl GraphicState {
    #[must_use]
    pub(crate) fn new(window: &Window, device: &wgpu::Device) -> Self {
        let size_data = window.inner_size().into();
        let size_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("size_uniform_buffer"),
            data: size_data,
        });

        let camera_data = CameraData::default();
        let camera_uniform_buffer = device.create_uniform_buffer(&UniformBufferDescriptor {
            label: Some("camera_uniform_buffer"),
            data: camera_data,
        });
        let camera_rotatable = false;

        Self {
            size_data,
            size_uniform_buffer,
            camera_data,
            camera_uniform_buffer,
            camera_rotatable,
        }
    }

    #[must_use]
    pub(crate) fn size_data(&self) -> SizeData {
        self.size_data
    }

    #[must_use]
    pub(crate) fn size_uniform_buffer(&self) -> &UniformBuffer {
        &self.size_uniform_buffer
    }

    #[must_use]
    pub(crate) fn camera_data(&self) -> CameraData {
        self.camera_data
    }

    #[must_use]
    pub(crate) fn camera_uniform_buffer(&self) -> &UniformBuffer {
        &self.camera_uniform_buffer
    }

    pub(crate) fn update_size(&mut self, queue: &wgpu::Queue, new_size: PhysicalSize<u32>) {
        self.size_data = new_size.into();
        self.size_uniform_buffer
            .update_buffer(queue, self.size_data);
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

    pub(crate) fn enable_camera_rotation(&mut self) {
        self.camera_rotatable = true;
    }

    pub(crate) fn disable_camera_rotation(&mut self) {
        self.camera_rotatable = false;
    }

    #[must_use]
    pub(crate) fn is_camera_rotatable(&self) -> bool {
        self.camera_rotatable
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.draw(0..3, 0..1);
    }
}
