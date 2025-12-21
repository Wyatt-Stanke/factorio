# Lane Algorithm - Simple

## Model Overview

- A *lane* is one side of a belt.
- A *belt* is a collection of two lanes, but belt grouping is not important to simulation. All connections are between lanes, not belts.

## Lane State

Each lane maintains its own state. A lane contains the following immutable parameters:

- `length: u32` - spatial resolution of the lane. Default: `256`.
- `connection_out: Option<KaneId>` - a fixed connection to at most one destination lane. If None, items cannot leave the lane, and must stop at the end.

Each lane contains the following mutable, per-tick state:

- `items: [Option<(position: u32, item: Item)>; 8]`
  - Always sorted by ascending position (lower positions first).
  - Positions range `0 .. length-1` (`0..255` on default).
  - Each occupied slot represents one item.
  - At most eight entrie, typically 4 or less.

## Position

Position 0 is the end of the lane (exit point), and position `length − 1` is the start of the lane (entry point). Items move toward position 0, and must maintain at least 64 units of spacing relative to the item ahead.

## Tick Model

Simulation is two-buffered:

1. Read from `current` lane state.
2. Produce a complete `next` lane state..
3. After all lanes complete their write phase, swap the buffers.

Tick order does not matter, because all items read from old state.

## Movement Speed per Belt Tier

Each item moves forward up to `S` position units per tick:  

| Belt Type | Speed S (pos/tick) |
| --------- | ------------------ |
| Regular   | 8                  |
| Fast      | 16                 |
| Express   | 24                 |
| Turbo     | 32                 |

Speed is lane-wide and constant.

## Item Movement Rules

Items must be processed in ascending position order, in which the item closest to position `0` moves first.

- $pos_i$ = current position of item `i`
- $pos_{next}$ = position of the item with the highest position on the connecting lane, if applicable
- $v$ = movement speed for this lane type
- $\Delta$ = maximum allowed spacing distance = 64

Then, as follows:

### 1. Compute tentative movement

For each item:

$$newpos_{i}=pos_i-v$$

### 2. Apply spacing constraints

For each item except the lead item, enforce that

$$newpos_{i} \ge newpos_{(i-1)} + \Delta$$

If not, reduce movement so that

$$newpos_{i} = newpos_{(i-1)} + \Delta$$

### 3. Overlap behavior

If an item begins the tick overlapping the item ahead ($pos_i \le pos_{i-1} + \Delta$), then the item does not move at all this tick unless $pos_i \ge pos_{i-1} + \Delta$ due to movement of the item ahead. Items may be created in a way that is overlapping, but simulating belts should not create any more overlaps then the previous tick.
