use egui::{Color32, Rgba};
use packed::{
    IntoPacked as _, IntoUnpacked, Matrix3x3F32Packed, RadiansPacked, Vector2Packed, Vector4Packed,
};
use scene::PrimitiveShape;
use uniform::{UniformData, UniformDataDescriptor};
use winit::dpi::PhysicalSize;

use crate::math::{Matrix3x3, Radians, Vector2};

pub mod packed;
pub mod scene;
pub mod uniform;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SizeUniformData {
    width: u32,
    height: u32,
}

impl UniformData for SizeUniformData {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniformData {
    origin_distance: f32,
    min_distance: f32,
    angles: Vector2Packed<RadiansPacked>,
    matrix: Matrix3x3F32Packed,
}

impl UniformData for CameraUniformData {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GuiUniformData {
    fractal_color: Vector4Packed<f32>,
    background_color: Vector4Packed<f32>,
    primitive_id: u32,
    _padding: [u32; 3],
}

impl UniformData for GuiUniformData {}

#[derive(Copy, Clone, Debug, Default)]
pub struct SizeData {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for SizeData {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

impl UniformDataDescriptor for SizeData {
    type Data = SizeUniformData;

    fn into_uniform_data(self) -> SizeUniformData {
        SizeUniformData {
            width: self.width,
            height: self.height,
        }
    }

    fn from_uniform_data(data: SizeUniformData) -> Self {
        Self {
            width: data.width,
            height: data.height,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CameraData {
    pub origin_distance: f32,
    pub min_distance: f32,
    pub angles: Vector2<Radians>,
}

impl CameraData {
    fn camera_matrix(&self) -> Matrix3x3<f32> {
        let Vector2(phi, theta) = self.angles;

        let phi_rotation = Matrix3x3::rotation_matrix_z(phi);
        let theta_rotation = Matrix3x3::rotation_matrix_y(-theta);

        phi_rotation * theta_rotation
    }
}

impl UniformDataDescriptor for CameraData {
    type Data = CameraUniformData;

    fn into_uniform_data(self) -> CameraUniformData {
        CameraUniformData {
            origin_distance: self.origin_distance,
            min_distance: self.min_distance,
            angles: self.angles.into_packed(),
            matrix: self.camera_matrix().into_packed(),
        }
    }

    fn from_uniform_data(data: CameraUniformData) -> Self {
        Self {
            origin_distance: data.origin_distance,
            min_distance: data.min_distance,
            angles: data.angles.into_unpacked(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GuiData {
    pub fractal_color: Color32,
    pub background_color: Color32,
    pub primitive_shape: PrimitiveShape,
}

impl Default for GuiData {
    fn default() -> Self {
        Self {
            fractal_color: Color32::from_rgb(200, 200, 200),
            background_color: Color32::from_rgb(0, 0, 0),
            primitive_shape: PrimitiveShape::default(),
        }
    }
}

impl UniformDataDescriptor for GuiData {
    type Data = GuiUniformData;

    fn into_uniform_data(self) -> GuiUniformData {
        GuiUniformData {
            fractal_color: Rgba::from(self.fractal_color).into_packed(),
            background_color: Rgba::from(self.background_color).into_packed(),
            primitive_id: self.primitive_shape.id(),
            _padding: [0, 0, 0],
        }
    }

    fn from_uniform_data(data: GuiUniformData) -> Self {
        Self {
            fractal_color: <Vector4Packed<f32> as IntoUnpacked<Rgba>>::into_unpacked(
                data.fractal_color,
            )
            .into(),
            background_color: <Vector4Packed<f32> as IntoUnpacked<Rgba>>::into_unpacked(
                data.background_color,
            )
            .into(),
            primitive_shape: PrimitiveShape::from_id(data.primitive_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{PI, Vector3};

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
