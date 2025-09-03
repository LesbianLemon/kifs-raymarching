use winit::dpi::PhysicalSize;

use crate::{
    data::packed::LinearRgb,
    util::{
        math::{Matrix3x3, Radians, Vector2, Vector3, Vector4},
        uniform::BufferDataDescriptor,
    },
};

pub(crate) mod packed;
pub(crate) mod scene;

use packed::{IntoPacked, Vector3Packed, Vector4Packed};
use scene::{FractalGroup, PrimitiveShape};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ScreenUniformData {
    width: f32,
    height: f32,
    aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniformData {
    origin: Vector3Packed<f32>,
    _padding: [u32; 1],
    matrix: Vector3Packed<Vector4Packed<f32>>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct OptionsUniformData {
    max_iterations: i32,
    max_distance: f32,
    epsilon: f32,
    _padding1: u32,
    fractal_color: Vector3Packed<f32>,
    _padding2: u32,
    background_color: Vector3Packed<f32>,
    fractal_group_id: u32,
    primitive_id: u32,
    power: f32,
    _padding3: [u32; 2],
    constant: Vector4Packed<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ScreenData {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl From<PhysicalSize<u32>> for ScreenData {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl BufferDataDescriptor for ScreenData {
    type BufferData = ScreenUniformData;

    fn into_buffer_data(self) -> Self::BufferData {
        #[allow(clippy::cast_precision_loss)]
        let width = self.width as f32;
        #[allow(clippy::cast_precision_loss)]
        let height = self.height as f32;

        Self::BufferData {
            width,
            height,
            aspect_ratio: width / height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CameraData {
    pub(crate) origin_distance: f32,
    pub(crate) min_distance: f32,
    pub(crate) angles: Vector2<Radians>,
}

impl CameraData {
    pub(crate) fn camera_matrix(&self) -> Matrix3x3<f32> {
        let Vector2(phi, theta) = self.angles;

        let phi_rotation = Matrix3x3::rotation_matrix_z(phi);
        let theta_rotation = Matrix3x3::rotation_matrix_y(-theta);

        phi_rotation * theta_rotation
    }

    pub(crate) fn transform_vector(&self, vector: Vector3<f32>) -> Vector3<f32> {
        self.camera_matrix() * vector
    }
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            origin_distance: 5.,
            min_distance: 2.,
            angles: Vector2(Radians::from_radians(0.), Radians::from_radians(0.)),
        }
    }
}

impl BufferDataDescriptor for CameraData {
    type BufferData = CameraUniformData;

    fn into_buffer_data(self) -> Self::BufferData {
        let camera_matrix = self.camera_matrix();
        let origin = self.origin_distance * camera_matrix * Vector3(1., 0., 0.);

        Self::BufferData {
            // Construct origin via spherical coordinates
            origin: origin.into_packed(),
            matrix: camera_matrix.into_packed(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GuiData {
    pub(crate) max_iterations: u32,
    pub(crate) max_distance: f32,
    pub(crate) epsilon: f32,
    pub(crate) fractal_color: [u8; 3],
    pub(crate) background_color: [u8; 3],
    pub(crate) fractal_group: FractalGroup,
    pub(crate) primitive_shape: PrimitiveShape,
    pub(crate) power: f32,
    pub(crate) constant: Vector4<f32>,
}

impl Default for GuiData {
    fn default() -> Self {
        Self {
            max_iterations: 256,
            max_distance: 1000.,
            epsilon: 0.0001,
            fractal_color: [200; 3],
            background_color: [0; 3],
            fractal_group: FractalGroup::default(),
            primitive_shape: PrimitiveShape::default(),
            power: 2.,
            constant: Vector4(-0.1, 0.6, 0.9, -0.3),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct OptionsData {
    pub(crate) max_iterations: u32,
    pub(crate) max_distance: f32,
    pub(crate) epsilon: f32,
    pub(crate) fractal_color: LinearRgb,
    pub(crate) background_color: LinearRgb,
    pub(crate) fractal_group: FractalGroup,
    pub(crate) primitive_shape: PrimitiveShape,
    pub(crate) power: f32,
    pub(crate) constant: Vector4<f32>,
}

impl BufferDataDescriptor for OptionsData {
    type BufferData = OptionsUniformData;

    fn into_buffer_data(self) -> Self::BufferData {
        Self::BufferData {
            #[allow(clippy::cast_possible_wrap)]
            max_iterations: self.max_iterations as i32,
            max_distance: self.max_distance,
            epsilon: self.epsilon,
            fractal_color: self.fractal_color.into_packed(),
            background_color: self.background_color.into_packed(),
            fractal_group_id: self.fractal_group.id(),
            primitive_id: self.primitive_shape.id(),
            power: self.power,
            constant: self.constant.into_packed(),
            ..Default::default()
        }
    }
}

impl From<GuiData> for OptionsData {
    fn from(gui_data: GuiData) -> Self {
        Self {
            max_iterations: gui_data.max_iterations,
            max_distance: gui_data.max_distance,
            epsilon: gui_data.epsilon,
            fractal_color: LinearRgb::from_srgb(
                gui_data.fractal_color[0],
                gui_data.fractal_color[1],
                gui_data.fractal_color[2],
            ),
            background_color: LinearRgb::from_srgb(
                gui_data.background_color[0],
                gui_data.background_color[1],
                gui_data.background_color[2],
            ),
            fractal_group: gui_data.fractal_group,
            primitive_shape: gui_data.primitive_shape,
            power: gui_data.power,
            constant: gui_data.constant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::math::{PI, Vector3};

    #[test]
    fn test_camera_matrix() {
        let camera_data = CameraData {
            origin_distance: 0.,
            min_distance: 0.,
            angles: Vector2(Radians::from_radians(PI), Radians::from_radians(PI)),
        };

        assert_eq!(
            camera_data.camera_matrix(),
            Matrix3x3::from_columns(
                Vector3(1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., -1.)
            )
        );
    }
}
