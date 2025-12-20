/// Represents the size of an entity in tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    /// Creates a new Size.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Creates a square Size.
    #[must_use]
    pub const fn square(side: u32) -> Self {
        Self {
            width: side,
            height: side,
        }
    }
}

/// An entity is an object that can be placed on the surface.
pub trait Entity {
    /// Returns the size of the entity in tiles.
    fn size(&self) -> Size;
}
