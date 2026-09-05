use crate::cube::Cube;
use crate::moves::random_scramble;
use crate::solver::{
    DaisySolver, DaisyToCrossSolver, FirstLayerCornerSolver, LastLayerCornerOrientationSolver,
    LastLayerCornerPositionSolver, LastLayerEdgeSolver, SecondLayerEdgeSolver, YellowCrossSolver,
};

use macroquad::prelude::*;
use std::collections::VecDeque;

// Fragment shader - will be loaded from file
const CUBE_FRAGMENT_SHADER: &str = include_str!("../cube_shader.frag");

// Vertex shader that works with macroquad's coordinate system
const VERTEX_SHADER: &str = include_str!("../cube_shader.vert");

#[derive(Clone, Copy, Debug)]
struct SolveResult {
    time: f32,
    moves: u32,
}

#[derive(Clone, Copy, Debug)]
struct SolveAverage {
    time: f32,
    moves: f32,
}

#[derive(Default)]
struct SolveTiming {
    elapsed: Option<f32>,
    moves: u32,
    last: Option<SolveResult>,
    best: Option<SolveResult>,
    history: VecDeque<SolveResult>,
}

impl SolveTiming {
    fn start(&mut self) {
        self.elapsed = Some(0.0);
        self.moves = 0;
    }

    fn update(&mut self, dt: f32, completed_move: bool) {
        if let Some(elapsed) = &mut self.elapsed {
            *elapsed += dt;
            if completed_move {
                self.moves += 1;
            }
        }
    }

    fn finish(&mut self) {
        let Some(time) = self.elapsed.take() else {
            return;
        };
        let result = SolveResult {
            time,
            moves: self.moves,
        };
        self.last = Some(result);
        if self.best.is_none_or(|best| result.time < best.time) {
            self.best = Some(result);
        }
        self.history.push_back(result);
        if self.history.len() > 100 {
            self.history.pop_front();
        }
    }

    fn average(&self, solve_count: usize) -> Option<SolveAverage> {
        if self.history.len() < solve_count {
            return None;
        }

        let recent = self.history.iter().rev().take(solve_count);
        let (total_time, total_moves) = recent.fold((0.0, 0_u32), |totals, solve| {
            (totals.0 + solve.time, totals.1 + solve.moves)
        });
        Some(SolveAverage {
            time: total_time / solve_count as f32,
            moves: total_moves as f32 / solve_count as f32,
        })
    }
}

pub struct Game {
    cube: Cube,
    material: Material,
    scramble_complete: bool,
    solve_timing: SolveTiming,
}

impl Game {
    pub fn new() -> Self {
        let mut cube = Cube::new();

        let mut rng = ::rand::rng();
        cube.queue_moves(random_scramble(&mut rng, 20));

        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: CUBE_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::array(UniformDesc::new("XRotation", UniformType::Mat4), 3),
                    UniformDesc::array(UniformDesc::new("YRotation", UniformType::Mat4), 3),
                    UniformDesc::array(UniformDesc::new("ZRotation", UniformType::Mat4), 3),
                    UniformDesc::new("CubeRotation", UniformType::Mat4),
                    UniformDesc::new("CameraPos", UniformType::Float3),
                ],
                pipeline_params: PipelineParams {
                    depth_write: true,
                    depth_test: Comparison::Less,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();

        Game {
            cube,
            material,
            scramble_complete: false,
            solve_timing: SolveTiming::default(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.update();
            self.render().await
        }
    }

    fn update(&mut self) {
        let dt = get_frame_time();
        let was_solving = self.scramble_complete && self.solve_timing.elapsed.is_some();
        let completed_move = self.cube.update(dt);

        if was_solving {
            self.solve_timing.update(dt, completed_move.is_some());
            if self.cube.is_idle() && self.cube.is_solved() {
                self.solve_timing.finish();
                self.queue_scramble();
                return;
            }
        }

        if !self.scramble_complete && self.cube.is_idle() {
            self.scramble_complete = true;
            self.solve_timing.start();
        }

        if self.scramble_complete && self.cube.is_idle() && !self.cube.is_solved() {
            if let Some(step) = LastLayerCornerOrientationSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = LastLayerCornerPositionSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = LastLayerEdgeSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = YellowCrossSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = SecondLayerEdgeSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = FirstLayerCornerSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = DaisyToCrossSolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            } else if let Some(step) = DaisySolver::next_step(&self.cube) {
                self.cube.queue_moves(step);
            }
        }
    }

