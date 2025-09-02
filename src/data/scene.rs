use std::fmt;
use strum_macros::{FromRepr, EnumIter};

#[derive(Clone, Copy, Debug, Default, PartialEq, FromRepr, EnumIter)]
#[repr(u32)]
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

    #[must_use]
    pub(crate) fn from_id(id: u32) -> Option<Self> {
        FractalGroup::from_repr(id)
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

#[derive(Clone, Copy, Debug, Default, PartialEq, FromRepr, EnumIter)]
#[repr(u32)]
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

    #[must_use]
    pub(crate) fn from_id(id: u32) -> Option<Self> {
        PrimitiveShape::from_repr(id)
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
