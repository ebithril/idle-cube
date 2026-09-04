use crate::cube::Cube;
use crate::moves::random_scramble;
use crate::solver::{DaisySolver, DaisyToCrossSolver, FirstLayerCornerSolver};

use macroquad::prelude::*;

// Fragment shader - will be loaded from file
const CUBE_FRAGMENT_SHADER: &str = include_str!("../cube_shader.frag");

// Vertex shader that works with macroquad's coordinate system
const VERTEX_SHADER: &str = include_str!("../cube_shader.vert");

pub struct Game {
    cube: Cube,
    material: Material,
    scramble_complete: bool,
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
        self.cube.update(dt);

        if !self.scramble_complete && self.cube.is_idle() {
            self.scramble_complete = true;
        }

        if self.scramble_complete && self.cube.is_idle() && !self.cube.is_first_layer_complete() {
            if let Some(step) = FirstLayerCornerSolver::next_step(&self.cube) {
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
        draw_fps();

        next_frame().await
    }
}
