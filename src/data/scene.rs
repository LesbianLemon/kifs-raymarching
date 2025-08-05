use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, FromPrimitive)]
pub(crate) enum FractalGroup {
    #[default]
    KaleidoscopicIFS = 0,
    JuliaSet = 1,
    GeneralizedJuliaSet = 2,
}

impl FractalGroup {
    #[must_use]
    pub(crate) fn id(self) -> u32 {
        self as u32
    }

    // Defaults to FractalGroup::KaleidoscopicIFS if the id is invalid
    #[must_use]
    pub(crate) fn from_id(id: u32) -> Self {
        match FractalGroup::from_u32(id) {
            Some(group) => group,
            _ => FractalGroup::KaleidoscopicIFS,
        }
    }
}

impl fmt::Display for FractalGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FractalGroup::KaleidoscopicIFS => write!(f, "Kaleidoscopic IFS"),
            FractalGroup::JuliaSet => write!(f, "Julia Set"),
            FractalGroup::GeneralizedJuliaSet => write!(f, "Generalized Julia Set"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, FromPrimitive)]
pub(crate) enum PrimitiveShape {
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
    pub(crate) fn id(self) -> u32 {
        self as u32
    }

    // Defaults to PrimitiveShape::Sphere if the id is invalid
    #[must_use]
    pub(crate) fn from_id(id: u32) -> Self {
        match FromPrimitive::from_u32(id) {
            Some(shape) => shape,
            _ => PrimitiveShape::Sphere,
        }
    }
}

impl fmt::Display for PrimitiveShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveShape::Sphere => write!(f, "Sphere"),
            PrimitiveShape::Cylinder => write!(f, "Cylinder"),
            PrimitiveShape::Box => write!(f, "Box"),
            PrimitiveShape::Torus => write!(f, "Torus"),
            PrimitiveShape::SierpinskiTetrahedron => write!(f, "Sierpinski Tetrahedron"),
            PrimitiveShape::Bunny => write!(f, "Bunny"),
        }
    }
}
