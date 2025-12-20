use crate::Coordinate;

pub trait Surface {
    type Building;

    fn get_building(&self, coord: Coordinate) -> Option<&Self::Building>;
    fn get_building_mut(&mut self, coord: Coordinate) -> Option<&mut Self::Building>;
    fn set_building(&mut self, coord: Coordinate, building: Self::Building);
    fn tick(&mut self);
}

pub trait Tickable {
    fn tick(&mut self);
}
