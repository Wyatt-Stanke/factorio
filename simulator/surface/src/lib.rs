pub mod simple;
pub mod traits;

pub use simple::*;
pub use traits::*;

/// Represents a 2D coordinate in the surface grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

/// Represents a direction for connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    #[must_use]
    pub const fn offset(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::East => (1, 0),
            Self::West => (-1, 0),
        }
    }
}

impl Coordinate {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn neighbor(self, direction: Direction) -> Self {
        let (dx, dy) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}
