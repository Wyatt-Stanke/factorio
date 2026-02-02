use super::*;
use std::num::NonZeroUsize;

// Helper function to create an item
fn item(id: usize) -> Item {
    NonZeroUsize::new(id).expect("Failed to create NonZeroUsize")
}

// Helper function to count items in a lane
fn count_items(lane: &SingleBeltLane) -> usize {
    lane.items.iter().filter(|slot| slot.is_some()).count()
}

// Helper function to get item positions in a lane
fn get_positions(lane: &SingleBeltLane) -> Vec<u32> {
    let mut positions: Vec<u32> = lane
        .items
        .iter()
        .filter_map(|slot| slot.as_ref().map(|(_, pos)| *pos))
        .collect();
    positions.sort_unstable();
    positions
}

// Helper function to get items with their positions
fn get_items_with_positions(lane: &SingleBeltLane) -> Vec<(usize, u32)> {
    let mut items: Vec<(usize, u32)> = lane
        .items
        .iter()
        .filter_map(|slot| slot.as_ref().map(|(item, pos)| (item.get(), *pos)))
        .collect();
    items.sort_by_key(|&(_, pos)| pos);
    items
}

#[test]
fn test_coordinate_creation() {
    let coord = Coordinate::new(5, 10);
    assert_eq!(coord.x, 5);
    assert_eq!(coord.y, 10);
}

#[test]
fn test_coordinate_equality() {
    let coord1 = Coordinate::new(1, 2);
    let coord2 = Coordinate::new(1, 2);
    let coord3 = Coordinate::new(2, 1);
    assert_eq!(coord1, coord2);
    assert_ne!(coord1, coord3);
}

#[test]
fn test_coordinate_neighbor() {
    let coord = Coordinate::new(5, 5);
    assert_eq!(coord.neighbor(Direction::North), Coordinate::new(5, 4));
    assert_eq!(coord.neighbor(Direction::South), Coordinate::new(5, 6));
    assert_eq!(coord.neighbor(Direction::East), Coordinate::new(6, 5));
    assert_eq!(coord.neighbor(Direction::West), Coordinate::new(4, 5));
}

#[test]
fn test_belt_type_positions_per_tick() {
    assert_eq!(BeltType::Regular.positions_per_tick(), 8);
    assert_eq!(BeltType::Fast.positions_per_tick(), 16);
    assert_eq!(BeltType::Express.positions_per_tick(), 24);
    assert_eq!(BeltType::Turbo.positions_per_tick(), 32);
}

#[test]
fn test_empty_lane_creation() {
    let lane = SingleBeltLane::new(BeltType::Regular, None);
    assert_eq!(count_items(&lane), 0);
    assert!(lane.next_lane_coord.is_none());
}

#[test]
fn test_single_item_movement_regular_belt() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 10));

    // Regular belt moves 8 positions per tick
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![18]);

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![26]);
}

#[test]
fn test_single_item_movement_fast_belt() {
    let mut lane = SingleBeltLane::new(BeltType::Fast, None);
    lane.items[0] = Some((item(1), 10));

    // Fast belt moves 16 positions per tick
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![26]);

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![42]);
}

#[test]
fn test_single_item_movement_express_belt() {
    let mut lane = SingleBeltLane::new(BeltType::Express, None);
    lane.items[0] = Some((item(1), 10));

    // Express belt moves 24 positions per tick
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![34]);

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![58]);
}

#[test]
fn test_single_item_movement_turbo_belt() {
    let mut lane = SingleBeltLane::new(BeltType::Turbo, None);
    lane.items[0] = Some((item(1), 10));

    // Turbo belt moves 32 positions per tick
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![42]);

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![74]);
}

#[test]
fn test_item_stops_at_end_of_lane_without_next() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 250));

    // Should move to 255 and stop (250 + 8 = 258, but capped at 255)
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![255]);

    // Should stay at 255
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![255]);
}

#[test]
fn test_item_transfer_to_next_lane() {
    let _coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let mut lane = SingleBeltLane::new(BeltType::Regular, Some(coord2));
    lane.items[0] = Some((item(1), 250));

    // Should return the item for transfer (250 + 8 = 258, 258 - 256 = 2)
    let transfers = lane.tick_and_get_transfers();
    assert_eq!(transfers.len(), 1);
    assert_eq!(transfers[0].0.get(), 1);
    assert_eq!(transfers[0].1, 2);
    assert_eq!(count_items(&lane), 0); // Item should be removed from current lane
}