    fn queue_scramble(&mut self) {
        let mut rng = ::rand::rng();
        self.cube.queue_moves(random_scramble(&mut rng, 20));
        self.scramble_complete = false;
    }

    async fn render(&self) {
        let cube_mesh = self.cube.create_mesh();
        clear_background(Color::new(0.05, 0.05, 0.1, 1.0));
        let camera = Camera3D {
            position: vec3(4.0, 3.0, 6.),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            ..Default::default()
        };

        // Set up camera
        set_camera(&camera);

        let rots = self.cube.get_side_rotation();

        self.material.set_uniform_array("XRotation", &rots[0][..]);
        self.material.set_uniform_array("YRotation", &rots[1][..]);
        self.material.set_uniform_array("ZRotation", &rots[2][..]);

        let cube_rotation = if self.scramble_complete {
            Mat4::from_rotation_x(std::f32::consts::PI)
        } else {
            Mat4::IDENTITY
        };
        self.material.set_uniform("CubeRotation", cube_rotation);

        self.material.set_uniform("CameraPos", camera.position);
        gl_use_material(&self.material);
        draw_mesh(&cube_mesh);
        gl_use_default_material();

        set_default_camera();
        if let Some(elapsed) = self.solve_timing.elapsed {
            draw_text(
                &format!("TIME: {elapsed:.2}s  MOVES: {}", self.solve_timing.moves),
                10.0,
                30.0,
                30.0,
                WHITE,
            );
        } else {
            draw_text("SCRAMBLING", 10.0, 30.0, 30.0, WHITE);
        }
        if let Some(last) = self.solve_timing.last {
            draw_text(
                &format!("LAST: {:.2}s / {} moves", last.time, last.moves),
                10.0,
                60.0,
                30.0,
                GREEN,
            );
        }
        if let Some(best) = self.solve_timing.best {
            draw_text(
                &format!("BEST: {:.2}s / {} moves", best.time, best.moves),
                10.0,
                90.0,
                30.0,
                GREEN,
            );
        }
        for (solve_count, y) in [(3, 120.0), (5, 150.0), (20, 180.0), (100, 210.0)] {
            let text = self.solve_timing.average(solve_count).map_or_else(
                || format!("AO{solve_count}: --"),
                |average| {
                    format!(
                        "AO{solve_count}: {:.2}s / {:.1} moves",
                        average.time, average.moves
                    )
                },
            );
            draw_text(&text, 10.0, y, 30.0, WHITE);
        }
        next_frame().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_timing_tracks_time_moves_and_personal_best() {
        let mut timing = SolveTiming::default();

        timing.start();
        timing.update(0.5, true);
        timing.update(0.25, false);
        timing.finish();
        assert_eq!(timing.last.unwrap().moves, 1);
        assert_eq!(timing.last.unwrap().time, 0.75);
        assert_eq!(timing.best.unwrap().time, 0.75);

        timing.start();
        timing.update(1.0, true);
        timing.finish();
        assert_eq!(timing.last.unwrap().time, 1.0);
        assert_eq!(timing.best.unwrap().time, 0.75);
    }

    #[test]
    fn solve_timing_calculates_rolling_averages() {
        let mut timing = SolveTiming::default();

        for seconds in 1..=5 {
            timing.start();
            timing.update(seconds as f32, true);
            timing.finish();
        }

        assert!(timing.average(20).is_none());
        let average_of_three = timing.average(3).unwrap();
        assert_eq!(average_of_three.time, 4.0);
        assert_eq!(average_of_three.moves, 1.0);
        assert_eq!(timing.average(5).unwrap().time, 3.0);
    }

    #[test]
    fn solve_history_keeps_the_most_recent_hundred_solves() {
        let mut timing = SolveTiming::default();

        for seconds in 1..=101 {
            timing.start();
            timing.update(seconds as f32, false);
            timing.finish();
        }

        assert_eq!(timing.history.len(), 100);
        assert_eq!(timing.average(100).unwrap().time, 51.5);
    }
}
