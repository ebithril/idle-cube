use crate::cube::{Cube, Direction};
use crate::moves::Move;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

const MOVES: [Move; 18] = [
    Move::U,
    Move::UPrime,
    Move::R,
    Move::RPrime,
    Move::F,
    Move::FPrime,
    Move::D,
    Move::DPrime,
    Move::L,
    Move::LPrime,
    Move::B,
    Move::BPrime,
    Move::U2,
    Move::R2,
    Move::F2,
    Move::D2,
    Move::L2,
    Move::B2,
];

const MAX_DAISY_STEP_LENGTH: usize = 5;
const MAX_DIRECT_CROSS_STEP_LENGTH: usize = 7;
const MAX_F2L_STEP_LENGTH: usize = 18;
const MAX_CORNER_STEP_LENGTH: usize = 4;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SkillId {
    BuildDaisy,
    DaisyToCross,
    DirectCross,
    IntuitiveF2L,
    BeginnerCornerInsertion,
    BeginnerSecondLayerInsertion,
    YellowCross,
    PermuteYellowEdges,
    PositionYellowCorners,
    OrientYellowCorners,
}

impl SkillId {
    pub fn name(self) -> &'static str {
        match self {
            Self::BuildDaisy => "Build daisy",
            Self::DaisyToCross => "Daisy to cross",
            Self::DirectCross => "Direct cross",
            Self::IntuitiveF2L => "Intuitive F2L",
            Self::BeginnerCornerInsertion => "Beginner corner insertion",
            Self::BeginnerSecondLayerInsertion => "Beginner second-layer insertion",
            Self::YellowCross => "Yellow cross",
            Self::PermuteYellowEdges => "Permute yellow edges",
            Self::PositionYellowCorners => "Position yellow corners",
            Self::OrientYellowCorners => "Orient yellow corners",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolveStep {
    pub skill: SkillId,
    pub name: &'static str,
    pub moves: Vec<Move>,
}

impl SolveStep {
    fn new(skill: SkillId, moves: Vec<Move>) -> Self {
        Self {
            skill,
            name: skill.name(),
            moves,
        }
    }
}

pub struct DirectCrossSolver;

impl DirectCrossSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if cube.is_white_cross_complete() {
            return None;
        }

        let completed_edges = cube.white_cross_count();
        let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
        let mut visited = HashSet::from([cube.state_key()]);

        while let Some((state, algorithm)) = pending.pop_front() {
            if algorithm.len() == MAX_DIRECT_CROSS_STEP_LENGTH {
                continue;
            }

            for move_to_make in MOVES {
                if algorithm
                    .last()
                    .is_some_and(|previous: &Move| previous.same_face(move_to_make))
                {
                    continue;
                }

                let mut next = state.clone();
                next.execute_move(move_to_make);
                if !visited.insert(next.state_key()) {
                    continue;
                }

                let mut next_algorithm = algorithm.clone();
                next_algorithm.push(move_to_make);
                if next.white_cross_count() > completed_edges {
                    return Some(SolveStep::new(SkillId::DirectCross, next_algorithm));
                }
                pending.push_back((next, next_algorithm));
            }
        }

        None
    }
}

pub struct DaisySolver;

impl DaisySolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if cube.is_daisy_complete() {
            return None;
        }

        let completed_petals = cube.daisy_count();
        let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
        let mut visited = HashSet::from([cube.state_key()]);

        while let Some((state, algorithm)) = pending.pop_front() {
            if algorithm.len() == MAX_DAISY_STEP_LENGTH {
                continue;
            }

            for move_to_make in MOVES {
                let mut next = state.clone();
                next.execute_move(move_to_make);

                if next.daisy_count() < completed_petals || !visited.insert(next.state_key()) {
                    continue;
                }

                let mut next_algorithm = algorithm.clone();
                next_algorithm.push(move_to_make);

                if next.daisy_count() > completed_petals {
                    return Some(SolveStep::new(SkillId::BuildDaisy, next_algorithm));
                }

                pending.push_back((next, next_algorithm));
            }
        }

        None
    }
}

pub struct DaisyToCrossSolver;

