use egui::Rgba;

use crate::math::{Matrix3x3, Num, Radians, Vector2, Vector3, Vector4};

pub trait IntoPacked<Packed> {
    fn into_packed(self) -> Packed;
}

pub trait IntoUnpacked<Unpacked> {
    fn into_unpacked(self) -> Unpacked;
}

impl<T> IntoPacked<T> for T
where
    T: Num,
{
    fn into_packed(self) -> T {
        self
    }
}

impl<T> IntoUnpacked<T> for T
where
    T: Num,
{
    fn into_unpacked(self) -> T {
        self
    }
}

impl IntoPacked<Vector4Packed<f32>> for Rgba {
    fn into_packed(self) -> Vector4Packed<f32> {
        Vector4Packed(self[0], self[1], self[2], self[3])
    }
}

impl IntoUnpacked<Rgba> for Vector4Packed<f32> {
    fn into_unpacked(self) -> Rgba {
        Rgba::from_rgba_premultiplied(
            self.0.clamp(0., 1.),
            self.1.clamp(0., 1.),
            self.2.clamp(0., 1.),
            self.3.clamp(0., 1.),
        )
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector2Packed<T>(pub T, pub T);

impl<T, U> IntoPacked<Vector2Packed<U>> for Vector2<T>
where
    T: IntoPacked<U>,
{
    fn into_packed(self) -> Vector2Packed<U> {
        Vector2Packed(self.0.into_packed(), self.1.into_packed())
    }
}

impl<T, U> IntoUnpacked<Vector2<U>> for Vector2Packed<T>
where
    T: IntoUnpacked<U>,
{
    fn into_unpacked(self) -> Vector2<U> {
        Vector2(self.0.into_unpacked(), self.1.into_unpacked())
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector3Packed<T>(pub T, pub T, pub T);

impl<T, U> IntoPacked<Vector3Packed<U>> for Vector3<T>
where
    T: IntoPacked<U>,
{
    fn into_packed(self) -> Vector3Packed<U> {
        Vector3Packed(
            self.0.into_packed(),
            self.1.into_packed(),
            self.2.into_packed(),
        )
    }
}

impl<T, U> IntoUnpacked<Vector3<U>> for Vector3Packed<T>
where
    T: IntoUnpacked<U>,
{
    fn into_unpacked(self) -> Vector3<U> {
        Vector3(
            self.0.into_unpacked(),
            self.1.into_unpacked(),
            self.2.into_unpacked(),
        )
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector4Packed<T>(pub T, pub T, pub T, pub T);

impl<T, U> IntoPacked<Vector4Packed<U>> for Vector4<T>
where
    T: IntoPacked<U>,
{
    fn into_packed(self) -> Vector4Packed<U> {
        Vector4Packed(
            self.0.into_packed(),
            self.1.into_packed(),
            self.2.into_packed(),
            self.3.into_packed(),
        )
    }
}

impl<T, U> IntoUnpacked<Vector4<U>> for Vector4Packed<T>
where
    T: IntoUnpacked<U>,
{
    fn into_unpacked(self) -> Vector4<U> {
        Vector4(
            self.0.into_unpacked(),
            self.1.into_unpacked(),
            self.2.into_unpacked(),
            self.3.into_unpacked(),
        )
    }
}

// Implementation for f32 matrix specifically
// Due to stupid alignment bullshit, making matrix packed more general is impossible (i swear i tried)
// Need to use Vector4Packed due to Vector3Packed<f32> having size 12 and alignment 16
#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix3x3F32Packed(
    pub Vector4Packed<f32>,
    pub Vector4Packed<f32>,
    pub Vector4Packed<f32>,
);

impl IntoPacked<Matrix3x3F32Packed> for Matrix3x3<f32> {
    fn into_packed(self) -> Matrix3x3F32Packed {
        let columns = self.columns();

        Matrix3x3F32Packed(
            columns.0.extend(0.).into_packed(),
            columns.1.extend(0.).into_packed(),
            columns.2.extend(0.).into_packed(),
        )
    }
}

impl IntoUnpacked<Matrix3x3<f32>> for Matrix3x3F32Packed {
    fn into_unpacked(self) -> Matrix3x3<f32> {
        Matrix3x3::from_columns(
            Vector4::shrink(self.0.into_unpacked()),
            Vector4::shrink(self.1.into_unpacked()),
            Vector4::shrink(self.2.into_unpacked()),
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RadiansPacked(pub f32);

impl IntoPacked<RadiansPacked> for Radians {
    fn into_packed(self) -> RadiansPacked {
        RadiansPacked(self.radians())
    }
}

impl IntoUnpacked<Radians> for RadiansPacked {
    fn into_unpacked(self) -> Radians {
        Radians::from_radians(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::packed::Vector3Packed,
        math::{Vector3, Vector4},
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
}
