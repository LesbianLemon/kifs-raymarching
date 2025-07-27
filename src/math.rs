use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait Num: num_traits::Num {}

macro_rules! impl_num {
    ($Type:ty) => {
        impl Num for $Type {}
    };
}

impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(u128);
impl_num!(usize);
impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);
impl_num!(i128);
impl_num!(isize);
impl_num!(f32);
impl_num!(f64);

pub trait Float: num_traits::Float {}

macro_rules! impl_float {
    ($type:ty) => {
        impl Float for $type {}
    };
}

impl_float!(f32);
impl_float!(f64);

pub use std::f32::consts::PI;
pub const TWO_PI: f32 = 2. * PI;
// Accuracy of 0.0001 is good enough for our graphics
pub const EPSILON: f32 = 1.0e-4;

macro_rules! impl_vector_extend {
    ($Vector:ident{$(.$field:tt)+} -> $VectorNext:ident{$(.$field_next:tt)+}) => {
        impl<T> $Vector<T> {
            pub fn extend(self, x: T) -> $VectorNext<T> {
                $VectorNext($(self.$field),+, x)
            }
        }
    };
}

macro_rules! impl_vector_shrink {
    ($Vector:ident{$(.$field:tt)+} -> $VectorPrev:ident{$(.$field_prev:tt)+}) => {
        impl<T> $Vector<T> {
            pub fn shrink(self) -> $VectorPrev<T> {
                $VectorPrev($(self.$field_prev),+)
            }
        }
    };
}

macro_rules! impl_vector_partial_eq {
    ($Vector:ident{.$field_head:tt $(.$field_tail:tt)*}) => {
        impl<T> PartialEq for $Vector<T>
        where
            T: Float,
        {
            fn eq(&self, other: &Self) -> bool {
                let epsilon_t =
                    T::from(EPSILON).expect("Can only compare values that can be cast to from f32");

                (self.$field_head - other.$field_head).abs() < epsilon_t $(&& (self.$field_tail - other.$field_tail).abs() < epsilon_t)*
            }
        }
    };
}

// Replace + with {plus} when stating traits passed to macro due to some weird Rust macro behaviours, which is impossible to get around
macro_rules! impl_operation {
    (
        $Type:ident$(<$($TypeGeneric:ident),+>)?,
        impl$(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>)? $OpTrait:ident$(<$($TraitGeneric:ty),+>)?::$op_method:ident($($argument:ident $(: $ArgumentType:ty)?),*) -> $OutputType:ty {
            $return:expr
        }
    ) => {
        impl$(<$($Generic $(: $HeadTrait $(+ $TailTrait)*)?),+>)? $OpTrait$(<$($TraitGeneric),*>)? for $Type$(<$($TypeGeneric),+>)?
        {
            type Output = $OutputType;

            fn $op_method($($argument $(: $ArgumentType)?),*) -> Self::Output {
                $return
            }
        }
    };
}

#[allow(dead_code)]
enum ComponentOperationType {
    // Operations like Neg which act like (x, y, ...) |-> (op(x), op(y), ...)
    Unary,
    // Operations like Add which act like ((x1, y1, ...), (x2, y2, ...)) |-> (op(x1, x2), op(y1, y2), ...)
    InternalBinary,
    // Operations like left multiplication with scalar which act like (c, (x2, y2, ...)) |-> (op(c, x2), op(c, y2), ...)
    ExternalBinaryLeft,
    // Operations like right multiplication with scalar which act like ((x1, y1, ...), c) |-> (op(x1, c), op(y1, c), ...)
    ExternalBinaryRight,
}

