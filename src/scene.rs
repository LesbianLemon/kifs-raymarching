#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PrimitiveShape {
    Sphere,
    Cylinder,
    Box,
    Torus,
    SierpinskiTetrahedron,
    Bunny,
}

impl PrimitiveShape {
    pub fn id(&self) -> u32 {
        match self {
            Self::Sphere => 0,
            Self::Cylinder => 1,
            Self::Box => 2,
            Self::Torus => 3,
            Self::SierpinskiTetrahedron => 4,
            Self::Bunny => 5,
        }
    }

    // Defaults to Sphere if the id is invalid
    pub fn from_id(id: u32) -> Self {
        match id {
            0 => Self::Sphere,
            1 => Self::Cylinder,
            2 => Self::Box,
            3 => Self::Torus,
            4 => Self::SierpinskiTetrahedron,
            5 => Self::Bunny,
            _ => Self::Sphere,
        }
    }
}