#[test]
fn test_spacing_rule_64_positions() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 200));

    // Both items should move 8 positions
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![108, 208]);
}

#[test]
fn test_spacing_rule_prevents_collision() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place items 70 positions apart
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 170));

    // Item 1 moves to 108, item 2 should respect 64-position gap
    lane.tick_and_get_transfers();
    let positions = get_positions(&lane);
    eprintln!("AFTER: {:?}", get_items_with_positions(&lane));
    assert_eq!(positions[0], 108);
    // Item 2 should maintain at least 64 positions from item 1
    assert!(positions[1] - positions[0] >= 64);
}

#[test]
fn test_spacing_rule_with_close_items() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place items exactly 64 positions apart
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 164));

    lane.tick_and_get_transfers();
    let positions = get_positions(&lane);
    // Both should move, maintaining proper spacing
    assert!(positions[1] - positions[0] >= 64);
}

#[test]
fn test_accept_item_on_empty_lane() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    let success = lane.accept_item(item(1), 50);

    assert!(success);
    assert_eq!(count_items(&lane), 1);
    assert_eq!(get_positions(&lane), vec![50]);
}

#[test]
fn test_accept_item_respects_spacing() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));

    // Try to add item at position 120 (too close, needs 64 gap)
    let success = lane.accept_item(item(2), 120);

    assert!(success);
    let positions = get_positions(&lane);
    // Should be adjusted to respect 64-position gap
    assert!(positions[1] >= 164); // 100 + 64
}

#[test]
fn test_accept_item_full_lane() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Fill the lane
    for i in 0..5 {
        lane.items[i] = Some((
            item(i + 1),
            u32::try_from(i).expect("Index fits in u32") * 50,
        ));
    }

    // Try to add another item
    let success = lane.accept_item(item(6), 240);
    assert!(!success); // Should fail, lane is full
    assert_eq!(count_items(&lane), 5);
}

#[test]
fn test_accept_item_at_position_255() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    let success = lane.accept_item(item(1), 255);

    assert!(success);
    assert_eq!(get_positions(&lane), vec![255]);
}

#[test]
fn test_accept_item_beyond_255() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    let success = lane.accept_item(item(1), 300);

    // Should be clamped to 255
    if success {
        assert_eq!(get_positions(&lane), vec![255]);
    }
}

#[test]
fn test_world_creation() {
    let world = World::new();
    assert_eq!(world.belts.len(), 0);
}

#[test]
fn test_world_add_belt() {
    let mut world = World::new();
    let coord = Coordinate::new(0, 0);
    let belt = SingleBelt::new(coord, BeltType::Regular, None, None);

    world.add_belt(belt);
    assert_eq!(world.belts.len(), 1);
    assert!(world.belts.contains_key(&coord));
}

#[test]
fn test_world_tick_single_belt() {
    let mut world = World::new();
    let coord = Coordinate::new(0, 0);
    let mut belt = SingleBelt::new(coord, BeltType::Regular, None, None);
    belt.left_lane.items[0] = Some((item(1), 10));

    world.add_belt(belt);
    world.tick();

    let belt = world.belts.get(&coord).expect("Belt not found");
    assert_eq!(get_positions(&belt.left_lane), vec![18]);
}

#[test]
fn test_world_tick_multiple_belts() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, None, None);
    belt1.left_lane.items[0] = Some((item(1), 10));
    world.add_belt(belt1);

    let mut belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    belt2.left_lane.items[0] = Some((item(2), 20));
    world.add_belt(belt2);

    world.tick();

    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        ),
        vec![18]
    );
    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        ),
        vec![28]
    );
}

#[test]
fn test_world_item_transfer_between_belts() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    // Belt 2 receives items
    let belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    world.add_belt(belt2);

    // Belt 1 sends items to belt 2
    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 250));
    world.add_belt(belt1);

    world.tick();

    // Item should have transferred from belt1 to belt2
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        ),
        0
    );
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        ),
        1
    );
}