impl DaisyToCrossSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if cube.is_white_cross_complete() || cube.daisy_count() + cube.white_cross_count() != 4 {
            return None;
        }

        const DAISY_POSITIONS: [(usize, usize, usize, Direction, Move); 4] = [
            (1, 0, 0, Direction::Back, Move::B2),
            (0, 0, 1, Direction::Left, Move::L2),
            (2, 0, 1, Direction::Right, Move::R2),
            (1, 0, 2, Direction::Front, Move::F2),
        ];

        for (x, y, z, position_side, insertion) in DAISY_POSITIONS {
            if cube.white_edge_home_side_at(x, y, z) == Some(position_side) {
                return Some(SolveStep::new(SkillId::DaisyToCross, vec![insertion]));
            }
        }

        Some(SolveStep::new(SkillId::DaisyToCross, vec![Move::D]))
    }
}

pub struct FirstLayerCornerSolver;

impl FirstLayerCornerSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.is_white_cross_complete() || cube.is_first_layer_complete() {
            return None;
        }

        let completed_corners = cube.first_layer_corner_count();
        let (target_corner, first_side, second_side) = cube.first_unsolved_layer_corner_sides()?;

        if cube.piece_layer(target_corner) == 2 {
            return find_corner_extraction(cube, target_corner, completed_corners)
                .map(|moves| SolveStep::new(SkillId::BeginnerCornerInsertion, moves));
        }

        let (current_x, _, current_z) = cube.piece_position(target_corner);
        let (home_x, _, home_z) = cube.home_position(target_corner);
        if (current_x, current_z) != (home_x, home_z) {
            for adjustment in [[Move::D].as_slice(), &[Move::DPrime], &[Move::D2]] {
                let mut adjusted = cube.clone();
                for &move_to_make in adjustment {
                    adjusted.execute_move(move_to_make);
                }
                let (x, _, z) = adjusted.piece_position(target_corner);
                if (x, z) == (home_x, home_z) {
                    return Some(SolveStep::new(
                        SkillId::BeginnerCornerInsertion,
                        adjustment.to_vec(),
                    ));
                }
            }
        }

        let side_faces = [move_for_side(first_side), move_for_side(second_side)];
        for side in side_faces
            .into_iter()
            .flat_map(|side| [side, side.inverse()])
        {
            for down in [Move::D, Move::DPrime] {
                for trigger in [
                    vec![side, down, side.inverse()],
                    vec![side, down, side.inverse(), down.inverse()],
                ] {
                    let mut attempt = cube.clone();
                    let mut repeated_trigger = Vec::new();
                    for _ in 0..6 {
                        for &move_to_make in &trigger {
                            attempt.execute_move(move_to_make);
                        }
                        repeated_trigger.extend_from_slice(&trigger);
                        if !attempt.is_white_cross_complete()
                            || attempt.first_layer_corner_count() < completed_corners
                        {
                            break;
                        }
                        if attempt.is_piece_solved(target_corner) {
                            return Some(SolveStep::new(
                                SkillId::BeginnerCornerInsertion,
                                repeated_trigger,
                            ));
                        }
                    }
                }
            }
        }

        None
    }
}

pub struct SecondLayerEdgeSolver;

impl SecondLayerEdgeSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.is_first_layer_complete() || cube.is_second_layer_complete() {
            return None;
        }

        find_second_layer_step(cube)
            .map(|moves| SolveStep::new(SkillId::BeginnerSecondLayerInsertion, moves))
    }
}

pub struct IntuitiveF2LSolver;

struct F2LSearchNode {
    cost: usize,
    order: usize,
    cube: Cube,
    moves: Vec<Move>,
}

impl PartialEq for F2LSearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.order == other.order
    }
}

impl Eq for F2LSearchNode {}

impl PartialOrd for F2LSearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F2LSearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| other.order.cmp(&self.order))
    }
}

