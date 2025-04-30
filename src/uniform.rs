use egui_wgpu::wgpu::{self, util::DeviceExt as _};

use winit::dpi::PhysicalSize;

use crate::math::{Matrix3x3, Radians, Vector2, Vector3, Vector4};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Vector2F32Packed(f32, f32);

impl From<Vector2<f32>> for Vector2F32Packed {
    fn from(value: Vector2<f32>) -> Self {
        Self(value.0, value.1)
    }
}

impl From<Vector2F32Packed> for Vector2<f32> {
    fn from(value: Vector2F32Packed) -> Self {
        Self(value.0, value.1)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
// Last field is used for padding when sending data to the GPU
struct Vector3F32Packed(f32, f32, f32, u32);

impl From<Vector3<f32>> for Vector3F32Packed {
    fn from(value: Vector3<f32>) -> Self {
        Self(value.0, value.1, value.2, 0)
    }
}

impl From<Vector3F32Packed> for Vector3<f32> {
    fn from(value: Vector3F32Packed) -> Self {
        Self(value.0, value.1, value.2)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Vector4F32Packed(f32, f32, f32, f32);

impl From<Vector4<f32>> for Vector4F32Packed {
    fn from(value: Vector4<f32>) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

impl From<Vector4F32Packed> for Vector4<f32> {
    fn from(value: Vector4F32Packed) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Matrix3x3F32Packed(Vector3F32Packed, Vector3F32Packed, Vector3F32Packed);

impl From<Matrix3x3<f32>> for Matrix3x3F32Packed {
    fn from(value: Matrix3x3<f32>) -> Self {
        Self(value.0.into(), value.1.into(), value.2.into())
    }
}

impl From<Matrix3x3F32Packed> for Matrix3x3<f32> {
    fn from(value: Matrix3x3F32Packed) -> Self {
        Self(value.0.into(), value.1.into(), value.2.into())
    }
}

pub trait UniformData: bytemuck::Pod {}

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
    angles: Vector2F32Packed,
    matrix: Matrix3x3F32Packed,
}

impl UniformData for CameraUniformData {}

// #[repr(C)]
// #[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
// struct FractalUniformData {
//     // TODO
// }

pub trait UniformDataDescriptor<Data: UniformData> {
    fn into_uniform_data(self) -> Data;
    fn from_uniform_data(data: Data) -> Self;
}

#[derive(Debug)]
pub struct SizeUniformDataDescriptor {
    pub width: u32,
    pub height: u32,
}

impl UniformDataDescriptor<SizeUniformData> for SizeUniformDataDescriptor {
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

impl From<PhysicalSize<u32>> for SizeUniformDataDescriptor {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}

#[derive(Debug)]
pub struct CameraUniformDataDescriptor {
    pub origin_distance: f32,
    pub min_distance: f32,
    pub angles: Vector2<Radians>,
}

impl CameraUniformDataDescriptor {
    fn get_camera_matrix(&self) -> Matrix3x3<f32> {
        let Vector2(phi, theta) = self.angles;

        let phi_rotation = Matrix3x3::get_rotation_matrix_z(phi);
        let theta_rotation = Matrix3x3::get_rotation_matrix_y(-theta);

        phi_rotation * theta_rotation
    }
}

impl UniformDataDescriptor<CameraUniformData> for CameraUniformDataDescriptor {
    fn into_uniform_data(self) -> CameraUniformData {
        CameraUniformData {
            origin_distance: self.origin_distance,
            min_distance: self.min_distance,
            angles: Vector2(self.angles.0.get_radians(), self.angles.1.get_radians()).into(),
            matrix: self.get_camera_matrix().into(),
        }
    }

    fn from_uniform_data(data: CameraUniformData) -> Self {
        Self {
            origin_distance: data.origin_distance,
            min_distance: data.min_distance,
            angles: Vector2(
                Radians::from_radians(data.angles.0),
                Radians::from_radians(data.angles.1),
            ),
        }
    }
}

#[derive(Debug)]
pub struct Uniform<Data: UniformData> {
    data: Data,
    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl<Data: UniformData> Uniform<Data> {
    pub fn data_descriptor<D>(&self) -> D
    where
        D: UniformDataDescriptor<Data>,
    {
        UniformDataDescriptor::<Data>::from_uniform_data(self.data)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

impl<Data: UniformData> Uniform<Data> {
    pub fn create_uniform_buffer(
        device: &wgpu::Device,
        data: &Data,
        label: Option<&str>,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&[*data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn create_uniform_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> wgpu::BindGroupLayout {
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
            label: label
                .map(|label| format!("{label}_bind_group_layout"))
                .as_deref(),
        })
    }

    pub fn create_uniform_bind_group(
        device: &wgpu::Device,
        buffer: &wgpu::Buffer,
        bind_group_layout: &wgpu::BindGroupLayout,
        label: Option<&str>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: label.map(|label| format!("{label}_bind_group")).as_deref(),
        })
    }

    pub fn create_uniform<D>(device: &wgpu::Device, data_descriptor: D, label: Option<&str>) -> Self
    where
        D: UniformDataDescriptor<Data>,
    {
        let data = data_descriptor.into_uniform_data();

        let buffer = Self::create_uniform_buffer(device, &data, label);
        let bind_group_layout = Self::create_uniform_bind_group_layout(device, label);
        let bind_group =
            Self::create_uniform_bind_group(device, &buffer, &bind_group_layout, label);

        Self {
            data,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_uniform<D>(&mut self, data_descriptor: D, queue: &wgpu::Queue)
    where
        D: UniformDataDescriptor<Data>,
    {
        self.data = data_descriptor.into_uniform_data();

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }
}

impl Uniform<SizeUniformData> {
    pub fn width(&self) -> u32 {
        self.data_descriptor::<SizeUniformDataDescriptor>().width
    }

    pub fn height(&self) -> u32 {
        self.data_descriptor::<SizeUniformDataDescriptor>().height
    }
}

impl Uniform<CameraUniformData> {
    pub fn origin_distance(&self) -> f32 {
        self.data_descriptor::<CameraUniformDataDescriptor>()
            .origin_distance
    }

    pub fn min_distance(&self) -> f32 {
        self.data_descriptor::<CameraUniformDataDescriptor>()
            .min_distance
    }

    pub fn angles(&self) -> Vector2<Radians> {
        self.data_descriptor::<CameraUniformDataDescriptor>().angles
    }

    pub fn matrix(&self) -> Matrix3x3<f32> {
        self.data_descriptor::<CameraUniformDataDescriptor>()
            .get_camera_matrix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::PI;

    #[test]
    fn test_podding_and_depodding() {
        assert_eq!(
            Vector2F32Packed::from(Vector2(1., 2.)),
            Vector2F32Packed(1., 2.)
        );
        assert_eq!(
            Into::<Vector2<f32>>::into(Vector2F32Packed(1., 2.)),
            Vector2(1., 2.)
        );
        assert_eq!(
            Vector3F32Packed::from(Vector3(1., 2., 3.)),
            Vector3F32Packed(1., 2., 3., 0)
        );
        assert_eq!(
            Into::<Vector3<f32>>::into(Vector3F32Packed(1., 2., 3., 0)),
            Vector3(1., 2., 3.)
        );
        assert_eq!(
            Vector4F32Packed::from(Vector4(1., 2., 3., 4.)),
            Vector4F32Packed(1., 2., 3., 4.)
        );
        assert_eq!(
            Into::<Vector4<f32>>::into(Vector4F32Packed(1., 2., 3., 4.)),
            Vector4(1., 2., 3., 4.)
        );
        assert_eq!(
            Matrix3x3F32Packed::from(Matrix3x3(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            )),
            Matrix3x3F32Packed(
                Vector3F32Packed(1., 2., 3., 0),
                Vector3F32Packed(4., 5., 6., 0),
                Vector3F32Packed(7., 8., 9., 0)
            )
        );
        assert_eq!(
            Into::<Matrix3x3<f32>>::into(Matrix3x3F32Packed(
                Vector3F32Packed(1., 2., 3., 0),
                Vector3F32Packed(4., 5., 6., 0),
                Vector3F32Packed(7., 8., 9., 0)
            )),
            Matrix3x3(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            )
        );
    }

    #[test]
    fn test_camera_matrix() {
        let descriptor = CameraUniformDataDescriptor {
            origin_distance: 0.,
            min_distance: 0.,
            angles: Vector2(Radians::from_radians(PI), Radians::from_radians(PI)),
        };

        assert_eq!(
            descriptor.get_camera_matrix(),
            Matrix3x3(
                Vector3(1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., -1.)
            )
        );
    }
}
