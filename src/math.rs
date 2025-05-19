use std::ops::{Add, Div, Mul, Neg, Sub};

pub use std::f32::consts::PI;
pub const TWO_PI: f32 = 2. * PI;
// Accuracy of 0.0001 is good enough for our graphics
pub const EPSILON: f32 = 1.0e-4;

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector2<T>(pub T, pub T);

impl<T> Vector2<T> {
    pub fn extend(self, x: T) -> Vector3<T> {
        Vector3(self.0, self.1, x)
    }
}

impl<T> PartialEq for Vector2<T>
where
    T: num_traits::Float,
{
    fn eq(&self, other: &Self) -> bool {
        let epsilon_t =
            T::from(EPSILON).expect("Can only compare values that can be cast to from f32");

        (self.0 - other.0).abs() < epsilon_t && (self.1 - other.1).abs() < epsilon_t
    }
}

impl<T> Add<Self> for Vector2<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T> Sub<Self> for Vector2<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T> Mul<Self> for Vector2<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T>,
{
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        self.0 * rhs.0 + self.1 * rhs.1
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl Mul<Vector2<f32>> for f32 {
    type Output = Vector2<f32>;

    fn mul(self, rhs: Vector2<f32>) -> Self::Output {
        Vector2(self * rhs.0, self * rhs.1)
    }
}

impl Mul<Vector2<f64>> for f64 {
    type Output = Vector2<f64>;

    fn mul(self, rhs: Vector2<f64>) -> Self::Output {
        Vector2(self * rhs.0, self * rhs.1)
    }
}

impl<T> Div<T> for Vector2<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs)
    }
}

impl<T> Neg for Vector2<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector3<T>(pub T, pub T, pub T);

impl<T> Vector3<T> {
    pub fn extend(self, x: T) -> Vector4<T> {
        Vector4(self.0, self.1, self.2, x)
    }

    pub fn shrink(self) -> Vector2<T> {
        Vector2(self.0, self.1)
    }
}

impl<T> PartialEq for Vector3<T>
where
    T: num_traits::Float,
{
    fn eq(&self, other: &Self) -> bool {
        let epsilon_t =
            T::from(EPSILON).expect("Can only compare values that can be cast to from f32");

        (self.0 - other.0).abs() < epsilon_t
            && (self.1 - other.1).abs() < epsilon_t
            && (self.2 - other.2).abs() < epsilon_t
    }
}

impl<T> Add<Self> for Vector3<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T> Sub<Self> for Vector3<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T> Mul<Self> for Vector3<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T>,
{
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl<T> Mul<T> for Vector3<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<Vector3<f32>> for f32 {
    type Output = Vector3<f32>;

    fn mul(self, rhs: Vector3<f32>) -> Self::Output {
        Vector3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl Mul<Vector3<f64>> for f64 {
    type Output = Vector3<f64>;

    fn mul(self, rhs: Vector3<f64>) -> Self::Output {
        Vector3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl<T> Div<T> for Vector3<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl<T> Neg for Vector3<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector4<T>(pub T, pub T, pub T, pub T);

impl<T> Vector4<T> {
    pub fn shrink(self) -> Vector3<T> {
        Vector3(self.0, self.1, self.2)
    }
}

impl<T> PartialEq for Vector4<T>
where
    T: num_traits::Float,
{
    fn eq(&self, other: &Self) -> bool {
        let epsilon_t =
            T::from(EPSILON).expect("Can only compare values that can be cast to from f32");

        (self.0 - other.0).abs() < epsilon_t
            && (self.1 - other.1).abs() < epsilon_t
            && (self.2 - other.2).abs() < epsilon_t
            && (self.3 - other.3).abs() < epsilon_t
    }
}

impl<T> Add<Self> for Vector4<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}

impl<T> Sub<Self> for Vector4<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl<T> Mul<Self> for Vector4<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T>,
{
    type Output = T;

    fn mul(self, rhs: Self) -> Self::Output {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2 + self.3 * rhs.3
    }
}

impl<T> Mul<T> for Vector4<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs, self.3 * rhs)
    }
}

impl Mul<Vector4<f32>> for f32 {
    type Output = Vector4<f32>;

    fn mul(self, rhs: Vector4<f32>) -> Self::Output {
        Vector4(self * rhs.0, self * rhs.1, self * rhs.2, self * rhs.3)
    }
}

impl Mul<Vector4<f64>> for f64 {
    type Output = Vector4<f64>;

    fn mul(self, rhs: Vector4<f64>) -> Self::Output {
        Vector4(self * rhs.0, self * rhs.1, self * rhs.2, self * rhs.3)
    }
}

impl<T> Div<T> for Vector4<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
    }
}

impl<T> Neg for Vector4<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2, -self.3)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Matrix3x3<T>(Vector3<T>, Vector3<T>, Vector3<T>);