impl IntuitiveF2LSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.is_white_cross_complete() || cube.is_second_layer_complete() {
            return None;
        }

        let completed_pairs = cube.f2l_pair_count();
        let actions = intuitive_f2l_actions();
        let mut pending = BinaryHeap::from([F2LSearchNode {
            cost: 0,
            order: 0,
            cube: cube.clone(),
            moves: Vec::new(),
        }]);
        let mut best_cost = HashMap::from([(cube.state_key(), 0)]);
        let mut order = 1;

        while let Some(node) = pending.pop() {
            if node.cost >= MAX_F2L_STEP_LENGTH {
                continue;
            }

            for action in &actions {
                let next_cost = node.cost + action.len();
                if next_cost > MAX_F2L_STEP_LENGTH {
                    continue;
                }

                let mut next = node.cube.clone();
                for &move_to_make in action {
                    next.execute_move(move_to_make);
                }
                if !next.is_white_cross_complete() || next.f2l_pair_count() < completed_pairs {
                    continue;
                }

                let state_key = next.state_key();
                if best_cost
                    .get(&state_key)
                    .is_some_and(|&known_cost| known_cost <= next_cost)
                {
                    continue;
                }
                best_cost.insert(state_key, next_cost);

                let mut next_algorithm = node.moves.clone();
                next_algorithm.extend(action);
                if next.f2l_pair_count() > completed_pairs {
                    return Some(SolveStep::new(SkillId::IntuitiveF2L, next_algorithm));
                }
                pending.push(F2LSearchNode {
                    cost: next_cost,
                    order,
                    cube: next,
                    moves: next_algorithm,
                });
                order += 1;
            }
        }

        None
    }
}

fn intuitive_f2l_actions() -> Vec<Vec<Move>> {
    let mut actions = vec![vec![Move::D], vec![Move::DPrime], vec![Move::D2]];
    for side in [Move::R, Move::F, Move::L, Move::B] {
        for side_turn in [side, side.inverse()] {
            for top_turn in [Move::D, Move::DPrime, Move::D2] {
                actions.push(vec![side_turn, top_turn, side_turn.inverse()]);
            }
        }
    }
    actions
}

fn find_second_layer_step(cube: &Cube) -> Option<Vec<Move>> {
    if !cube.is_white_cross_complete() || cube.is_second_layer_complete() {
        return None;
    }

    let completed_edges = cube.second_layer_edge_count();
    let completed_corners = cube.first_layer_corner_count();
    let edges_on_down = cube.second_layer_edges_on_down_count();
    let mut ejection = None;

    for setup in [vec![], vec![Move::D], vec![Move::DPrime], vec![Move::D2]] {
        for insertion in second_layer_insertions() {
            let algorithm: Vec<_> = setup.iter().chain(insertion.iter()).copied().collect();
            let mut attempt = cube.clone();
            for &move_to_make in &algorithm {
                attempt.execute_move(move_to_make);
            }

            if !attempt.is_white_cross_complete()
                || attempt.first_layer_corner_count() < completed_corners
                || attempt.second_layer_edge_count() < completed_edges
            {
                continue;
            }
            if attempt.second_layer_edge_count() > completed_edges {
                return Some(algorithm);
            }
            if attempt.second_layer_edges_on_down_count() > edges_on_down && ejection.is_none() {
                ejection = Some(algorithm);
            }
        }
    }

    ejection
}

pub struct YellowCrossSolver;

impl YellowCrossSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.is_second_layer_complete() || cube.is_yellow_cross_complete() {
            return None;
        }

        let completed_edges = cube.yellow_cross_count();
        let mut actions = Vec::new();
        for setup in [vec![], vec![Move::D], vec![Move::DPrime], vec![Move::D2]] {
            for algorithm in yellow_cross_algorithms() {
                let step: Vec<_> = setup.iter().chain(algorithm.iter()).copied().collect();
                actions.push(step);
            }
        }

        let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
        let mut visited = HashSet::from([cube.state_key()]);
        while let Some((state, algorithm)) = pending.pop_front() {
            if algorithm.len() == 2 {
                continue;
            }

            for action in &actions {
                let mut next = state.clone();
                for &move_to_make in action {
                    next.execute_move(move_to_make);
                }
                if !next.is_second_layer_complete() || !visited.insert(next.state_key()) {
                    continue;
                }

                let mut next_algorithm = algorithm.clone();
                next_algorithm.push(action.clone());
                if next.yellow_cross_count() > completed_edges {
                    return Some(SolveStep::new(
                        SkillId::YellowCross,
                        next_algorithm.into_iter().flatten().collect(),
                    ));
                }
                pending.push_back((next, next_algorithm));
            }
        }

        None
    }
}