macro_rules! impl_component_operation {
    (
        $(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>,)?
        $Type:ident$(<$($ComponentGeneric:ident),+>)?$({$(.$field:tt)+})?,
        $OpTrait:ident::$op_method:ident,
        ComponentOperationType::Unary
    ) => {
        impl_operation!(
            $Type$(<$($ComponentGeneric),+>)?,
            impl$(<$($Generic $(: $HeadTrait $({plus} $TailTrait)*)?),+>)? $OpTrait::$op_method(self) -> Self {
                Self{
                    $($($field: $OpTrait::$op_method(self.$field)),+)?
                }
            }
        );
    };
    (
        $(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>,)?
        $Type:ident$(<$($ComponentGeneric:ident),+>)?$({$(.$field:tt)+})?,
        $OpTrait:ident::$op_method:ident,
        ComponentOperationType::InternalBinary
    ) => {
        impl_operation!(
            $Type$(<$($ComponentGeneric),+>)?,
            impl$(<$($Generic $(: $HeadTrait $({plus} $TailTrait)*)?),+>)? $OpTrait<$Type$(<$($ComponentGeneric),+>)?>::$op_method(self, rhs: Self) -> Self {
                Self{
                    $($($field: $OpTrait::$op_method(self.$field, rhs.$field)),+)?
                }
            }
        );
    };
    (
        $(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>,)?
        $TypeLeft:ident$(<$($ComponentGenericLeft:ident),+>)?,
        $TypeRight:ident$(<$($ComponentGenericRight:ident),+>)?$({$(.$field:tt)+})?,
        $OpTrait:ident::$op_method:ident,
        ComponentOperationType::ExternalBinaryLeft
    ) => {
        impl_operation!(
            $TypeLeft$(<$($ComponentGenericLeft),+>)?,
            impl$(<$($Generic $(: $HeadTrait $({plus} $TailTrait)*)?),+>)? $OpTrait<$TypeRight$(<$($ComponentGenericRight),+>)?>::$op_method(self, rhs: $TypeRight$(<$($ComponentGenericRight),+>)?) -> $TypeRight$(<$($ComponentGenericRight),+>)? {
                $TypeRight{
                    $($($field: $OpTrait::$op_method(self, rhs.$field)),+)?
                }
            }
        );
    };
    (
        $(<$($Generic:ident $(: $HeadTrait:path $({plus} $TailTrait:path)*)?),*>,)?
        $TypeLeft:ident$(<$($ComponentGenericLeft:ident),+>)?$({$(.$field:tt)+})?,
        $TypeRight:ident$(<$($ComponentGenericRight:ident),+>)?,
        $OpTrait:ident::$op_method:ident,
        ComponentOperationType::ExternalBinaryRight
    ) => {
        impl_operation!(
            $TypeLeft$(<$($ComponentGenericLeft),+>)?,
            impl$(<$($Generic $(: $HeadTrait $({plus} $TailTrait)*)?),+>)? $OpTrait<$TypeRight$(<$($ComponentGenericRight),+>)?>::$op_method(self, rhs: $TypeRight$(<$($ComponentGenericRight),+>)?) -> Self {
                Self{
                    $($($field: $OpTrait::$op_method(self.$field, rhs)),+)?
                }
            }
        );
    };
}