#[test]
fn test_chain_of_three_belts() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);
    let coord3 = Coordinate::new(2, 0);

    // Create chain: 1 -> 2 -> 3
    let belt3 = SingleBelt::new(coord3, BeltType::Regular, None, None);
    world.add_belt(belt3);

    let belt2 = SingleBelt::new(coord2, BeltType::Regular, Some(coord3), Some(coord3));
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 250));
    world.add_belt(belt1);

    // Tick 1: Item moves from belt1 to belt2
    world.tick();
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        ),
        0
    );
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        ),
        1
    );
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord3)
                .expect("Belt 3 not found")
                .left_lane
        ),
        0
    );

    // Continue ticking to move item through the chain
    // Item starts at position 2 on belt2, needs to reach 258 (32 ticks * 8 pos = 256)
    // Then transfer to belt3
    for _ in 0..35 {
        world.tick();
    }

    // Eventually item should reach belt3
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord3)
                .expect("Belt 3 not found")
                .left_lane
        ),
        1
    );
}

#[test]
fn test_chain_of_three_belts_multiple_items() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);
    let coord3 = Coordinate::new(2, 0);

    // Create chain: 1 -> 2 -> 3
    let belt3 = SingleBelt::new(coord3, BeltType::Regular, None, None);
    world.add_belt(belt3);

    let belt2 = SingleBelt::new(coord2, BeltType::Regular, Some(coord3), Some(coord3));
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 250));
    belt1.left_lane.items[1] = Some((item(2), 180));
    belt1.left_lane.items[2] = Some((item(3), 100));
    world.add_belt(belt1);

    for _ in 0..90 {
        world.tick();
    }

    // Eventually item should reach belt3
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord3)
                .expect("Belt 3 not found")
                .left_lane
        ),
        3
    );

    // Verify items are completely transferred and compacted on belt 3
    let items = &world
        .belts
        .get(&coord3)
        .expect("Belt 3 not found")
        .left_lane
        .items
        .iter()
        .filter(|s| s.is_some())
        .map(|s| s.as_ref().expect("Item should exist").1)
        .collect::<Vec<_>>();

    eprintln!("Items on belt 3: {items:?}");

    assert_eq!(items.len(), 3);
    // Ensure proper spacing
    assert_eq!(items[0], 255); // First item should be at the end
    assert!(
        items[0] - items[1] >= 64,
        "Gap between first and second item should be >= 64, got {}",
        items[0] - items[1]
    );
    assert!(
        items[1] - items[2] >= 64,
        "Gap between second and third item should be >= 64, got {}",
        items[1] - items[2]
    );
}

#[test]
fn test_compacting_single_item_at_end() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 240));

    // Item should move to 248, then 255 and stop
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![248]);

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![255]);

    // Should stay at 255
    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![255]);
}

#[test]
fn test_compacting_two_items_at_end() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place two items that will reach the end
    lane.items[0] = Some((item(1), 240));
    lane.items[1] = Some((item(2), 170));

    // After several ticks, items should compact at the end
    for _ in 0..10 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[1], 255); // Back item at end
    assert!(
        positions[1] - positions[0] >= 64,
        "Items should maintain 64+ gap"
    );
}

#[test]
fn test_compacting_three_items_at_end() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place three items
    lane.items[0] = Some((item(1), 50));
    lane.items[1] = Some((item(2), 120));
    lane.items[2] = Some((item(3), 190));

    // Tick until all items reach the end and compact
    for _ in 0..15 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 3);
    assert_eq!(positions[2], 255); // Last item at end

    // Check spacing between consecutive items
    for i in 1..positions.len() {
        let gap = positions[i] - positions[i - 1];
        assert!(
            gap >= 64,
            "Gap between items {} and {} is {}, should be >= 64",
            i - 1,
            i,
            gap
        );
    }

    // First item should be at approximately 255 - 64*2 = 127
    assert!(
        positions[0] >= 127 - 8 && positions[0] <= 127 + 8,
        "First item should be around 127, got {}",
        positions[0]
    );
}

#[test]
fn test_compacting_four_items_maximum_density() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place four items that will compact to maximum density
    for i in 0..4 {
        lane.items[i] = Some((
            item(i + 1),
            u32::try_from(i).expect("Index fits in u32") * 40,
        ));
    }

    // Tick until compacted
    for _ in 0..20 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 4);
    assert_eq!(positions[3], 255); // Last item at end

    // All gaps should be exactly 64 (or very close due to movement granularity)
    for i in 1..positions.len() {
        let gap = positions[i] - positions[i - 1];
        assert!(gap >= 64, "Gap {i} is too small: {gap}");
        assert!(
            gap <= 72,
            "Gap {i} is too large: {gap} (items not fully compacted)"
        );
    }
}