pub struct LastLayerEdgeSolver;

impl LastLayerEdgeSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.is_yellow_cross_complete() || cube.are_last_layer_edges_complete() {
            return None;
        }

        let completed_edges = cube.last_layer_edge_count();
        let mut actions = Vec::new();
        for setup in [vec![], vec![Move::D], vec![Move::DPrime], vec![Move::D2]] {
            for algorithm in last_layer_edge_algorithms() {
                actions.push(
                    setup
                        .iter()
                        .chain(algorithm.iter())
                        .copied()
                        .collect::<Vec<_>>(),
                );
            }
        }

        let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
        let mut visited = HashSet::from([cube.state_key()]);
        while let Some((state, algorithms)) = pending.pop_front() {
            if algorithms.len() == 3 {
                continue;
            }

            for action in &actions {
                let mut next = state.clone();
                for &move_to_make in action {
                    next.execute_move(move_to_make);
                }
                if !next.is_yellow_cross_complete() || !visited.insert(next.state_key()) {
                    continue;
                }

                let mut next_algorithms = algorithms.clone();
                next_algorithms.push(action.clone());
                if next.last_layer_edge_count() > completed_edges {
                    return Some(SolveStep::new(
                        SkillId::PermuteYellowEdges,
                        next_algorithms.into_iter().flatten().collect(),
                    ));
                }
                pending.push_back((next, next_algorithms));
            }
        }

        None
    }
}

pub struct LastLayerCornerPositionSolver;

impl LastLayerCornerPositionSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.are_last_layer_edges_complete() || cube.are_last_layer_corners_positioned() {
            return None;
        }

        let positioned_corners = cube.positioned_last_layer_corner_count();
        let actions = last_layer_corner_position_algorithms();
        let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
        let mut visited = HashSet::from([cube.state_key()]);

        while let Some((state, algorithms)) = pending.pop_front() {
            if algorithms.len() == 3 {
                continue;
            }

            for action in &actions {
                let mut next = state.clone();
                for &move_to_make in action {
                    next.execute_move(move_to_make);
                }
                if !next.are_last_layer_edges_complete() || !visited.insert(next.state_key()) {
                    continue;
                }

                let mut next_algorithms = algorithms.clone();
                next_algorithms.push(action.clone());
                if next.positioned_last_layer_corner_count() > positioned_corners {
                    return Some(SolveStep::new(
                        SkillId::PositionYellowCorners,
                        next_algorithms.into_iter().flatten().collect(),
                    ));
                }
                pending.push_back((next, next_algorithms));
            }
        }

        None
    }
}

pub struct LastLayerCornerOrientationSolver;

impl LastLayerCornerOrientationSolver {
    pub fn next_step(cube: &Cube) -> Option<SolveStep> {
        if !cube.are_last_layer_corners_positioned() || cube.is_solved() {
            return None;
        }

        for trigger in last_layer_corner_orientation_actions() {
            for layer_turn in [Move::D, Move::DPrime] {
                if let Some(solution) =
                    search_corner_twists(cube, &trigger, layer_turn, 4, &mut Vec::new())
                {
                    return Some(SolveStep::new(SkillId::OrientYellowCorners, solution));
                }
            }
        }

        None
    }
}

fn search_corner_twists(
    cube: &Cube,
    trigger: &[Move],
    layer_turn: Move,
    corners_remaining: usize,
    moves: &mut Vec<Move>,
) -> Option<Vec<Move>> {
    if corners_remaining == 0 {
        return cube.is_solved().then(|| moves.clone());
    }

    let mut twisted = cube.clone();
    let initial_length = moves.len();
    for repetitions in 0..6 {
        if repetitions > 0 {
            for &move_to_make in trigger {
                twisted.execute_move(move_to_make);
                moves.push(move_to_make);
            }
        }

        let mut next = twisted.clone();
        next.execute_move(layer_turn);
        moves.push(layer_turn);
        if let Some(solution) =
            search_corner_twists(&next, trigger, layer_turn, corners_remaining - 1, moves)
        {
            return Some(solution);
        }
        moves.pop();
    }
    moves.truncate(initial_length);

    None
}

