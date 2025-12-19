use std::{collections::HashMap, num::NonZeroUsize};

// Temp
type Item = NonZeroUsize;

/// Represents a 2D coordinate in the world grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate {
    x: i32,
    y: i32,
}

/// Represents a direction for belt connections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn offset(&self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

impl Coordinate {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn neighbor(&self, direction: Direction) -> Self {
        let (dx, dy) = direction.offset();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeltType {
    /// Also known as a yellow belt
    Regular,
    /// Also known as a red belt
    Fast,
    /// Also known as a blue belt
    Express,
    /// Also known as a green belt
    Turbo,
}

impl BeltType {
    const fn tiles_traveled_per_second(&self) -> f32 {
        match self {
            Self::Regular => 1.875,
            Self::Fast => 3.75,
            Self::Express => 5.625,
            Self::Turbo => 7.5,
        }
    }

    const fn item_throughput_per_second_one_lane(&self) -> f32 {
        match self {
            Self::Regular => 7.5,
            Self::Fast => 15.0,
            Self::Express => 22.5,
            Self::Turbo => 30.0,
        }
    }

    const fn positions_per_tick(&self) -> u32 {
        match self {
            Self::Regular => 8,
            Self::Fast => 16,
            Self::Express => 24,
            Self::Turbo => 32,
        }
    }
}

struct SingleBeltLane {
    // A belt lane can have a maximum of 5 items on it at any time.
    // The tuple stores the item and its relative position on the belt (0 to 255).
    // The way belts are simulated in the game is that items can be on one of 256 discrete positions on the belt.
    // To see more, check
    // - Factorio wiki/Belt transport system
    // - Factorio wiki/Transport Belts/Physics
    // This uses a fixed-size array for performance reasons.
    items: [Option<(Item, u32)>; 5],
    belt_type: BeltType,
    /// Coordinate of the next lane in the chain
    next_lane_coord: Option<Coordinate>,
}

impl SingleBeltLane {
    fn new(belt_type: BeltType, next_lane_coord: Option<Coordinate>) -> Self {
        Self {
            items: [None, None, None, None, None],
            belt_type,
            next_lane_coord,
        }
    }

    /// Returns items that should be transferred to the next lane
    /// Returns a list of (item, position) tuples
    fn tick_and_get_transfers(&mut self) -> Vec<(Item, u32)> {
        let mut transfers = Vec::new();
        let positions_per_tick = self.belt_type.positions_per_tick();

        // Collect all items with their array indices
        let mut items_with_idx: Vec<(usize, Item, u32)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| slot.as_ref().map(|(item, pos)| (idx, *item, *pos)))
            .collect();

        // Sort by position in reverse order (front items first)
        items_with_idx.sort_by_key(|&(_, _, pos)| std::cmp::Reverse(pos));

        // Track new positions for each item: (idx, actual_pos, spacing_pos)
        // spacing_pos is used for spacing calculations by items behind
        let mut new_positions: Vec<(usize, u32, u32)> = Vec::new();

        // Process items from front to back
        for (i, (idx, _item, current_pos)) in items_with_idx.iter().enumerate() {
            let desired_position = current_pos + positions_per_tick;
            let mut can_move_to = desired_position;

            // Check if moving would violate spacing with any item ahead
            for &(_check_idx, _, ahead_pos) in &new_positions {
                // ahead_pos is where an item ahead will be after this tick
                if ahead_pos > *current_pos {
                    // Check if moving to desired_position would be too close
                    if desired_position + 64 > ahead_pos {
                        // Would violate spacing - move as close as possible while maintaining 64-gap
                        // This allows items to compact when the front item stops
                        // But never move backward - stay at current position if that would happen
                        let max_forward = if ahead_pos > 64 { ahead_pos - 64 } else { 0 };
                        can_move_to = max_forward.max(*current_pos);
                        break;
                    }
                }
            }

            // Store the calculated position for spacing checks
            // For items beyond 255 without next lane, store 255 for spacing
            let spacing_pos = if can_move_to > 255 && self.next_lane_coord.is_none() {
                255
            } else {
                can_move_to
            };

            new_positions.push((*idx, can_move_to, spacing_pos));
        }

        // Apply the new positions
        for (idx, new_pos, _) in new_positions {
            if let Some((item, position)) = &mut self.items[idx] {
                if new_pos > 255 {
                    // Transfer to next lane
                    let target_position = new_pos - 256;
                    if self.next_lane_coord.is_some() {
                        transfers.push((*item, target_position));
                        self.items[idx] = None;
                    } else {
                        // No next lane, clamp to 255
                        *position = 255;
                    }
                } else {
                    *position = new_pos;
                }
            }
        }

        transfers
    }