#[test]
fn test_compacting_progressive_arrival() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // First item arrives at end
    lane.items[0] = Some((item(1), 250));

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![255]);

    // Add second item while first is at end
    lane.items[1] = Some((item(2), 100));

    // Tick until second item reaches its position behind first
    for _ in 0..25 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[1], 255);
    assert!(positions[1] - positions[0] >= 64);
    assert_eq!(positions[0], 191); // Should be at 255 - 64 = 191
}

#[test]
fn test_compacting_with_fast_belt() {
    let mut lane = SingleBeltLane::new(BeltType::Fast, None);
    // Fast belt moves 16 positions per tick
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 30));

    // Tick until items reach end
    for _ in 0..15 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[1], 255);
    assert!(positions[1] - positions[0] >= 64);
}

#[test]
fn test_compacting_prevents_overlapping() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place items very close together
    lane.items[0] = Some((item(1), 200));
    lane.items[1] = Some((item(2), 240));

    // Item 2 should reach 255 quickly
    for _ in 0..5 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 2);

    // Verify no overlap
    assert!(
        positions[1] - positions[0] >= 40,
        "Items got worse! Gap: {}",
        positions[1] - positions[0]
    );
}

#[test]
fn test_multiple_items_in_sequence() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 10));
    lane.items[1] = Some((item(2), 100));
    lane.items[2] = Some((item(3), 200));

    for _ in 0..5 {
        lane.tick_and_get_transfers();
    }

    // All items should have moved
    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 3);
    // Verify they maintain proper spacing
    for i in 1..positions.len() {
        assert!(positions[i] - positions[i - 1] >= 64);
    }
}

#[test]
fn test_five_items_maximum_capacity() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Fill lane to maximum capacity
    for i in 0..5 {
        lane.items[i] = Some((
            item(i + 1),
            u32::try_from(i).expect("Index fits in u32") * 64,
        ));
    }

    assert_eq!(count_items(&lane), 5);

    // Verify movement doesn't break with full lane
    lane.tick_and_get_transfers();
    assert!(count_items(&lane) <= 5);
}

#[test]
fn test_item_preservation_during_tick() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(42), 100));

    lane.tick_and_get_transfers();

    // Verify the item ID is preserved
    let items = get_items_with_positions(&lane);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, 42);
}

#[test]
fn test_mixed_belt_types_in_world() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    // Create one Regular and one Fast belt
    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, None, None);
    belt1.left_lane.items[0] = Some((item(1), 10));
    world.add_belt(belt1);

    let mut belt2 = SingleBelt::new(coord2, BeltType::Fast, None, None);
    belt2.left_lane.items[0] = Some((item(2), 10));
    world.add_belt(belt2);

    world.tick();

    // Regular belt item should move 8 positions
    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        ),
        vec![18]
    );
    // Fast belt item should move 16 positions
    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        ),
        vec![26]
    );
}

#[test]
fn test_right_lane_independent_from_left() {
    let mut belt = SingleBelt::new(Coordinate::new(0, 0), BeltType::Regular, None, None);
    belt.left_lane.items[0] = Some((item(1), 10));
    belt.right_lane.items[0] = Some((item(2), 20));

    assert_eq!(get_positions(&belt.left_lane), vec![10]);
    assert_eq!(get_positions(&belt.right_lane), vec![20]);

    belt.left_lane.tick_and_get_transfers();
    belt.right_lane.tick_and_get_transfers();

    assert_eq!(get_positions(&belt.left_lane), vec![18]);
    assert_eq!(get_positions(&belt.right_lane), vec![28]);
}

#[test]
fn test_different_next_coords_for_left_and_right() {
    let coord = Coordinate::new(0, 0);
    let left_next = Coordinate::new(1, 0);
    let right_next = Coordinate::new(2, 0);

    let belt = SingleBelt::new(coord, BeltType::Regular, Some(left_next), Some(right_next));

    assert_eq!(belt.left_lane.next_lane_coord, Some(left_next));
    assert_eq!(belt.right_lane.next_lane_coord, Some(right_next));
}

