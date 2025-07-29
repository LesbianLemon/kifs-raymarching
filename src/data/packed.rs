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

macro_rules! impl_packing_unpacking {
    (
        $Type:ty,
        impl$(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>)? $PackUnpackTrait:ident::$op_method:ident($self:ident) -> $OutputType:ty $return:block
    ) => {
        impl$(<$($Generic $(: $HeadTrait $(+ $TailTrait)*)?),+>)? $PackUnpackTrait<$OutputType> for $Type
        {
            fn $op_method($self) -> $OutputType $return
        }
    };
}

macro_rules! impl_vector_packing_unpacking {
    ($Vector:ident{$(.$field:tt)+} <-> $VectorPacked:ident{$(.$field_packed:tt)+}) => {
        impl_packing_unpacking!(
            $Vector<T>,
            impl<T: IntoPacked<U>, U> IntoPacked::into_packed(self) -> $VectorPacked<U> {
                $VectorPacked($(self.$field.into_packed()),+)
            }
        );
        impl_packing_unpacking!(
            $VectorPacked<T>,
            impl<T: IntoUnpacked<U>, U> IntoUnpacked::into_unpacked(self) -> $Vector<U> {
                $Vector($(self.$field_packed.into_unpacked()),+)
            }
        );
    };
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vector2Packed<T>(T, T);

impl_vector_packing_unpacking!(Vector2{ .0 .1 } <-> Vector2Packed{ .0 .1 });

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vector3Packed<T>(T, T, T);

impl_vector_packing_unpacking!(Vector3{ .0 .1 .2 } <-> Vector3Packed{ .0 .1 .2 });

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vector4Packed<T>(T, T, T, T);

impl_vector_packing_unpacking!(Vector4{ .0 .1 .2 .3 } <-> Vector4Packed{ .0 .1 .2 .3 });

pub(super) type Matrix3x3F32Packed = Vector3Packed<Vector4Packed<f32>>;

// Implemented only for f32 due to not being able to generalize alignments
// Using Vector4 due to simply converting to Packed variant not creating the correct alignment for f32
impl IntoPacked<Matrix3x3F32Packed> for Matrix3x3<f32> {
    fn into_packed(self) -> Matrix3x3F32Packed {
        let columns = self.columns();

        Vector3Packed(
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

impl IntoPacked<f32> for Radians {
    fn into_packed(self) -> f32 {
        self.radians()
    }
}

impl IntoUnpacked<Radians> for f32 {
    fn into_unpacked(self) -> Radians {
        Radians::from_radians(self)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packing_and_unpacking() {
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
            Vector2(Vector2(1., 2.), Vector2(3., 4.)).into_packed(),
            Vector2Packed(Vector2Packed(1., 2.), Vector2Packed(3., 4.))
        );
    }
}
