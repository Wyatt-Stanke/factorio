use std::num::NonZeroUsize;

// Temp
type Item = NonZeroUsize;

enum BeltType {
    /// Also known as a yellow belt
    Regular,
    /// Also known as a red belt
    Fast,
    /// Also known as a blue belt
    Express,
    /// Also known as a green belt
    Turbo
}

impl BeltType {
    pub fn tiles_traveled_per_second(&self) -> f32 {
        match self {
            BeltType::Regular => 1.875,
            BeltType::Fast => 3.75,
            BeltType::Express => 5.625,
            BeltType::Turbo => 7.5,
        }
    }

    pub fn item_throughput_per_second_one_lane(&self) -> f32 {
        match self {
            BeltType::Regular => 7.5,
            BeltType::Fast => 15.0,
            BeltType::Express => 22.5,
            BeltType::Turbo => 30.0,
        }
    }
}

struct BeltLane {
    // A belt lane can have a maximum of 5 items on it at any time.
    // The tuple stores the item and its relative position on the belt (0.0 to 1.0).
    // TODO: Might be more efficent to use fixed point instead of float since it's ranged. This would
    // also allow for us to make the whole tuple non-zero which would allow for the option to be zero.
    // The way belts are simulated in the game is that items can be on one of 256 discrete positions on the belt.
    // To see more, check
    // - Factorio wiki/Belt transport system
    // - Factorio wiki/Transport Belts/Physics
    // This uses a fixed-size array for performance reasons.
    items: [Option<(Item, f32)>; 5],
    belt_type: BeltType,
    next_belt: Option<Box<BeltLane>>,
}

struct Belt {
    left_lane: BeltLane,
    right_lane: BeltLane,
}

fn main() {
    println!("Hello, world!");
}