    /// Attempts to accept an item from a previous lane
    /// Returns true if successful, false if there's no space
    fn accept_item(&mut self, item: Item, target_position: u32) -> bool {
        // Check if target position respects the 64 position gap rule
        let mut adjusted_position = target_position.min(255);

        // Check distance to existing items
        for slot in &self.items {
            if let Some((_, pos)) = slot {
                if *pos < adjusted_position {
                    let distance = adjusted_position - pos;
                    if distance < 64 {
                        adjusted_position = pos + 64;
                    }
                }
            }
        }

        if adjusted_position <= 255 {
            // Find an empty slot
            if let Some(empty_slot) = self.items.iter_mut().find(|slot| slot.is_none()) {
                *empty_slot = Some((item, adjusted_position));
                return true;
            }
        }
        false
    }
}

struct SingleBelt {
    left_lane: SingleBeltLane,
    right_lane: SingleBeltLane,
    coordinate: Coordinate,
}

impl SingleBelt {
    fn new(
        coordinate: Coordinate,
        belt_type: BeltType,
        left_next: Option<Coordinate>,
        right_next: Option<Coordinate>,
    ) -> Self {
        Self {
            left_lane: SingleBeltLane::new(belt_type, left_next),
            right_lane: SingleBeltLane::new(belt_type, right_next),
            coordinate,
        }
    }
}

/// The world contains all belts organized by their coordinates
struct World {
    belts: HashMap<Coordinate, SingleBelt>,
}

impl World {
    fn new() -> Self {
        Self {
            belts: HashMap::new(),
        }
    }

    fn add_belt(&mut self, belt: SingleBelt) {
        self.belts.insert(belt.coordinate, belt);
    }

    fn get_lane_mut(&mut self, coord: Coordinate, is_left: bool) -> Option<&mut SingleBeltLane> {
        self.belts.get_mut(&coord).map(|belt| {
            if is_left {
                &mut belt.left_lane
            } else {
                &mut belt.right_lane
            }
        })
    }

    /// Tick all belts in the world
    fn tick(&mut self) {
        // Collect all transfers first
        let mut all_transfers: Vec<(Coordinate, Item, u32)> = Vec::new();

        // Process all lanes and collect transfers
        for belt in self.belts.values_mut() {
            let left_transfers = belt.left_lane.tick_and_get_transfers();
            for (item, pos) in left_transfers {
                if let Some(next_coord) = belt.left_lane.next_lane_coord {
                    all_transfers.push((next_coord, item, pos));
                }
            }

            let right_transfers = belt.right_lane.tick_and_get_transfers();
            for (item, pos) in right_transfers {
                if let Some(next_coord) = belt.right_lane.next_lane_coord {
                    all_transfers.push((next_coord, item, pos));
                }
            }
        }

        // Apply all transfers
        for (target_coord, item, position) in all_transfers {
            // Try to find the target belt and accept the item
            // For now, we'll assume we're transferring to the left lane of the target
            // A more complete implementation would track which lane to transfer to
            if let Some(target_belt) = self.belts.get_mut(&target_coord) {
                // Try left lane first, then right lane if that fails
                if !target_belt.left_lane.accept_item(item, position) {
                    target_belt.right_lane.accept_item(item, position);
                }
            }
        }
    }
}

fn main() {
    // Create a world with a chain of belts
    let mut world = World::new();

    // Create coordinates for a line of belts
    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);
    let coord3 = Coordinate::new(2, 0);

    // Create belt 3 (end of the chain)
    let belt3 = SingleBelt::new(coord3, BeltType::Regular, None, None);
    world.add_belt(belt3);

    // Create belt 2 (middle)
    let belt2 = SingleBelt::new(coord2, BeltType::Regular, Some(coord3), Some(coord3));
    world.add_belt(belt2);

    // Create belt 1 (start) with some items
    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items = [
        Some((
            NonZeroUsize::new(1).expect("Failed to create NonZeroUsize"),
            20,
        )),
        Some((
            NonZeroUsize::new(2).expect("Failed to create NonZeroUsize"),
            160,
        )),
        None,
        None,
        None,
    ];
    world.add_belt(belt1);

    println!("World initialized with {} belts", world.belts.len());
    println!("Initial state:");
    print_world_state(&world);

    // Simulate a few ticks
    for tick in 0..10 {
        world.tick();
        println!("\nTick {} completed:", tick + 1);
        print_world_state(&world);
    }
}

fn print_world_state(world: &World) {
    for (coord, belt) in &world.belts {
        // println!("  Belt at ({}, {}):", coord.x, coord.y);
        print!("    Left lane: ");
        for item_opt in &belt.left_lane.items {
            if let Some((item, pos)) = item_opt {
                print!("[Item {} at pos {}] ", item.get(), pos);
            }
        }
        println!();
        print!("    Right lane: ");
        for item_opt in &belt.right_lane.items {
            if let Some((item, pos)) = item_opt {
                print!("[Item {} at pos {}] ", item.get(), pos);
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests;
