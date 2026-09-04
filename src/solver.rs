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
}
