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
    fn items_per_second(&self) -> f32 {
        match self {
            BeltType::Regular => 1.0,
            BeltType::Fast => 1.5,
            BeltType::Express => 2.0,
            BeltType::Turbo => 3.0,
        }
    }
}

struct BeltLane {
    // A belt lane can have a maximum of 5 items on it at any time.
    // The tuple stores the item and its relative position on the belt (0.0 to 1.0).
    // TODO: Might be more efficent to use fixed point instead of float since it's ranged. This would
    // also allow for us to make the whole tuple non-zero which would allow for the option to be zero.
    // This uses a fixed-size array for performance reasons.
    items: [Option<(Item, f32)>; 5],
    belt_type: BeltType,
}

struct Belt {
    left_lane: BeltLane,
    right_lane: BeltLane,
}

fn main() {
    println!("Hello, world!");
}