impl<T> PartialEq for Matrix3x3<T>
where
    T: num_traits::Float,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl<T> Matrix3x3<T> {
    pub fn from_columns(col1: Vector3<T>, col2: Vector3<T>, col3: Vector3<T>) -> Self {
        Self(col1, col2, col3)
    }

    pub fn get_columns(&self) -> (Vector3<T>, Vector3<T>, Vector3<T>)
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

    pub fn get_rows(&self) -> (Vector3<T>, Vector3<T>, Vector3<T>)
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

impl<T> Add<Self> for Matrix3x3<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T> Sub<Self> for Matrix3x3<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl<T> Mul<Self> for Matrix3x3<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let rows = self.get_rows();

        Self(
            Vector3(rows.0 * rhs.0, rows.1 * rhs.0, rows.2 * rhs.0),
            Vector3(rows.0 * rhs.1, rows.1 * rhs.1, rows.2 * rhs.1),
            Vector3(rows.0 * rhs.2, rows.1 * rhs.2, rows.2 * rhs.2),
        )
    }
}

impl<T> Mul<T> for Matrix3x3<T>
where
    T: Mul<T, Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<Matrix3x3<f32>> for f32 {
    type Output = Matrix3x3<f32>;

    fn mul(self, rhs: Matrix3x3<f32>) -> Self::Output {
        Matrix3x3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl Mul<Matrix3x3<f64>> for f64 {
    type Output = Matrix3x3<f64>;

    fn mul(self, rhs: Matrix3x3<f64>) -> Self::Output {
        Matrix3x3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl<T> Div<T> for Matrix3x3<T>
where
    T: Div<T, Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl<T> Neg for Matrix3x3<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}

impl<T> Mul<Vector3<T>> for Matrix3x3<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
    type Output = Vector3<T>;

    fn mul(self, rhs: Vector3<T>) -> Self::Output {
        let rows = self.get_rows();

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

    pub fn get_rotation_matrix_x(angle: Radians) -> Self {
        let Vector2(cos, sin) = angle.cos_sin();

        Self(
            Vector3(1., 0., 0.),
            Vector3(0., cos, sin),
            Vector3(0., -sin, cos),
        )
    }

    pub fn get_rotation_matrix_y(angle: Radians) -> Self {
        let Vector2(cos, sin) = angle.cos_sin();

        Self(
            Vector3(cos, 0., -sin),
            Vector3(0., 1., 0.),
            Vector3(sin, 0., cos),
        )
    }

    pub fn get_rotation_matrix_z(angle: Radians) -> Self {
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

    pub fn get_radians(&self) -> f32 {
        self.0
    }

    pub fn get_degrees(&self) -> f32 {
        (self.get_radians() / PI) * 180.0
    }

    // Clamps its value to [min, max]
    pub fn clamp(self, min: f32, max: f32) -> Radians {
        Radians::from_radians(self.get_radians().clamp(min, max))
    }

    // Returns the same angle, but on [0, 2PI]
    pub fn standardize(self) -> Self {
        Self(((self.get_radians() % TWO_PI) + TWO_PI) % TWO_PI)
    }

    pub fn cos(&self) -> f32 {
        self.get_radians().cos()
    }

    pub fn sin(&self) -> f32 {
        self.get_radians().sin()
    }

    pub fn cos_sin(&self) -> Vector2<f32> {
        Vector2(self.get_radians().cos(), self.get_radians().sin())
    }
}

impl PartialEq for Radians {
    fn eq(&self, other: &Self) -> bool {
        let diff = (self.get_radians() - other.get_radians()).abs() % TWO_PI;

        !(EPSILON..=TWO_PI - EPSILON).contains(&diff)
    }
}

impl Add<Self> for Radians {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.get_radians() + rhs.get_radians())
    }
}

impl Sub<Self> for Radians {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_radians(self.get_radians() - rhs.get_radians())
    }
}

impl Neg for Radians {
    type Output = Radians;

    fn neg(self) -> Self::Output {
        Self::from_radians(-self.get_radians())
    }
}

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
            Matrix3x3::get_rotation_matrix_x(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(1., 0., 0.),
                Vector3(0., 0., 1.),
                Vector3(0., -1., 0.)
            )
        );
        assert_eq!(
            Matrix3x3::get_rotation_matrix_x(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., -1.)
            )
        );
        assert_eq!(
            Matrix3x3::get_rotation_matrix_y(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(0., 0., -1.),
                Vector3(0., 1., 0.),
                Vector3(1., 0., 0.)
            )
        );
        assert_eq!(
            Matrix3x3::get_rotation_matrix_y(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(-1., 0., 0.),
                Vector3(0., 1., 0.),
                Vector3(0., 0., -1.)
            )
        );
        assert_eq!(
            Matrix3x3::get_rotation_matrix_z(Radians::from_radians(PI / 2.)),
            Matrix3x3(
                Vector3(0., 1., 0.),
                Vector3(-1., 0., 0.),
                Vector3(0., 0., 1.)
            )
        );
        assert_eq!(
            Matrix3x3::get_rotation_matrix_z(Radians::from_radians(PI)),
            Matrix3x3(
                Vector3(-1., 0., 0.),
                Vector3(0., -1., 0.),
                Vector3(0., 0., 1.)
            )
        );
    }
}
