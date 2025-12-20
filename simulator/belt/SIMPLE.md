# Lane Algorithm - Simple

Notes:

- A lane is one side of a belt.
- A belt is a collection of two lanes. Belts are less important to model than lanes. Connections are between lanes, not belts.

Each lane stores:

- A mutable array of eight items (`[<T>; 8]`), strictly ordered by position ascending, each of those items being an `Option` of a tuple with two parts:
  - A `u32` position on the lane. 0 is at the end of the lane, and the maximum as at the very start of the lane.
  - A generic `Item` type
- A `u32` `length`. The default length is 256, and the position will be out this value.
For example, with the default length, the position could be anywhere from 0 to 255. This will never change.
- An optional outwards connection to a different lane. This will never change.

## Tick Procedure

On every tick, every item attempts to move forward a number of positions along the lane. The items furthest along the lane (items with a lower position) must be processed first. Assuming 60 ticks per second, here are the positions/tick of each tier of belt type:

| Type    | P/T |
| ------- | --- |
| Regular | 8   |
| Fast    | 16  |
| Express | 24  |
| Turbo   | 32  |

An item can move forward a maximum of `n` positions. The position of item `i` must be less than or equal to the position of item `i + 1` minus 64 positions. If an item would collide with the next item ahead, it stops 64 spaces behind that item.

Under normal conditions, a lane should not have more than 4 items on it at a time. However, due to the way inserters insert into non-fully compacted lanes, there may be 5 or more items in a lane. Items may be be in an overlapping state, but you may not cause any new overlaps to be created. If an item is overlapping with a previous item, then it does not move until there is greater than 64 positions of space ahead of it (regular spacing rules), and then it moves like normal.

Tick updates read from the old state and write to a new state in a two-phase simulation. Connection transfers go into the new lane state. After all lanes update, swap buffers.