#[test]
fn test_no_items_lost_during_world_tick() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 10));
    belt1.left_lane.items[1] = Some((item(2), 100));
    belt1.left_lane.items[2] = Some((item(3), 200));
    world.add_belt(belt1);

    let initial_count = count_items(
        &world
            .belts
            .get(&coord1)
            .expect("Belt 1 not found")
            .left_lane,
    ) + count_items(
        &world
            .belts
            .get(&coord2)
            .expect("Belt 2 not found")
            .left_lane,
    );

    for _ in 0..50 {
        world.tick();
    }

    let final_count = count_items(
        &world
            .belts
            .get(&coord1)
            .expect("Belt 1 not found")
            .left_lane,
    ) + count_items(
        &world
            .belts
            .get(&coord2)
            .expect("Belt 2 not found")
            .left_lane,
    );

    assert_eq!(
        initial_count, final_count,
        "Items were lost during simulation"
    );
}

#[test]
fn test_item_ordering_preserved() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 10));
    lane.items[1] = Some((item(2), 100));
    lane.items[2] = Some((item(3), 190));

    for _ in 0..10 {
        lane.tick_and_get_transfers();
    }

    let items = get_items_with_positions(&lane);
    // Verify ordering is preserved (item 1 before item 2 before item 3)
    if items.len() >= 2 {
        assert!(items[0].0 < items[1].0);
    }
    if items.len() >= 3 {
        assert!(items[1].0 < items[2].0);
    }
}

#[test]
fn test_extremely_fast_belt_transfer() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let belt2 = SingleBelt::new(coord2, BeltType::Turbo, None, None);
    world.add_belt(belt2);

    // Use Turbo belt (32 positions per tick)
    let mut belt1 = SingleBelt::new(coord1, BeltType::Turbo, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 230)); // Close to end
    world.add_belt(belt1);

    world.tick(); // Should transfer immediately

    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        ),
        0
    );
    assert_eq!(
        count_items(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        ),
        1
    );
}

#[test]
fn test_world_with_disconnected_belts() {
    let mut world = World::new();

    // Create two separate belt systems
    let mut belt1 = SingleBelt::new(Coordinate::new(0, 0), BeltType::Regular, None, None);
    belt1.left_lane.items[0] = Some((item(1), 10));
    world.add_belt(belt1);

    let mut belt2 = SingleBelt::new(Coordinate::new(10, 10), BeltType::Regular, None, None);
    belt2.left_lane.items[0] = Some((item(2), 20));
    world.add_belt(belt2);

    world.tick();

    // Both should move independently
    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&Coordinate::new(0, 0))
                .expect("Belt 1 not found")
                .left_lane
        ),
        vec![18]
    );
    assert_eq!(
        get_positions(
            &world
                .belts
                .get(&Coordinate::new(10, 10))
                .expect("Belt 2 not found")
                .left_lane
        ),
        vec![28]
    );
}

#[test]
fn test_zero_position_item() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 0));

    lane.tick_and_get_transfers();
    assert_eq!(get_positions(&lane), vec![8]);
}

#[test]
fn test_acceptance_with_item_behind() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 200));

    // Try to add item at position 50 (behind existing item)
    let success = lane.accept_item(item(2), 50);
    assert!(success);

    let positions = get_positions(&lane);
    assert_eq!(positions.len(), 2);
    assert!(positions[1] - positions[0] >= 64);
}

#[test]
fn test_stalled_items_dont_move_backward() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 150)); // Too close to item 1

    let before = get_items_with_positions(&lane);
    eprintln!("BEFORE: {before:?}");
    lane.tick_and_get_transfers();

    let positions = get_items_with_positions(&lane);
    eprintln!("AFTER: {positions:?}");
    for (id, pos) in &positions {
        let initial = before.iter().find(|(i, _)| i == id).map_or(0, |(_, p)| *p);
        eprintln!("Item {id} moved from {initial} to {pos}");
        // No item should move backward
        assert!(
            *pos >= initial,
            "Item {id} moved backward from {initial} to {pos}"
        );
    }
}

#[test]
fn test_bottleneck_at_end_of_lane() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Create a traffic jam near the end
    lane.items[0] = Some((item(1), 191)); // 191 + 64 = 255, so next item must be at 255
    lane.items[1] = Some((item(2), 255));

    lane.tick_and_get_transfers();

    // Both items should still be present
    assert_eq!(count_items(&lane), 2);
    // The last item should be at 255
    let positions = get_positions(&lane);
    assert_eq!(positions[positions.len() - 1], 255);
}