macro_rules! impl_vector_scalar_product {
    ($Vector:ident{.$field_head:tt $(.$field_tail:tt)*}) => {
        impl_operation!(
            $Vector<T>,
            impl<T: Add<T, Output = T> {plus} Mul<T, Output = T>> Mul<Self>::mul(self, rhs: Self) -> T {
                self.$field_head * rhs.$field_head $(+ self.$field_tail * rhs.$field_tail)*
            }
        );
    };
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector2<T>(pub T, pub T);

impl_vector_extend!(Vector2{ .0 .1 } -> Vector3{ .0 .1 .2 });

impl_vector_partial_eq!(Vector2{ .0 .1 });

impl_component_operation!(<T: Neg<Output = T>>, Vector2<T>{ .0 .1 }, Neg::neg, ComponentOperationType::Unary);
impl_component_operation!(<T: Add<T, Output = T>>, Vector2<T>{ .0 .1 }, Add::add, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Sub<T, Output = T>>, Vector2<T>{ .0 .1 }, Sub::sub, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Mul<T, Output = T> {plus} Copy>, Vector2<T>{ .0 .1 }, T, Mul::mul, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(<T: Div<T, Output = T> {plus} Copy>, Vector2<T>{ .0 .1 }, T, Div::div, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(f32, Vector2<f32>{ .0 .1 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);
impl_component_operation!(f64, Vector2<f64>{ .0 .1 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);

impl_vector_scalar_product!(Vector2{ .0 .1 });

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector3<T>(pub T, pub T, pub T);

impl_vector_extend!(Vector3{ .0 .1 .2 } -> Vector4{ .0 .1 .2 .3 });
impl_vector_shrink!(Vector3{ .0 .1 .2 } -> Vector2{ .0 .1 });

impl_vector_partial_eq!(Vector3{ .0 .1 .2 });

impl_component_operation!(<T: Neg<Output = T>>, Vector3<T>{ .0 .1 .2 }, Neg::neg, ComponentOperationType::Unary);
impl_component_operation!(<T: Add<T, Output = T>>, Vector3<T>{ .0 .1 .2 }, Add::add, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Sub<T, Output = T>>, Vector3<T>{ .0 .1 .2 }, Sub::sub, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Mul<T, Output = T> {plus} Copy>, Vector3<T>{ .0 .1 .2 }, T, Mul::mul, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(<T: Div<T, Output = T> {plus} Copy>, Vector3<T>{ .0 .1 .2 }, T, Div::div, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(f32, Vector3<f32>{ .0 .1 .2 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);
impl_component_operation!(f64, Vector3<f64>{ .0 .1 .2 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);

impl_vector_scalar_product!(Vector3{ .0 .1 .2 });

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector4<T>(pub T, pub T, pub T, pub T);

impl_vector_shrink!(Vector4{ .0 .1 .2 .3 } -> Vector3{ .0 .1 .2 });

impl_vector_partial_eq!(Vector4{ .0 .1 .2 .3 });

impl_component_operation!(<T: Neg<Output = T>>, Vector4<T>{ .0 .1 .2 .3 }, Neg::neg, ComponentOperationType::Unary);
impl_component_operation!(<T: Add<T, Output = T>>, Vector4<T>{ .0 .1 .2 .3 }, Add::add, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Sub<T, Output = T>>, Vector4<T>{ .0 .1 .2 .3 }, Sub::sub, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Mul<T, Output = T> {plus} Copy>, Vector4<T>{ .0 .1 .2 .3 }, T, Mul::mul, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(<T: Div<T, Output = T> {plus} Copy>, Vector4<T>{ .0 .1 .2 .3 }, T, Div::div, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(f32, Vector4<f32>{ .0 .1 .2 .3 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);
impl_component_operation!(f64, Vector4<f64>{ .0 .1 .2 .3 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);

impl_vector_scalar_product!(Vector4{ .0 .1 .2 .3 });

#[derive(Copy, Clone, Debug, Default)]
pub struct Matrix3x3<T>(Vector3<T>, Vector3<T>, Vector3<T>);

impl<T> Matrix3x3<T> {
    pub fn from_columns(col1: Vector3<T>, col2: Vector3<T>, col3: Vector3<T>) -> Self {
        Self(col1, col2, col3)
    }

    pub fn columns(&self) -> (Vector3<T>, Vector3<T>, Vector3<T>)
    where
        T: Copy,
    {
        (self.0, self.1, self.2)
    }

    pub fn from_rows(row1: Vector3<T>, row2: Vector3<T>, row3: Vector3<T>) -> Self {
        Self(
            Vector3(row1.0, row2.0, row3.0),
            Vector3(row1.1, row2.1, row3.1),
            Vector3(row1.2, row2.2, row3.2),
        )
    }

    pub fn rows(&self) -> (Vector3<T>, Vector3<T>, Vector3<T>)
    where
        T: Copy,
    {
        (
            Vector3(self.0.0, self.1.0, self.2.0),
            Vector3(self.0.1, self.1.1, self.2.1),
            Vector3(self.0.2, self.1.2, self.2.2),
        )
    }
}

impl<T> PartialEq for Matrix3x3<T>
where
    T: Float,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl_component_operation!(<T: Neg<Output = T>>, Matrix3x3<T>{ .0 .1 .2 }, Neg::neg, ComponentOperationType::Unary);
impl_component_operation!(<T: Add<T, Output = T>>, Matrix3x3<T>{ .0 .1 .2 }, Add::add, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Sub<T, Output = T>>, Matrix3x3<T>{ .0 .1 .2 }, Sub::sub, ComponentOperationType::InternalBinary);
impl_component_operation!(<T: Mul<T, Output = T> {plus} Copy>, Matrix3x3<T>{ .0 .1 .2 }, T, Mul::mul, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(<T: Div<T, Output = T> {plus} Copy>, Matrix3x3<T>{ .0 .1 .2 }, T, Div::div, ComponentOperationType::ExternalBinaryRight);
impl_component_operation!(f32, Matrix3x3<f32>{ .0 .1 .2 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);
impl_component_operation!(f64, Matrix3x3<f64>{ .0 .1 .2 }, Mul::mul, ComponentOperationType::ExternalBinaryLeft);

impl<T> Mul<Self> for Matrix3x3<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let rows = self.rows();

        Self(
            Vector3(rows.0 * rhs.0, rows.1 * rhs.0, rows.2 * rhs.0),
            Vector3(rows.0 * rhs.1, rows.1 * rhs.1, rows.2 * rhs.1),
            Vector3(rows.0 * rhs.2, rows.1 * rhs.2, rows.2 * rhs.2),
        )
    }
}

impl<T> Mul<Vector3<T>> for Matrix3x3<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn mul(self, rhs: Vector3<T>) -> Self::Output {
        let rows = self.rows();

        Vector3(rows.0 * rhs, rows.1 * rhs, rows.2 * rhs)
    }
}

impl Matrix3x3<f32> {
    pub const IDENTITY: Self = Self(
        Vector3(1., 0., 0.),
        Vector3(0., 1., 0.),
        Vector3(0., 0., 1.),
    );

    pub fn zero() -> Self {
        Self(
            Vector3(0., 0., 0.),
            Vector3(0., 0., 0.),
            Vector3(0., 0., 0.),
        )
    }

    pub fn rotation_matrix_x(angle: Radians) -> Self {
        let Vector2(cos, sin) = angle.cos_sin();

        Self(
            Vector3(1., 0., 0.),
            Vector3(0., cos, sin),
            Vector3(0., -sin, cos),
        )
    }

    pub fn rotation_matrix_y(angle: Radians) -> Self {
        let Vector2(cos, sin) = angle.cos_sin();

        Self(
            Vector3(cos, 0., -sin),
            Vector3(0., 1., 0.),
            Vector3(sin, 0., cos),
        )
    }

    pub fn rotation_matrix_z(angle: Radians) -> Self {
        let Vector2(cos, sin) = angle.cos_sin();

        Self(
            Vector3(cos, sin, 0.),
            Vector3(-sin, cos, 0.),
            Vector3(0., 0., 1.),
        )
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Radians(f32);

impl Radians {
    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Self::from_radians((degrees / 180.0) * PI)
    }

    pub fn radians(&self) -> f32 {
        self.0
    }

    pub fn degrees(&self) -> f32 {
        (self.radians() / PI) * 180.0
    }

    // Clamps its value to [min, max]
    pub fn clamp(self, min: f32, max: f32) -> Self {
        Radians::from_radians(self.radians().clamp(min, max))
    }

    // Returns the same angle, but on [0, 2PI]
    pub fn standardize(self) -> Self {
        Self(((self.radians() % TWO_PI) + TWO_PI) % TWO_PI)
    }

    pub fn cos(&self) -> f32 {
        self.radians().cos()
    }

    pub fn sin(&self) -> f32 {
        self.radians().sin()
    }

    pub fn cos_sin(&self) -> Vector2<f32> {
        Vector2(self.radians().cos(), self.radians().sin())
    }
}

impl PartialEq for Radians {
    fn eq(&self, other: &Self) -> bool {
        let diff = (self.radians() - other.radians()).abs() % TWO_PI;

        !(EPSILON..=TWO_PI - EPSILON).contains(&diff)
    }
}

impl_component_operation!(Radians{ .0 }, Neg::neg, ComponentOperationType::Unary);
impl_component_operation!(Radians{ .0 }, Add::add, ComponentOperationType::InternalBinary);
impl_component_operation!(Radians{ .0 }, Sub::sub, ComponentOperationType::InternalBinary);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_extension_and_shrinking() {
        assert_eq!(Vector2(1., 2.).extend(3.), Vector3(1., 2., 3.));
        assert_eq!(Vector3(1., 2., 3.).extend(4.), Vector4(1., 2., 3., 4.));
    }

    #[test]
    fn test_vector_addition() {
        assert_eq!(Vector2(1., 2.) + Vector2(-1., -2.), Vector2(0., 0.));
        assert_eq!(
            Vector3(1., 2., 3.) + Vector3(-1., -2., -3.),
            Vector3(0., 0., 0.)
        );
        assert_eq!(
            Vector4(1., 2., 3., 4.) + Vector4(-1., -2., -3., -4.),
            Vector4(0., 0., 0., 0.)
        );
    }

    #[test]
    fn test_vector_subtraction() {
        assert_eq!(Vector2(1., 2.) - Vector2(1., 2.), Vector2(0., 0.));
        assert_eq!(
            Vector3(1., 2., 3.) - Vector3(1., 2., 3.),
            Vector3(0., 0., 0.)
        );
        assert_eq!(
            Vector4(1., 2., 3., 4.) - Vector4(1., 2., 3., 4.),
            Vector4(0., 0., 0., 0.)
        );
    }

    #[test]
    fn test_vector_scalar_product() {
        assert_eq!(Vector2(1, 2) * Vector2(2, -1), 0);
        assert_eq!(Vector3(1, 2, 3) * Vector3(1, 1, -1), 0);
        assert_eq!(Vector4(1, 2, 3, 4) * Vector4(-1, 1, 1, -1), 0);
    }

    #[test]
    fn test_vector_multiplication_with_scalar() {
        assert_eq!(Vector2(1.0f32, 2.0f32) * 2.0f32, Vector2(2.0f32, 4.0f32));
        assert_eq!(Vector2(1.0f64, 2.0f64) * 2.0f64, Vector2(2.0f64, 4.0f64));
        assert_eq!(2.0f32 * Vector2(1.0f32, 2.0f32), Vector2(2.0f32, 4.0f32));
        assert_eq!(2.0f64 * Vector2(1.0f64, 2.0f64), Vector2(2.0f64, 4.0f64));
        assert_eq!(
            Vector3(1.0f32, 2.0f32, 3.0f32) * 2.0f32,
            Vector3(2.0f32, 4.0f32, 6.0f32)
        );
        assert_eq!(
            Vector3(1.0f64, 2.0f64, 3.0f64) * 2.0f64,
            Vector3(2.0f64, 4.0f64, 6.0f64)
        );
        assert_eq!(
            2.0f32 * Vector3(1.0f32, 2.0f32, 3.0f32),
            Vector3(2.0f32, 4.0f32, 6.0f32)
        );
        assert_eq!(
            2.0f64 * Vector3(1.0f64, 2.0f64, 3.0f64),
            Vector3(2.0f64, 4.0f64, 6.0f64)
        );
        assert_eq!(
            Vector4(1.0f32, 2.0f32, 3.0f32, 4.0f32) * 2.0f32,
            Vector4(2.0f32, 4.0f32, 6.0f32, 8.0f32)
        );
        assert_eq!(
            Vector4(1.0f64, 2.0f64, 3.0f64, 4.0f64) * 2.0f64,
            Vector4(2.0f64, 4.0f64, 6.0f64, 8.0f64)
        );
        assert_eq!(
            2.0f32 * Vector4(1.0f32, 2.0f32, 3.0f32, 4.0f32),
            Vector4(2.0f32, 4.0f32, 6.0f32, 8.0f32)
        );
        assert_eq!(
            2.0f64 * Vector4(1.0f64, 2.0f64, 3.0f64, 4.0f64),
            Vector4(2.0f64, 4.0f64, 6.0f64, 8.0f64)
        );
    }

    #[test]
    fn test_vector_division_with_scalar() {
        assert_eq!(Vector2(2., 4.) / 2., Vector2(1., 2.));
        assert_eq!(Vector3(2., 4., 6.) / 2., Vector3(1., 2., 3.));
        assert_eq!(Vector4(2., 4., 6., 8.) / 2., Vector4(1., 2., 3., 4.));
    }

    #[test]
    fn test_vector_negation() {
        assert_eq!(-Vector2(1., 2.), Vector2(-1., -2.));
        assert_eq!(-Vector3(1., 2., 3.), Vector3(-1., -2., -3.));
        assert_eq!(-Vector4(1., 2., 3., 4.), Vector4(-1., -2., -3., -4.));
    }

    #[test]
    fn test_matrix_addition() {
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) + Matrix3x3::from_columns(
                Vector3(-1., -2., -3.),
                Vector3(-4., -5., -6.),
                Vector3(-7., -8., -9.)
            ),
            Matrix3x3::zero()
        );
    }

    #[test]
    fn test_matrix_subtraction() {
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) - Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ),
            Matrix3x3::zero()
        );
    }

    #[test]
    fn test_matrix_multiplication() {
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) * Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ),
            Matrix3x3::from_columns(
                Vector3(30., 36., 42.),
                Vector3(66., 81., 96.),
                Vector3(102., 126., 150.)
            )
        );
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) * Matrix3x3::IDENTITY,
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            )
        );
    }

    #[test]
    fn test_matrix_multiplication_with_scalar() {
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) * 2.0f32,
            Matrix3x3::from_columns(
                Vector3(2., 4., 6.),
                Vector3(8., 10., 12.),
                Vector3(14., 16., 18.)
            )
        );
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1.0f64, 2.0f64, 3.0f64),
                Vector3(4.0f64, 5.0f64, 6.0f64),
                Vector3(7.0f64, 8.0f64, 9.0f64)
            ) * 2.0f64,
            Matrix3x3::from_columns(
                Vector3(2.0f64, 4.0f64, 6.0f64),
                Vector3(8.0f64, 10.0f64, 12.0f64),
                Vector3(14.0f64, 16.0f64, 18.0f64)
            )
        );
        assert_eq!(
            2.0f32
                * Matrix3x3::from_columns(
                    Vector3(1.0f32, 2.0f32, 3.0f32),
                    Vector3(4.0f32, 5.0f32, 6.0f32),
                    Vector3(7.0f32, 8.0f32, 9.0f32)
                ),
            Matrix3x3::from_columns(
                Vector3(2.0f32, 4.0f32, 6.0f32),
                Vector3(8.0f32, 10.0f32, 12.0f32),
                Vector3(14.0f32, 16.0f32, 18.0f32)
            )
        );
        assert_eq!(
            2.0f64
                * Matrix3x3::from_columns(
                    Vector3(1.0f64, 2.0f64, 3.0f64),
                    Vector3(4.0f64, 5.0f64, 6.0f64),
                    Vector3(7.0f64, 8.0f64, 9.0f64)
                ),
            Matrix3x3::from_columns(
                Vector3(2.0f64, 4.0f64, 6.0f64),
                Vector3(8.0f64, 10.0f64, 12.0f64),
                Vector3(14.0f64, 16.0f64, 18.0f64)
            )
        );
    }

    #[test]
    fn test_matrix_division_with_scalar() {
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(2., 4., 6.),
                Vector3(8., 10., 12.),
                Vector3(14., 16., 18.)
            ) / 2.,
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            )
        );
    }

    #[test]
    fn test_matrix_negation() {
        assert_eq!(
            -Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ),
            Matrix3x3::from_columns(
                Vector3(-1., -2., -3.),
                Vector3(-4., -5., -6.),
                Vector3(-7., -8., -9.)
            )
        );
    }

    #[test]
    fn test_matrix_multiplication_with_vector() {
        assert_eq!(
            Matrix3x3::IDENTITY * Vector3(1., 2., 3.),
            Vector3(1., 2., 3.)
        );
        assert_eq!(
            Matrix3x3::from_columns(
                Vector3(1., 2., 3.),
                Vector3(4., 5., 6.),
                Vector3(7., 8., 9.)
            ) * Vector3(1., 2., 3.),
            Vector3(30., 36., 42.)
        );
    }

    #[test]
    fn test_radians_creation() {
        assert_eq!(Radians::from_radians(0.), Radians::from_radians(TWO_PI));
        assert_eq!(Radians::from_radians(0.), Radians::from_degrees(0.));
        assert_eq!(Radians::from_radians(PI), Radians::from_radians(3. * PI));
        assert_eq!(Radians::from_radians(PI), Radians::from_degrees(180.));
    }

    #[test]
    fn test_radians_cos_sin() {
        assert_eq!(Radians::from_radians(0.).cos_sin(), Vector2(1., 0.));
        assert_eq!(Radians::from_radians(PI).cos_sin(), Vector2(-1., 0.));
    }

    #[test]
    fn test_radians_addition() {
        assert_eq!(
            Radians::from_radians(PI) + Radians::from_radians(PI),
            Radians::from_radians(0.)
        );
        assert_eq!(
            Radians::from_radians(TWO_PI) + Radians::from_radians(PI),
            Radians::from_radians(PI)
        );
    }

    #[test]
    fn test_radians_subtraction() {
        assert_eq!(
            Radians::from_radians(PI) - Radians::from_radians(PI),
            Radians::from_radians(0.)
        );
        assert_eq!(
            Radians::from_radians(0.) - Radians::from_radians(PI),
            Radians::from_radians(PI)
        );
    }

    #[test]
    fn test_radians_negation() {
        assert_eq!(-Radians::from_radians(0.), Radians::from_radians(0.));
        assert_eq!(-Radians::from_radians(PI), Radians::from_radians(PI));
        assert_eq!(-Radians::from_radians(TWO_PI), Radians::from_radians(0.));
    }

    #[test]
    fn test_rotation_matrix_creation() {
        assert_eq!(
            Matrix3x3::rotation_matrix_x(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(1., 0., 0.),
                Vector3(0., 0., 1.),
                Vector3(0., -1., 0.)
            )
        );
        assert_eq!(
            Matrix3x3::rotation_matrix_x(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., -1.)
            )
        );
        assert_eq!(
            Matrix3x3::rotation_matrix_y(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(0., 0., -1.),
                Vector3(0., 1., 0.),
                Vector3(1., 0., 0.)
            )
        );
        assert_eq!(
            Matrix3x3::rotation_matrix_y(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(-1., 0., 0.),
                Vector3(0., 1., 0.),
                Vector3(0., 0., -1.)
            )
        );
        assert_eq!(
            Matrix3x3::rotation_matrix_z(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(0., 1., 0.),
                Vector3(-1., 0., 0.),
                Vector3(0., 0., 1.)
            )
        );
        assert_eq!(
            Matrix3x3::rotation_matrix_z(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(-1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., 1.)
            )
        );
    }
}
