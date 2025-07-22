use egui::{Color32, Rgba};
use egui_wgpu::wgpu::{self, util::DeviceExt as _};
use winit::dpi::PhysicalSize;

use crate::math::{Matrix3x3, Radians, Vector2};
use crate::packed::{
    IntoPacked as _, IntoUnpacked, Matrix3x3F32Packed, RadiansPacked, Vector2Packed, Vector4Packed,
};
use crate::scene::PrimitiveShape;

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

pub trait UniformDataDescriptor<Data: UniformData> {
    fn into_uniform_data(self) -> Data;
    fn from_uniform_data(data: Data) -> Self;
}

#[derive(Copy, Clone, Debug, Default)]
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

#[derive(Copy, Clone, Debug, Default)]
pub struct CameraUniformDataDescriptor {
    pub origin_distance: f32,
    pub min_distance: f32,
    pub angles: Vector2<Radians>,
}

impl CameraUniformDataDescriptor {
    fn camera_matrix(&self) -> Matrix3x3<f32> {
        let Vector2(phi, theta) = self.angles;

        let phi_rotation = Matrix3x3::rotation_matrix_z(phi);
        let theta_rotation = Matrix3x3::rotation_matrix_y(-theta);

        phi_rotation * theta_rotation
    }
}

impl UniformDataDescriptor<CameraUniformData> for CameraUniformDataDescriptor {
    fn into_uniform_data(self) -> CameraUniformData {
        CameraUniformData {
            origin_distance: self.origin_distance,
            min_distance: self.min_distance,
            angles: Vector2(self.angles.0, self.angles.1).into_packed(),
            matrix: self.camera_matrix().into_packed(),
        }
    }

    fn from_uniform_data(data: CameraUniformData) -> Self {
        Self {
            origin_distance: data.origin_distance,
            min_distance: data.min_distance,
            angles: Vector2(data.angles.0.into_unpacked(), data.angles.1.into_unpacked()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GuiUniformDataDescriptor {
    pub fractal_color: Color32,
    pub background_color: Color32,
    pub primitive_shape: PrimitiveShape,
}

impl Default for GuiUniformDataDescriptor {
    fn default() -> Self {
        Self {
            fractal_color: Color32::from_rgb(200, 200, 200),
            background_color: Color32::from_rgb(0, 0, 0),
            primitive_shape: PrimitiveShape::Sphere,
        }
    }
}

impl UniformDataDescriptor<GuiUniformData> for GuiUniformDataDescriptor {
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
            .camera_matrix()
    }
}

impl Uniform<GuiUniformData> {
    pub fn background_color(&self) -> Color32 {
        self.data_descriptor::<GuiUniformDataDescriptor>()
            .background_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::{PI, Vector3, Vector4},
        packed::Vector3Packed,
    };

    #[test]
    fn test_packing_and_depacking() {
        assert_eq!(Vector2(1., 2.).into_packed(), Vector2Packed(1., 2.));
        assert_eq!(Vector2Packed(1., 2.).into_unpacked(), Vector2(1., 2.));
        assert_eq!(Vector3(1., 2., 3.).into_packed(), Vector3Packed(1., 2., 3.));
        assert_eq!(
            Vector3Packed(1., 2., 3.).into_unpacked(),
            Vector3(1., 2., 3.)
        );
        assert_eq!(
            Vector4(1., 2., 3., 4.).into_packed(),
            Vector4Packed(1., 2., 3., 4.)
        );
        assert_eq!(
            Vector4Packed(1., 2., 3., 4.).into_unpacked(),
            Vector4(1., 2., 3., 4.)
        );
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.),
            )
            .into_packed(),
            Matrix3x3F32Packed(
                Vector4Packed(1., 2., 3., 0.),
                Vector4Packed(4., 5., 6., 0.),
                Vector4Packed(7., 8., 9., 0.),
            )
        );
        assert_eq!(
            Matrix3x3F32Packed(
                Vector4Packed(1., 2., 3., 0.),
                Vector4Packed(4., 5., 6., 0.),
                Vector4Packed(7., 8., 9., 0.),
            )
            .into_unpacked(),
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.),
            )
        );
        assert_eq!(
            Vector2(Vector2(1., 2.), Vector2(3., 4.)).into_packed(),
            Vector2Packed(Vector2Packed(1., 2.), Vector2Packed(3., 4.))
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
            descriptor.camera_matrix(),
            Matrix3x3::from_columns(
                Vector3(1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., -1.)
            )
        );
    }
}