fn last_layer_corner_orientation_actions() -> Vec<Vec<Move>> {
    let mut actions = Vec::new();

    for side in [Move::R, Move::F, Move::L, Move::B] {
        for side in [side, side.inverse()] {
            for up in [Move::U, Move::UPrime] {
                actions.push(vec![side, up, side.inverse(), up.inverse()]);
            }
        }
    }

    actions
}

fn last_layer_corner_position_algorithms() -> Vec<Vec<Move>> {
    let opposite_faces = [(Move::R, Move::L), (Move::F, Move::B)];
    let mut algorithms = Vec::new();

    for (first, second) in opposite_faces {
        for (first, second) in [(first, second), (second, first)] {
            for first in [first, first.inverse()] {
                for second in [second, second.inverse()] {
                    for down in [Move::D, Move::DPrime] {
                        algorithms.push(vec![
                            down,
                            first,
                            down.inverse(),
                            second.inverse(),
                            down,
                            first.inverse(),
                            down.inverse(),
                            second,
                        ]);
                    }
                }
            }
        }
    }

    algorithms
}

fn last_layer_edge_algorithms() -> Vec<Vec<Move>> {
    let mut algorithms = Vec::new();

    for side in [Move::R, Move::F, Move::L, Move::B] {
        for side in [side, side.inverse()] {
            for down in [Move::D, Move::DPrime] {
                algorithms.push(vec![
                    side,
                    down,
                    side.inverse(),
                    down,
                    side,
                    Move::D2,
                    side.inverse(),
                    down,
                ]);
            }
        }
    }

    algorithms
}

fn yellow_cross_algorithms() -> Vec<Vec<Move>> {
    let adjacent_faces = [
        (Move::R, Move::F),
        (Move::F, Move::L),
        (Move::L, Move::B),
        (Move::B, Move::R),
    ];
    let mut algorithms = Vec::new();

    for (first, second) in adjacent_faces {
        for (first, second) in [(first, second), (second, first)] {
            for first in [first, first.inverse()] {
                for second in [second, second.inverse()] {
                    for down in [Move::D, Move::DPrime] {
                        algorithms.push(vec![
                            first,
                            second,
                            down,
                            second.inverse(),
                            down.inverse(),
                            first.inverse(),
                        ]);
                    }
                }
            }
        }
    }

    algorithms
}

fn second_layer_insertions() -> Vec<Vec<Move>> {
    let adjacent_faces = [
        (Move::R, Move::F),
        (Move::F, Move::L),
        (Move::L, Move::B),
        (Move::B, Move::R),
    ];
    let mut algorithms = Vec::new();

    for (first, second) in adjacent_faces {
        for (first, second) in [(first, second), (second, first)] {
            for first in [first, first.inverse()] {
                for second in [second, second.inverse()] {
                    for down in [Move::D, Move::DPrime] {
                        algorithms.push(vec![
                            down,
                            first,
                            down.inverse(),
                            first.inverse(),
                            down.inverse(),
                            second.inverse(),
                            down,
                            second,
                        ]);
                    }
                }
            }
        }
    }

    algorithms
}

fn find_corner_extraction(
    cube: &Cube,
    target_corner: usize,
    completed_corners: usize,
) -> Option<Vec<Move>> {
    let mut pending = VecDeque::from([(cube.clone(), Vec::new())]);
    let mut visited = HashSet::from([cube.state_key()]);

    while let Some((state, algorithm)) = pending.pop_front() {
        if algorithm.len() == MAX_CORNER_STEP_LENGTH {
            continue;
        }

        for move_to_make in MOVES {
            if move_to_make.same_face(Move::U)
                || algorithm
                    .last()
                    .is_some_and(|previous: &Move| previous.same_face(move_to_make))
            {
                continue;
            }

            let mut next = state.clone();
            next.execute_move(move_to_make);
            if !visited.insert(next.state_key()) {
                continue;
            }

            let mut next_algorithm = algorithm.clone();
            next_algorithm.push(move_to_make);
            if next.piece_layer(target_corner) == 0
                && next.is_white_cross_complete()
                && next.first_layer_corner_count() >= completed_corners
            {
                return Some(next_algorithm);
            }
            pending.push_back((next, next_algorithm));
        }
    }

    None
}