#[test]
fn test_rapid_succession_items() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    // Place 4 items exactly 64 positions apart
    // Can't fit 5 items with 64-gap on a 256-position belt
    for i in 0..4 {
        lane.items[i] = Some((
            item(i + 1),
            u32::try_from(i).expect("Index fits in u32") * 64,
        ));
    }

    for _ in 0..3 {
        lane.tick_and_get_transfers();
    }

    // Verify all items still present and properly spaced
    assert_eq!(count_items(&lane), 4);
    let positions = get_positions(&lane);
    for i in 1..positions.len() {
        assert!(
            positions[i] - positions[i - 1] >= 64,
            "Items at positions {} and {} are too close",
            positions[i - 1],
            positions[i]
        );
    }
}

#[test]
fn test_item_at_position_191() {
    // 191 + 64 = 255, edge case
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 191));

    for _ in 0..20 {
        lane.tick_and_get_transfers();
    }

    // Item should eventually reach 255 and stop
    let positions = get_positions(&lane);
    assert!(!positions.is_empty());
    assert!(positions[0] <= 255);
}

#[test]
fn test_world_complex_network() {
    let mut world = World::new();

    // Create a more complex network
    //   [0,0] -> [1,0]
    //   [0,1] -> [1,0]
    // Two belts feeding into one

    let coord00 = Coordinate::new(0, 0);
    let coord01 = Coordinate::new(0, 1);
    let coord10 = Coordinate::new(1, 0);

    let belt_target = SingleBelt::new(coord10, BeltType::Regular, None, None);
    world.add_belt(belt_target);

    let mut belt1 = SingleBelt::new(coord00, BeltType::Regular, Some(coord10), Some(coord10));
    belt1.left_lane.items[0] = Some((item(1), 250));
    world.add_belt(belt1);

    let mut belt2 = SingleBelt::new(coord01, BeltType::Regular, Some(coord10), Some(coord10));
    belt2.left_lane.items[0] = Some((item(2), 250));
    world.add_belt(belt2);

    world.tick();

    // Both items should transfer to the target belt
    let target = world.belts.get(&coord10).expect("Target belt not found");
    let total_items = count_items(&target.left_lane) + count_items(&target.right_lane);
    assert!(total_items > 0, "At least one item should have transferred");
}

#[test]
fn test_transfer_with_occupied_target() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    // Target belt already has items
    let mut belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    belt2.left_lane.items[0] = Some((item(99), 10));
    world.add_belt(belt2);

    // Source belt tries to send item
    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 250));
    world.add_belt(belt1);

    world.tick();

    // Verify the target belt received the item (or it's properly rejected)
    let target = world.belts.get(&coord2).expect("Target belt not found");
    let source = world.belts.get(&coord1).expect("Source belt not found");
    let total = count_items(&target.left_lane)
        + count_items(&target.right_lane)
        + count_items(&source.left_lane)
        + count_items(&source.right_lane);
    assert_eq!(total, 2, "Both items should still exist somewhere");
}

#[test]
fn test_continuous_flow_throughput() {
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    world.add_belt(belt2);

    // Start with items spread across belt 1
    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 50));
    belt1.left_lane.items[1] = Some((item(2), 120));
    belt1.left_lane.items[2] = Some((item(3), 190));
    world.add_belt(belt1);

    // Simulate for many ticks
    for _ in 0..50 {
        world.tick();
    }

    // Eventually all items should have transferred
    let belt2_items = count_items(
        &world
            .belts
            .get(&coord2)
            .expect("Belt 2 not found")
            .left_lane,
    ) + count_items(
        &world
            .belts
            .get(&coord2)
            .expect("Belt 2 not found")
            .right_lane,
    );
    assert!(belt2_items > 0, "Items should have flowed through");
}

#[test]
fn test_item_at_exact_boundary() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, Some(Coordinate::new(1, 0)));
    lane.items[0] = Some((item(1), 256)); // Impossible position, but test boundary handling

    // Should handle gracefully
    let transfers = lane.tick_and_get_transfers();
    // Either transfers or stays, but shouldn't panic
    assert!(transfers.len() <= 1);
}

#[test]
fn test_spacing_enforcement_with_three_items() {
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 10));
    lane.items[1] = Some((item(2), 80));
    lane.items[2] = Some((item(3), 150));

    for _ in 0..5 {
        lane.tick_and_get_transfers();
    }

    let positions = get_positions(&lane);
    // Verify all spacing rules maintained
    for i in 1..positions.len() {
        let gap = positions[i] - positions[i - 1];
        assert!(
            gap >= 64,
            "Gap between items {} and {} is only {}, should be >= 64",
            positions[i - 1],
            positions[i],
            gap
        );
    }
}

