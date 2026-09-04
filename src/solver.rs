use crate::cube::{Cube, Direction};
use crate::moves::Move;
use std::collections::{HashSet, VecDeque};

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
const MAX_CORNER_STEP_LENGTH: usize = 4;

pub struct DaisySolver;

impl DaisySolver {
    pub fn next_step(cube: &Cube) -> Option<Vec<Move>> {
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
                    return Some(next_algorithm);
                }

                pending.push_back((next, next_algorithm));
            }
        }

        None
    }
}

pub struct DaisyToCrossSolver;

impl DaisyToCrossSolver {
    pub fn next_step(cube: &Cube) -> Option<Vec<Move>> {
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
                return Some(vec![insertion]);
            }
        }

        Some(vec![Move::D])
    }
}

pub struct FirstLayerCornerSolver;

impl FirstLayerCornerSolver {
    pub fn next_step(cube: &Cube) -> Option<Vec<Move>> {
        if !cube.is_white_cross_complete() || cube.is_first_layer_complete() {
            return None;
        }

        let completed_corners = cube.first_layer_corner_count();
        let (target_corner, first_side, second_side) = cube.first_unsolved_layer_corner_sides()?;

        if cube.piece_layer(target_corner) == 2 {
            return find_corner_extraction(cube, target_corner, completed_corners);
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
                    return Some(adjustment.to_vec());
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
                            return Some(repeated_trigger);
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
    pub fn next_step(cube: &Cube) -> Option<Vec<Move>> {
        if !cube.is_first_layer_complete() || cube.is_second_layer_complete() {
            return None;
        }

        let completed_edges = cube.second_layer_edge_count();
        let edges_on_down = cube.second_layer_edges_on_down_count();
        let mut ejection = None;

        for setup in [vec![], vec![Move::D], vec![Move::DPrime], vec![Move::D2]] {
            for insertion in second_layer_insertions() {
                let algorithm: Vec<_> = setup.iter().chain(insertion.iter()).copied().collect();
                let mut attempt = cube.clone();
                for &move_to_make in &algorithm {
                    attempt.execute_move(move_to_make);
                }

                if !attempt.is_first_layer_complete()
                    || attempt.second_layer_edge_count() < completed_edges
                {
                    continue;
                }
                if attempt.second_layer_edge_count() > completed_edges {
                    return Some(algorithm);
                }
                if attempt.second_layer_edges_on_down_count() > edges_on_down && ejection.is_none()
                {
                    ejection = Some(algorithm);
                }
            }
        }

        ejection
    }
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
            for move_to_make in step {
                cube.execute_move(move_to_make);
            }
        }
    }

    fn build_cross(cube: &mut Cube) {
        build_daisy(cube);
        while !cube.is_white_cross_complete() {
            let step =
                DaisyToCrossSolver::next_step(cube).expect("a daisy-to-cross step should exist");
            for move_to_make in step {
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
            for move_to_make in step {
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
            for move_to_make in step {
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
}
