use crate::cube::Cube;
use crate::moves::random_scramble;
use crate::progression::{PlayerSkills, Technique};
use crate::solver::{
    DaisySolver, DaisyToCrossSolver, DirectCrossSolver, FirstLayerCornerSolver, IntuitiveF2LSolver,
    LastLayerCornerOrientationSolver, LastLayerCornerPositionSolver, LastLayerEdgeSolver,
    SecondLayerEdgeSolver, SolveStep, YellowCrossSolver,
};

use macroquad::prelude::*;
use std::collections::VecDeque;

// Fragment shader - will be loaded from file
const CUBE_FRAGMENT_SHADER: &str = include_str!("../cube_shader.frag");

// Vertex shader that works with macroquad's coordinate system
const VERTEX_SHADER: &str = include_str!("../cube_shader.vert");

fn direct_cross_button() -> Rect {
    Rect::new(screen_width() - 310.0, 80.0, 290.0, 70.0)
}

fn intuitive_f2l_button() -> Rect {
    Rect::new(screen_width() - 310.0, 170.0, 290.0, 70.0)
}

#[derive(Clone, Copy, Debug)]
struct SolveResult {
    time: f32,
    moves: u32,
    recognition_time: f32,
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
    recognition_time: f32,
    last: Option<SolveResult>,
    best: Option<SolveResult>,
    history: VecDeque<SolveResult>,
}

impl SolveTiming {
    fn start(&mut self) {
        self.elapsed = Some(0.0);
        self.moves = 0;
        self.recognition_time = 0.0;
    }