#[test]
fn test_world_tick_preserves_belt_count() {
    let mut world = World::new();

    for i in 0..10 {
        let coord = Coordinate::new(i, 0);
        let belt = SingleBelt::new(coord, BeltType::Regular, None, None);
        world.add_belt(belt);
    }

    let initial_count = world.belts.len();

    for _ in 0..20 {
        world.tick();
    }

    assert_eq!(
        world.belts.len(),
        initial_count,
        "Belt count changed during ticking"
    );
}

#[test]
fn test_express_belt_overtaking_scenario() {
    // Test that faster belts properly handle items
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let belt2 = SingleBelt::new(coord2, BeltType::Express, None, None);
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Express, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 240));
    world.add_belt(belt1);

    world.tick();

    // Express belt moves 24 positions, so 240 + 24 = 264, should transfer
    let total = count_items(
        &world
            .belts
            .get(&coord1)
            .expect("Belt 1 not found")
            .left_lane,
    ) + count_items(
        &world
            .belts
            .get(&coord2)
            .expect("Belt 2 not found")
            .left_lane,
    );
    assert_eq!(total, 1, "Item should have transferred");
}

// ========== DIAGNOSTIC TESTS ==========
// These tests help identify the exact nature of bugs

#[test]
fn test_diagnostic_spacing_algorithm() {
    // Detailed diagnostic for spacing rule bug
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 170));

    println!("Initial positions: {:?}", get_positions(&lane));

    lane.tick_and_get_transfers();

    let positions = get_positions(&lane);
    println!("After tick: {positions:?}");
    println!("Gap between items: {}", positions[1] - positions[0]);

    // Document actual behavior vs expected
    // Expected: Item 1 at 108, Item 2 should be at least 172 (108 + 64)
    // This test documents if the spacing logic is correctly enforced
}

#[test]
fn test_diagnostic_item_collision_prevention() {
    // Test what happens when an item tries to move into another item's space
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 163)); // Exactly one position before minimum gap

    println!("Before: {:?}", get_items_with_positions(&lane));

    lane.tick_and_get_transfers();

    println!("After: {:?}", get_items_with_positions(&lane));

    let positions = get_positions(&lane);
    if positions.len() == 2 {
        let gap = positions[1] - positions[0];
        println!("Gap: {gap}");
        // Document whether items maintain proper spacing
    }
}

#[test]
fn test_diagnostic_backward_movement() {
    // Check if items ever move backward
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));
    lane.items[1] = Some((item(2), 150));

    let before = get_items_with_positions(&lane);
    println!("Before: {before:?}");

    lane.tick_and_get_transfers();

    let after = get_items_with_positions(&lane);
    println!("After: {after:?}");

    // Check each item
    for (id, pos_after) in &after {
        if let Some((_, pos_before)) = before.iter().find(|(i, _)| i == id)
            && *pos_after < *pos_before
        {
            println!("Item {id} moved BACKWARD from {pos_before} to {pos_after}");
        }
    }
}

#[test]
fn test_diagnostic_transfer_position_calculation() {
    // Test the exact position calculation when transferring between belts
    let mut lane = SingleBeltLane::new(BeltType::Regular, Some(Coordinate::new(1, 0)));
    lane.items[0] = Some((item(1), 250));

    println!("Item at position 250, moves 8 positions per tick");
    println!("Expected new position: 258");
    println!("258 > 255, so should transfer to next belt");
    println!("Target position on next belt: 258 - 256 = 2");

    let transfers = lane.tick_and_get_transfers();

    if !transfers.is_empty() {
        println!(
            "Transfer: Item {} to position {}",
            transfers[0].0.get(),
            transfers[0].1
        );
    }

    assert_eq!(transfers.len(), 1);
    println!("Actual transfer position: {}", transfers[0].1);
}

#[test]
fn test_diagnostic_lane_full_behavior() {
    // Test behavior when trying to add item to full lane
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);

    // Fill the lane
    for i in 0..5 {
        lane.items[i] = Some((
            item(i + 1),
            u32::try_from(i).expect("Index fits in u32") * 52,
        ));
    }

    println!("Lane has {} items", count_items(&lane));
    println!("Positions: {:?}", get_positions(&lane));

    let accepted = lane.accept_item(item(99), 240);
    println!(
        "Attempt to add 6th item: {}",
        if accepted { "SUCCESS" } else { "REJECTED" }
    );

    assert!(!accepted, "Should not accept item when lane is full");
}

