use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Clone, Copy, Debug, Default, PartialEq, FromPrimitive)]
pub enum PrimitiveShape {
    #[default]
    Sphere = 0,
    Cylinder = 1,
    Box = 2,
    Torus = 3,
    SierpinskiTetrahedron = 4,
    Bunny = 5,
}

impl PrimitiveShape {
    #[must_use]
    pub fn id(&self) -> u32 {
        *self as u32
    }

    // Defaults to Sphere if the id is invalid
    #[must_use]
    pub fn from_id(id: u32) -> Self {
        match FromPrimitive::from_u32(id) {
            Some(shape) => shape,
            _ => PrimitiveShape::Sphere,
        }
    }
}
