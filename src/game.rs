use crate::cube::Cube;
use crate::moves::random_scramble;
use crate::solver::{
    DaisySolver, DaisyToCrossSolver, FirstLayerCornerSolver, LastLayerCornerOrientationSolver,
    LastLayerCornerPositionSolver, LastLayerEdgeSolver, SecondLayerEdgeSolver, YellowCrossSolver,
};

use macroquad::prelude::*;

// Fragment shader - will be loaded from file
const CUBE_FRAGMENT_SHADER: &str = include_str!("../cube_shader.frag");

// Vertex shader that works with macroquad's coordinate system
const VERTEX_SHADER: &str = include_str!("../cube_shader.vert");

#[derive(Clone, Copy, Debug)]
struct SolveResult {
    time: f32,
    moves: u32,
}

#[derive(Default)]
struct SolveTiming {
    elapsed: Option<f32>,
    moves: u32,
    last: Option<SolveResult>,
    best: Option<SolveResult>,
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
        if self.cube.is_solved() {
            draw_text("SOLVED", 10.0, 30.0, 30.0, GREEN);
        } else {
            draw_text("UNSOLVED", 10.0, 30.0, 30.0, RED);
        }
        draw_text(
            &format!("DAISY: {}/4", self.cube.daisy_count()),
            10.0,
            60.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("CROSS: {}/4", self.cube.white_cross_count()),
            10.0,
            90.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("CORNERS: {}/4", self.cube.first_layer_corner_count()),
            10.0,
            120.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("SECOND LAYER: {}/4", self.cube.second_layer_edge_count()),
            10.0,
            150.0,
            30.0,
            WHITE,
        );
        draw_text(
            &format!("YELLOW CROSS: {}/4", self.cube.yellow_cross_count()),
            10.0,
            180.0,
            30.0,
            YELLOW,
        );
        draw_text(
            &format!("LAST EDGES: {}/4", self.cube.last_layer_edge_count()),
            10.0,
            210.0,
            30.0,
            YELLOW,
        );
        draw_text(
            &format!(
                "POSITIONED CORNERS: {}/4",
                self.cube.positioned_last_layer_corner_count()
            ),
            10.0,
            240.0,
            30.0,
            YELLOW,
        );
        draw_text(
            &format!(
                "ORIENTED CORNERS: {}/4",
                self.cube.oriented_last_layer_corner_count()
            ),
            10.0,
            270.0,
            30.0,
            YELLOW,
        );
        if let Some(elapsed) = self.solve_timing.elapsed {
            draw_text(
                &format!("TIME: {elapsed:.2}s  MOVES: {}", self.solve_timing.moves),
                10.0,
                310.0,
                30.0,
                WHITE,
            );
        }
        if let Some(last) = self.solve_timing.last {
            draw_text(
                &format!("LAST: {:.2}s / {} moves", last.time, last.moves),
                10.0,
                340.0,
                30.0,
                GREEN,
            );
        }
        if let Some(best) = self.solve_timing.best {
            draw_text(
                &format!("BEST: {:.2}s / {} moves", best.time, best.moves),
                10.0,
                370.0,
                30.0,
                GREEN,
            );
        }
        draw_fps();

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
}
