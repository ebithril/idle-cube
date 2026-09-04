# Idle Cube TODO

## 1. Exact cube state

- [ ] Give every cubie an explicit, stable identity.
- [ ] Replace committed floating-point cubie rotations with an exact discrete orientation.
- [ ] Derive rendering quaternions from the exact orientation.
- [ ] Keep move animation separate from committed cube state.
- [ ] Add queries for locating cubies and determining which direction each sticker faces.
- [ ] Make solved-state detection use the exact cube state.
- [ ] Preserve the existing move bitflags and prime-move handling.

### Cube-state tests

- [ ] A new cube is solved.
- [ ] Every move followed by its inverse restores the cube.
- [ ] Applying any face move four times restores the cube.
- [ ] A non-trivial scramble is not solved.
- [ ] A scramble followed by its inverse restores the cube.
- [ ] Rendering orientation agrees with logical orientation after moves.

## 2. Solving framework

- [ ] Add a solver that owns an ordered list of unlocked techniques.
- [ ] Define a common technique interface for recognizing a case and returning the next short move sequence.
- [ ] Only recognize and select another technique after the current sequence finishes animating.
- [ ] Prefer the first applicable technique so faster and more specific algorithms can be placed before fallbacks.
- [ ] Include useful metadata with a solve step, such as its name and moves.

## 3. Daisy milestone

- [ ] Define and document the fixed cube orientation used by the solver.
- [ ] Add queries for finding white edge pieces and recognizing completed daisy petals.
- [ ] Implement the human-style rules for moving one white edge to the daisy without losing completed petals.
- [ ] Repeat daisy steps until all four white edges surround the yellow center.
- [ ] Display the currently selected technique or step while it executes.
- [ ] Test daisy construction against many deterministic scrambles.
- [ ] Stop after completing the daisy for the first playable solving milestone.

## 4. Complete beginner solving

- [ ] Convert the daisy into the white cross.
- [ ] Insert first-layer corners.
- [ ] Insert second-layer edges.
- [ ] Build the last-layer cross.
- [ ] Complete the remaining beginner-method last-layer steps.
- [ ] Test that the fallback repertoire solves many deterministic scrambles.

## 5. Progression and idle-game systems

- [ ] Add currency and rewards for completed solves.
- [ ] Add technique and algorithm unlocks.
- [ ] Allow direct cross to supersede the daisy route when unlocked.
- [ ] Add intuitive F2L techniques as alternatives to beginner first- and second-layer steps.
- [ ] Add individual OLL and PLL cases above the beginner fallbacks as they are learned.
- [ ] Track recognition time, turning speed, move count, and solve time.
- [ ] Add persistence for progression and settings.

## Later

- [ ] Improve scramble generation and avoid immediate cancellations or redundant turns.
- [ ] Add cube and camera controls without changing the logical cube state.
- [ ] Replace the placeholder README with accurate project and gameplay documentation.
