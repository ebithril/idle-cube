# Idle Cube TODO

## 1. Exact cube state

- [x] Give every cubie an explicit, stable identity.
- [x] Replace committed floating-point cubie rotations with an exact discrete orientation.
- [x] Derive rendering quaternions from the exact orientation.
- [x] Keep move animation separate from committed cube state.
- [x] Add the first exact-state queries for identifying white edges and their sticker direction.
- [x] Make solved-state detection use the exact cube state.
- [x] Preserve the existing move bitflags and prime-move handling.

### Cube-state tests

- [x] A new cube is solved.
- [x] Every move followed by its inverse restores the cube.
- [x] Applying any face move four times restores the cube.
- [x] A non-trivial scramble is not solved.
- [x] A scramble followed by its inverse restores the cube.
- [x] Rendering orientation agrees with logical orientation after moves.

## 2. Solving framework

- [ ] Add a solver that owns an ordered list of unlocked techniques.
- [ ] Define a common technique interface for recognizing a case and returning the next short move sequence.
- [ ] Only recognize and select another technique after the current sequence finishes animating.
- [ ] Prefer the first applicable technique so faster and more specific algorithms can be placed before fallbacks.
- [ ] Include useful metadata with a solve step, such as its name and moves.

## 3. Daisy milestone

- [ ] Define and document the fixed cube orientation used by the solver.
- [x] Add queries for finding white edge pieces and recognizing completed daisy petals.
- [x] Move one white edge to the daisy at a time without losing completed petals.
- [x] Repeat daisy steps until all four white edges surround the yellow center.
- [ ] Display the currently selected technique or step while it executes.
- [x] Test daisy construction against many deterministic scrambles.
- [x] Stop after completing the daisy for the first playable solving milestone.

## 4. Complete beginner solving

- [x] Convert the daisy into the white cross.
- [x] Insert first-layer corners.
- [x] Insert second-layer edges.
- [x] Build the last-layer cross.
- [x] Permute the last-layer edges.
- [x] Position the last-layer corners.
- [x] Orient the last-layer corners and complete the beginner solve.
- [ ] Test that the fallback repertoire solves many deterministic scrambles.

## 5. Progression and idle-game systems

- [x] Automatically start another scramble after each completed solve.
- [ ] Add currency and rewards for completed solves.
- [ ] Add technique and algorithm unlocks.
- [ ] Allow direct cross to supersede the daisy route when unlocked.
- [ ] Add intuitive F2L techniques as alternatives to beginner first- and second-layer steps.
- [ ] Add individual OLL and PLL cases above the beginner fallbacks as they are learned.
- [x] Track move count and solve time.
- [x] Track rolling AO3, AO5, AO20, and AO100 statistics.
- [ ] Track recognition time and turning speed.
- [ ] Add persistence for progression and settings.

## Later

- [x] Improve scramble generation and avoid immediate cancellations or redundant turns.
- [ ] Animate the whole-cube rotation into the white-down solving orientation.
- [ ] Add cube and camera controls without changing the logical cube state.
- [ ] Replace the placeholder README with accurate project and gameplay documentation.
