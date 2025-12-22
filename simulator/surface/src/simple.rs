use crate::{Coordinate, Surface, Tickable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimpleSurface<T> {
    grid: HashMap<Coordinate, T>,
}

impl<T> Default for SimpleSurface<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SimpleSurface<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
        }
    }
}

impl<T> Surface for SimpleSurface<T>
where
    T: Tickable,
{
    type Building = T;

    fn get_building(&self, coord: Coordinate) -> Option<&Self::Building> {
        self.grid.get(&coord)
    }

    fn get_building_mut(&mut self, coord: Coordinate) -> Option<&mut Self::Building> {
        self.grid.get_mut(&coord)
    }

    fn set_building(&mut self, coord: Coordinate, building: Self::Building) {
        self.grid.insert(coord, building);
    }

    fn tick(&mut self) {
        for item in self.grid.values_mut() {
            item.tick();
        }
    }
}