    fn update(&mut self, dt: f32, completed_move: bool, recognizing: bool) {
        if let Some(elapsed) = &mut self.elapsed {
            *elapsed += dt;
            if completed_move {
                self.moves += 1;
            }
            if recognizing {
                self.recognition_time += dt;
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
            recognition_time: self.recognition_time,
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
    pending_step: Option<SolveStep>,
    recognition_remaining: f32,
    active_skill: Option<crate::solver::SkillId>,
    player_skills: PlayerSkills,
}

impl Game {
    pub fn new() -> Self {
        let player_skills = PlayerSkills::default();
        let mut cube = Cube::new();
        cube.set_tps(player_skills.max_tps());

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
            pending_step: None,
            recognition_remaining: 0.0,
            active_skill: None,
            player_skills,
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
        self.handle_skill_tree();
        let was_solving = self.scramble_complete && self.solve_timing.elapsed.is_some();
        let was_recognizing = self.pending_step.is_some();
        let completed_move = self.cube.update(dt);

        if completed_move.is_some() && self.cube.is_idle() {
            if let Some(skill) = self.active_skill.take() {
                self.player_skills.practice_turning(skill);
            }
        }

        if was_solving {
            self.solve_timing
                .update(dt, completed_move.is_some(), was_recognizing);
            if self.cube.is_idle() && self.cube.is_solved() {
                self.solve_timing.finish();
                self.player_skills.award_solve_xp();
                self.queue_scramble();
                return;
            }
        }

        if !self.scramble_complete && self.cube.is_idle() {
            self.scramble_complete = true;
            self.solve_timing.start();
        }

        if self.pending_step.is_some() {
            self.recognition_remaining -= dt;
            if self.recognition_remaining <= 0.0 {
                let step = self.pending_step.take().expect("the step is pending");
                self.player_skills.practice_recognition(step.skill);
                self.cube
                    .set_tps(self.player_skills.turning_tps(step.skill));
                self.active_skill = Some(step.skill);
                self.cube.queue_moves(step.moves);
            }
        } else if self.scramble_complete && self.cube.is_idle() && !self.cube.is_solved() {
            if let Some(step) = self.select_next_step() {
                self.recognition_remaining = self.player_skills.recognition_time(step.skill);
                self.pending_step = Some(step);
            }
        }
    }

    fn select_next_step(&self) -> Option<SolveStep> {
        LastLayerCornerOrientationSolver::next_step(&self.cube)
            .or_else(|| LastLayerCornerPositionSolver::next_step(&self.cube))
            .or_else(|| LastLayerEdgeSolver::next_step(&self.cube))
            .or_else(|| YellowCrossSolver::next_step(&self.cube))
            .or_else(|| {
                if self.player_skills.is_unlocked(Technique::IntuitiveF2L) {
                    IntuitiveF2LSolver::next_step(&self.cube)
                } else {
                    SecondLayerEdgeSolver::next_step(&self.cube)
                        .or_else(|| FirstLayerCornerSolver::next_step(&self.cube))
                }
            })
            .or_else(|| {
                if self.player_skills.is_unlocked(Technique::DirectCross) {
                    DirectCrossSolver::next_step(&self.cube)
                } else {
                    DaisyToCrossSolver::next_step(&self.cube)
                        .or_else(|| DaisySolver::next_step(&self.cube))
                }
            })
    }

    fn queue_scramble(&mut self) {
        let mut rng = ::rand::rng();
        self.cube.set_tps(self.player_skills.max_tps());
        self.cube.queue_moves(random_scramble(&mut rng, 20));
        self.scramble_complete = false;
        self.pending_step = None;
        self.recognition_remaining = 0.0;
        self.active_skill = None;
    }

    fn handle_skill_tree(&mut self) {
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = Vec2::from(mouse_position());
            if direct_cross_button().contains(mouse) {
                self.player_skills.unlock(Technique::DirectCross);
            } else if intuitive_f2l_button().contains(mouse) {
                self.player_skills.unlock(Technique::IntuitiveF2L);
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
        if let Some(elapsed) = self.solve_timing.elapsed {
            draw_text(
                &format!(
                    "TIME: {elapsed:.2}s  MOVES: {}  RECOGNITION: {:.2}s",
                    self.solve_timing.moves, self.solve_timing.recognition_time
                ),
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
                &format!(
                    "LAST: {:.2}s / {} moves / {:.2}s recognition",
                    last.time, last.moves, last.recognition_time
                ),
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

        let direct_cross = direct_cross_button();
        let cross_unlocked = self.player_skills.is_unlocked(Technique::DirectCross);
        let can_afford = self.player_skills.skill_points() >= Technique::DirectCross.cost();
        draw_text(
            &format!(
                "SKILL POINTS: {}    MAX TPS: {:.2}",
                self.player_skills.skill_points(),
                self.player_skills.max_tps()
            ),
            direct_cross.x,
            50.0,
            24.0,
            WHITE,
        );
        draw_rectangle(
            direct_cross.x,
            direct_cross.y,
            direct_cross.w,
            direct_cross.h,
            if cross_unlocked {
                DARKGREEN
            } else if can_afford {
                BLUE
            } else {
                DARKGRAY
            },
        );
        draw_text(
            "DIRECT CROSS",
            direct_cross.x + 10.0,
            direct_cross.y + 28.0,
            24.0,
            WHITE,
        );
        draw_text(
            if cross_unlocked {
                "UNLOCKED - skips the daisy"
            } else {
                "1 SP - skips the daisy"
            },
            direct_cross.x + 10.0,
            direct_cross.y + 56.0,
            24.0,
            WHITE,
        );

        let intuitive_f2l = intuitive_f2l_button();
        let f2l_unlocked = self.player_skills.is_unlocked(Technique::IntuitiveF2L);
        let f2l_available =
            cross_unlocked && self.player_skills.skill_points() >= Technique::IntuitiveF2L.cost();
        draw_rectangle(
            intuitive_f2l.x,
            intuitive_f2l.y,
            intuitive_f2l.w,
            intuitive_f2l.h,
            if f2l_unlocked {
                DARKGREEN
            } else if f2l_available {
                BLUE
            } else {
                DARKGRAY
            },
        );
        draw_text(
            "INTUITIVE F2L",
            intuitive_f2l.x + 10.0,
            intuitive_f2l.y + 28.0,
            24.0,
            WHITE,
        );
        draw_text(
            if f2l_unlocked {
                "UNLOCKED - pairs corners and edges"
            } else if cross_unlocked {
                "1 SP - pairs corners and edges"
            } else {
                "REQUIRES DIRECT CROSS"
            },
            intuitive_f2l.x + 10.0,
            intuitive_f2l.y + 56.0,
            19.0,
            LIGHTGRAY,
        );

        let xp_bar = Rect::new(20.0, screen_height() - 38.0, screen_width() - 40.0, 22.0);
        draw_rectangle(xp_bar.x, xp_bar.y, xp_bar.w, xp_bar.h, DARKGRAY);
        draw_rectangle(
            xp_bar.x,
            xp_bar.y,
            xp_bar.w * self.player_skills.xp_progress(),
            xp_bar.h,
            BLUE,
        );
        draw_rectangle_lines(xp_bar.x, xp_bar.y, xp_bar.w, xp_bar.h, 2.0, WHITE);
        let level_text = format!(
            "LEVEL {}    XP {:.0} / {:.0}",
            self.player_skills.level(),
            self.player_skills.xp(),
            self.player_skills.xp_to_next_level()
        );
        let text_width = measure_text(&level_text, None, 20, 1.0).width;
        draw_text(
            &level_text,
            xp_bar.x + (xp_bar.w - text_width) / 2.0,
            xp_bar.y + 18.0,
            20.0,
            WHITE,
        );
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
        timing.update(0.5, true, true);
        timing.update(0.25, false, false);
        timing.finish();
        assert_eq!(timing.last.unwrap().moves, 1);
        assert_eq!(timing.last.unwrap().time, 0.75);
        assert_eq!(timing.last.unwrap().recognition_time, 0.5);
        assert_eq!(timing.best.unwrap().time, 0.75);

        timing.start();
        timing.update(1.0, true, false);
        timing.finish();
        assert_eq!(timing.last.unwrap().time, 1.0);
        assert_eq!(timing.best.unwrap().time, 0.75);
    }

    #[test]
    fn solve_timing_calculates_rolling_averages() {
        let mut timing = SolveTiming::default();

        for seconds in 1..=5 {
            timing.start();
            timing.update(seconds as f32, true, false);
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
            timing.update(seconds as f32, false, false);
            timing.finish();
        }

        assert_eq!(timing.history.len(), 100);
        assert_eq!(timing.average(100).unwrap().time, 51.5);
    }
}