fn move_for_side(direction: Direction) -> Move {
    match direction {
        Direction::Left => Move::L,
        Direction::Right => Move::R,
        Direction::Front => Move::F,
        Direction::Back => Move::B,
        Direction::Up | Direction::Down => unreachable!("a corner side cannot be up or down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_daisy(cube: &mut Cube) {
        while !cube.is_daisy_complete() {
            let step = DaisySolver::next_step(cube).expect("a daisy step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_cross(cube: &mut Cube) {
        build_daisy(cube);
        while !cube.is_white_cross_complete() {
            let step =
                DaisyToCrossSolver::next_step(cube).expect("a daisy-to-cross step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_direct_cross(cube: &mut Cube) {
        let mut attempts = 0;
        while !cube.is_white_cross_complete() {
            attempts += 1;
            assert!(attempts <= 4, "direct cross solver stopped making progress");
            let step =
                DirectCrossSolver::next_step(cube).expect("a direct-cross step should exist");
            assert_eq!(step.skill, SkillId::DirectCross);
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_intuitive_f2l(cube: &mut Cube) {
        let mut attempts = 0;
        while !cube.is_second_layer_complete() {
            attempts += 1;
            assert!(attempts < 30, "intuitive F2L stopped making progress");
            let step =
                IntuitiveF2LSolver::next_step(cube).expect("an intuitive-F2L step should exist");
            assert_eq!(step.skill, SkillId::IntuitiveF2L);
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_first_layer(cube: &mut Cube) {
        build_cross(cube);
        let mut attempts = 0;
        while !cube.is_first_layer_complete() {
            attempts += 1;
            assert!(attempts < 50, "corner solver stopped making progress");
            let step = FirstLayerCornerSolver::next_step(cube)
                .expect("a first-layer corner step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_second_layer(cube: &mut Cube) {
        build_first_layer(cube);
        let mut attempts = 0;
        while !cube.is_second_layer_complete() {
            attempts += 1;
            assert!(attempts < 20, "second-layer solver stopped making progress");
            let step = SecondLayerEdgeSolver::next_step(cube)
                .expect("a second-layer edge step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_yellow_cross(cube: &mut Cube) {
        build_second_layer(cube);
        let mut attempts = 0;
        while !cube.is_yellow_cross_complete() {
            attempts += 1;
            assert!(attempts < 5, "yellow-cross solver stopped making progress");
            let step =
                YellowCrossSolver::next_step(cube).expect("a yellow-cross step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn complete_last_layer_edges(cube: &mut Cube) {
        build_yellow_cross(cube);
        let mut attempts = 0;
        while !cube.are_last_layer_edges_complete() {
            attempts += 1;
            assert!(
                attempts < 5,
                "last-layer edge solver stopped making progress"
            );
            let step =
                LastLayerEdgeSolver::next_step(cube).expect("a last-layer edge step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn position_last_layer_corners(cube: &mut Cube) {
        complete_last_layer_edges(cube);
        let mut attempts = 0;
        while !cube.are_last_layer_corners_positioned() {
            attempts += 1;
            assert!(
                attempts < 5,
                "corner-position solver stopped making progress"
            );
            let step = LastLayerCornerPositionSolver::next_step(cube)
                .expect("a corner-position step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn orient_last_layer_corners(cube: &mut Cube) {
        position_last_layer_corners(cube);
        if !cube.is_solved() {
            let step = LastLayerCornerOrientationSolver::next_step(cube)
                .expect("a corner-orientation step should exist");
            for move_to_make in step.moves {
                cube.execute_move(move_to_make);
            }
        }
    }

    #[test]
    fn solved_cube_can_build_a_daisy() {
        let mut cube = Cube::new();
        build_daisy(&mut cube);
        assert!(cube.is_daisy_complete());
    }

    #[test]
    fn solve_steps_identify_the_skill_that_produced_them() {
        let step = DaisySolver::next_step(&Cube::new()).expect("a solved cube needs a daisy step");

        assert_eq!(step.skill, SkillId::BuildDaisy);
        assert_eq!(step.name, "Build daisy");
        assert!(!step.moves.is_empty());
    }

    #[test]
    fn scrambled_cubes_can_build_a_daisy() {
        let scrambles = [
            vec![Move::R, Move::U, Move::FPrime, Move::L, Move::D],
            vec![Move::F, Move::R, Move::UPrime, Move::B, Move::LPrime],
            vec![Move::DPrime, Move::BPrime, Move::R, Move::F, Move::U],
            vec![Move::L, Move::FPrime, Move::D, Move::RPrime, Move::B],
        ];

        for scramble in scrambles {
            let mut cube = Cube::new();
            for move_to_make in scramble {
                cube.execute_move(move_to_make);
            }
            build_daisy(&mut cube);
            assert!(cube.is_daisy_complete());
        }
    }

    #[test]
    fn scrambled_cubes_can_build_a_cross_directly() {
        let scrambles = [
            vec![Move::R, Move::U, Move::FPrime, Move::L, Move::D],
            vec![Move::F, Move::R, Move::UPrime, Move::B, Move::LPrime],
            vec![Move::DPrime, Move::BPrime, Move::R, Move::F, Move::U],
            vec![Move::L, Move::FPrime, Move::D, Move::RPrime, Move::B],
        ];

        for scramble in scrambles {
            let mut cube = Cube::new();
            for move_to_make in scramble {
                cube.execute_move(move_to_make);
            }
            build_direct_cross(&mut cube);
            assert!(cube.is_white_cross_complete());
        }
    }

    #[test]
    fn scrambled_cubes_can_complete_f2l_intuitively() {
        let scrambles = [
            vec![Move::R, Move::U, Move::FPrime, Move::L, Move::D],
            vec![Move::F, Move::R, Move::UPrime, Move::B, Move::LPrime],
            vec![Move::DPrime, Move::BPrime, Move::R, Move::F, Move::U],
            vec![Move::L, Move::FPrime, Move::D, Move::RPrime, Move::B],
        ];

        for scramble in scrambles {
            let mut cube = Cube::new();
            for move_to_make in scramble {
                cube.execute_move(move_to_make);
            }
            build_direct_cross(&mut cube);
            build_intuitive_f2l(&mut cube);
            assert!(cube.is_second_layer_complete());
        }
    }

    #[test]
    fn intuitive_f2l_recognizes_a_one_trigger_insertion() {
        let mut cube = Cube::new();
        for move_to_make in [Move::R, Move::D, Move::RPrime] {
            cube.execute_move(move_to_make);
        }
        assert!(cube.is_white_cross_complete());
        assert_eq!(cube.f2l_pair_count(), 3);

        let step = IntuitiveF2LSolver::next_step(&cube).expect("the open F2L slot is recognized");
        assert!(step.moves.len() <= 3);
        for move_to_make in step.moves {
            cube.execute_move(move_to_make);
        }
        assert_eq!(cube.f2l_pair_count(), 4);
    }

    #[test]
    fn intuitive_f2l_does_not_regress_move_count_on_representative_scrambles() {
        let scrambles = [
            vec![Move::R, Move::U, Move::FPrime, Move::L, Move::D],
            vec![Move::F, Move::R, Move::UPrime, Move::B, Move::LPrime],
            vec![Move::DPrime, Move::BPrime, Move::R, Move::F, Move::U],
            vec![Move::L, Move::FPrime, Move::D, Move::RPrime, Move::B],
        ];

        for scramble in scrambles {
            let mut cross = Cube::new();
            for move_to_make in scramble {
                cross.execute_move(move_to_make);
            }
            build_direct_cross(&mut cross);

            let mut beginner = cross.clone();
            let mut beginner_moves = 0;
            while !beginner.is_first_layer_complete() {
                let step = FirstLayerCornerSolver::next_step(&beginner).unwrap();
                beginner_moves += step.moves.len();
                for move_to_make in step.moves {
                    beginner.execute_move(move_to_make);
                }
            }
            while !beginner.is_second_layer_complete() {
                let step = SecondLayerEdgeSolver::next_step(&beginner).unwrap();
                beginner_moves += step.moves.len();
                for move_to_make in step.moves {
                    beginner.execute_move(move_to_make);
                }
            }

            let mut intuitive = cross;
            let mut intuitive_moves = 0;
            while !intuitive.is_second_layer_complete() {
                let step = IntuitiveF2LSolver::next_step(&intuitive).unwrap();
                intuitive_moves += step.moves.len();
                for move_to_make in step.moves {
                    intuitive.execute_move(move_to_make);
                }
            }

            assert!(
                intuitive_moves <= beginner_moves,
                "intuitive F2L used {intuitive_moves} moves versus {beginner_moves} beginner moves"
            );
        }
    }

    #[test]
    fn many_deterministic_scrambles_can_build_a_daisy() {
        let mut random_state = 0x1234_5678_u32;

        for _ in 0..100 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            build_daisy(&mut cube);
        }
    }

    #[test]
    fn daisy_can_be_converted_to_a_cross() {
        let mut cube = Cube::new();
        for move_to_make in [Move::R, Move::U2, Move::FPrime, Move::L, Move::D2] {
            cube.execute_move(move_to_make);
        }

        build_cross(&mut cube);
        assert!(cube.is_white_cross_complete());
    }

    #[test]
    fn many_deterministic_scrambles_can_build_a_cross() {
        let mut random_state = 0x8765_4321_u32;

        for _ in 0..100 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            build_cross(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_complete_the_first_layer() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        build_first_layer(&mut cube);
        assert!(cube.is_first_layer_complete());
    }

    #[test]
    fn deterministic_scrambles_can_complete_the_first_layer() {
        let mut random_state = 0xcafe_babe_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            build_first_layer(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_complete_the_second_layer() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        build_second_layer(&mut cube);
        assert!(cube.is_second_layer_complete());
    }

    #[test]
    fn deterministic_scrambles_can_complete_the_second_layer() {
        let mut random_state = 0x0ddc_0ffe_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            build_second_layer(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_build_the_yellow_cross() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        build_yellow_cross(&mut cube);
        assert!(cube.is_yellow_cross_complete());
    }

    #[test]
    fn deterministic_scrambles_can_build_the_yellow_cross() {
        let mut random_state = 0xface_feed_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            build_yellow_cross(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_complete_last_layer_edges() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        complete_last_layer_edges(&mut cube);
        assert!(cube.are_last_layer_edges_complete());
    }

    #[test]
    fn deterministic_scrambles_can_complete_last_layer_edges() {
        let mut random_state = 0x1a57_ed6e_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            complete_last_layer_edges(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_position_last_layer_corners() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        position_last_layer_corners(&mut cube);
        assert!(cube.are_last_layer_corners_positioned());
    }

    #[test]
    fn deterministic_scrambles_can_position_last_layer_corners() {
        let mut random_state = 0xc012_3e25_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            position_last_layer_corners(&mut cube);
        }
    }

    #[test]
    fn scrambled_cube_can_be_solved() {
        let mut cube = Cube::new();
        for move_to_make in [
            Move::R,
            Move::U2,
            Move::FPrime,
            Move::L,
            Move::D2,
            Move::BPrime,
        ] {
            cube.execute_move(move_to_make);
        }

        orient_last_layer_corners(&mut cube);
        assert!(cube.is_solved());
    }

    #[test]
    fn deterministic_scrambles_can_be_solved() {
        let mut random_state = 0x501d_c0de_u32;

        for _ in 0..20 {
            let mut cube = Cube::new();
            for _ in 0..20 {
                random_state = random_state
                    .wrapping_mul(1_664_525)
                    .wrapping_add(1_013_904_223);
                cube.execute_move(MOVES[random_state as usize % MOVES.len()]);
            }
            orient_last_layer_corners(&mut cube);
        }
    }
}