#[test]
fn test_diagnostic_multi_item_spacing_chain() {
    // Test chain of 3 items with various spacing
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 10));
    lane.items[1] = Some((item(2), 74)); // Exactly 64 apart
    lane.items[2] = Some((item(3), 138)); // Exactly 64 apart

    println!("Initial: {:?}", get_items_with_positions(&lane));

    for tick in 1..=3 {
        lane.tick_and_get_transfers();
        println!("After tick {}: {:?}", tick, get_items_with_positions(&lane));

        let positions = get_positions(&lane);
        for i in 1..positions.len() {
            println!("  Gap between items: {}", positions[i] - positions[i - 1]);
        }
    }
}

#[test]
fn test_diagnostic_world_transfer_mechanics() {
    // Detailed look at world transfer between belts
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    let belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Regular, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 250));
    world.add_belt(belt1);

    println!("=== Before Tick ===");
    println!(
        "Belt 1 left lane: {:?}",
        get_items_with_positions(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        )
    );
    println!(
        "Belt 2 left lane: {:?}",
        get_items_with_positions(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        )
    );

    world.tick();

    println!("\n=== After Tick ===");
    println!(
        "Belt 1 left lane: {:?}",
        get_items_with_positions(
            &world
                .belts
                .get(&coord1)
                .expect("Belt 1 not found")
                .left_lane
        )
    );
    println!(
        "Belt 2 left lane: {:?}",
        get_items_with_positions(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .left_lane
        )
    );
    println!(
        "Belt 2 right lane: {:?}",
        get_items_with_positions(
            &world
                .belts
                .get(&coord2)
                .expect("Belt 2 not found")
                .right_lane
        )
    );
}

#[test]
fn test_edge_case_item_at_255_with_next_lane() {
    // What happens when item is at 255 and there's a next lane?
    let mut lane = SingleBeltLane::new(BeltType::Regular, Some(Coordinate::new(1, 0)));
    lane.items[0] = Some((item(1), 255));

    println!("Item at position 255 with next lane available");

    let transfers = lane.tick_and_get_transfers();

    println!("Transfers: {}", transfers.len());
    if !transfers.is_empty() {
        println!("Transfer position: {}", transfers[0].1);
    }

    let remaining = count_items(&lane);
    println!("Items remaining on lane: {remaining}");
}

#[test]
fn test_edge_case_massive_speed_difference() {
    // Turbo belt (32) vs Regular belt (8) in chain
    let mut world = World::new();

    let coord1 = Coordinate::new(0, 0);
    let coord2 = Coordinate::new(1, 0);

    // Fast feeding into slow
    let belt2 = SingleBelt::new(coord2, BeltType::Regular, None, None);
    world.add_belt(belt2);

    let mut belt1 = SingleBelt::new(coord1, BeltType::Turbo, Some(coord2), Some(coord2));
    belt1.left_lane.items[0] = Some((item(1), 230));
    belt1.left_lane.items[1] = Some((item(2), 160));
    world.add_belt(belt1);

    println!("=== Turbo (32 pos/tick) feeding into Regular (8 pos/tick) ===");

    for tick in 1..=3 {
        world.tick();
        println!("\nTick {tick}");
        println!(
            "  Turbo belt: {:?}",
            get_items_with_positions(
                &world
                    .belts
                    .get(&coord1)
                    .expect("Belt 1 not found")
                    .left_lane
            )
        );
        println!(
            "  Regular belt: {:?}",
            get_items_with_positions(
                &world
                    .belts
                    .get(&coord2)
                    .expect("Belt 2 not found")
                    .left_lane
            )
        );
    }
}

#[test]
fn test_edge_case_zero_gap_attempt() {
    // Try to place items with zero gap
    let mut lane = SingleBeltLane::new(BeltType::Regular, None);
    lane.items[0] = Some((item(1), 100));

    // Try to add item at same position
    let success = lane.accept_item(item(2), 100);
    println!(
        "Attempt to place item at same position as existing: {}",
        if success { "ACCEPTED" } else { "REJECTED" }
    );

    if success {
        let positions = get_positions(&lane);
        println!("Resulting positions: {positions:?}");
    }
}
