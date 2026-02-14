use crate::moves::Move;
use macroquad::prelude::*;
use std::collections::VecDeque;
use std::f32::consts::PI;

const UP_NORMAL: Vec3 = vec3(0., 1., 0.);
const RIGHT_NORMAL: Vec3 = vec3(1., 0., 0.);
const FRONT_NORMAL: Vec3 = vec3(0., 0., 1.);

const XMAX: usize = 3;
const YMAX: usize = 3;

#[derive(Clone)]
struct PieceColor {
    front: Color,
    back: Color,
    left: Color,
    right: Color,
    up: Color,
    down: Color,
}

impl Default for PieceColor {
    fn default() -> Self {
        Self {
            front: DARKGRAY,
            back: DARKGRAY,
            left: DARKGRAY,
            right: DARKGRAY,
            up: DARKGRAY,
            down: DARKGRAY,
        }
    }
}

#[derive(Clone)]
struct Piece {
    color: PieceColor,
    rotation: Quat,
}

pub struct Cube {
    pieces: Vec<Piece>,
    alg: VecDeque<Move>,
    move_time: f32,
    alg_time: f32,
}

impl Cube {
    pub fn new() -> Self {
        let piece_colors: [PieceColor; 26] = [
            PieceColor {
                left: ORANGE,
                back: BLUE,
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                right: RED,
                back: BLUE,
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                left: ORANGE,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                right: RED,
                ..Default::default()
            },
            PieceColor {
                left: ORANGE,
                back: BLUE,
                up: WHITE,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                up: WHITE,
                ..Default::default()
            },
            PieceColor {
                back: BLUE,
                right: RED,
                up: WHITE,
                ..Default::default()
            },
            PieceColor {
                left: ORANGE,
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                right: RED,
                down: YELLOW,
                ..Default::default()
            },
            PieceColor {
                left: ORANGE,
                ..Default::default()
            },
            PieceColor {
                right: RED,
                ..Default::default()
            },
            PieceColor {
                left: ORANGE,
                up: WHITE,
                ..Default::default()
            },
            PieceColor {
                up: WHITE,
                ..Default::default()
            },
            PieceColor {
                up: WHITE,
                right: RED,
                ..Default::default()
            },
            PieceColor {
                down: YELLOW,
                front: GREEN,
                left: ORANGE,
                ..Default::default()
            },
            PieceColor {
                down: YELLOW,
                front: GREEN,
                ..Default::default()
            },
            PieceColor {
                down: YELLOW,
                front: GREEN,
                right: RED,
                ..Default::default()
            },
            PieceColor {
                front: GREEN,
                left: ORANGE,
                ..Default::default()
            },
            PieceColor {
                front: GREEN,
                ..Default::default()
            },
            PieceColor {
                front: GREEN,
                right: RED,
                ..Default::default()
            },
            PieceColor {
                up: WHITE,
                front: GREEN,
                left: ORANGE,
                ..Default::default()
            },
            PieceColor {
                up: WHITE,
                front: GREEN,
                ..Default::default()
            },
            PieceColor {
                up: WHITE,
                front: GREEN,
                right: RED,
                ..Default::default()
            },
        ];

        let mut pieces: Vec<Piece> = Vec::new();

        for color in piece_colors {
            pieces.push(Piece {
                color,
                rotation: Quat::IDENTITY,
            });
        }

        Self {
            pieces,
            alg: VecDeque::new(),
            move_time: 1.0,
            alg_time: 0.,
        }
    }

    pub fn queue_move(&mut self, move_to_make: Move) {
        self.alg.push_back(move_to_make);
    }

    pub fn create_mesh(&self) -> Mesh {
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u16>::new();

        for (i, piece) in self.pieces.iter().enumerate() {
            add_piece_verticies(&mut vertices, &mut indices, i, &piece, 1.0);
        }

        Mesh {
            vertices,
            indices,
            texture: None,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.alg.is_empty() {
            return;
        }

        self.alg_time += dt;
        if self.alg_time > self.move_time {
            self.alg_time = 0.;
            let current_move = self.alg.pop_front();
            self.execute_move(current_move.unwrap().clone());
        }
    }

    fn get_rotation_angle(&self, expected_move: Move) -> Mat4 {
        let current_move = match self.alg.front() {
            Some(m) => m.clone(),
            None => return Mat4::IDENTITY,
        };

        if !current_move.contains(expected_move) {
            return Mat4::IDENTITY;
        }

        let sign = current_move.get_sign();

        let rot_angle = (PI / 2.) * self.alg_time / self.move_time;

        Mat4::from_quat(current_move.create_rotation(rot_angle * sign))
    }

    pub fn get_side_rotation(&self) -> [[Mat4; 3]; 3] {
        let r_rotation = self.get_rotation_angle(Move::R);
        let u_rotation = self.get_rotation_angle(Move::U);
        let f_rotation = self.get_rotation_angle(Move::F);
        let l_rotation = self.get_rotation_angle(Move::L);
        let d_rotation = self.get_rotation_angle(Move::D);
        let b_rotation = self.get_rotation_angle(Move::B);

        [
            [l_rotation, Mat4::IDENTITY, r_rotation],
            [d_rotation, Mat4::IDENTITY, u_rotation],
            [b_rotation, Mat4::IDENTITY, f_rotation],
        ]
    }

    fn get_edge_idx_for_move(&self, move_to_make: Move) -> [usize; 4] {
        let edges = if move_to_make.contains(Move::U) {
            [
                get_idx(0, 2, 1),
                get_idx(1, 2, 2),
                get_idx(2, 2, 1),
                get_idx(1, 2, 0),
            ]
        } else if move_to_make.contains(Move::R) {
            [
                get_idx(2, 1, 2),
                get_idx(2, 0, 1),
                get_idx(2, 1, 0),
                get_idx(2, 2, 1),
            ]
        } else if move_to_make.contains(Move::F) {
            [
                get_idx(1, 2, 2),
                get_idx(0, 1, 2),
                get_idx(1, 0, 2),
                get_idx(2, 1, 2),
            ]
        } else if move_to_make.contains(Move::D) {
            [
                get_idx(1, 0, 2),
                get_idx(0, 0, 1),
                get_idx(1, 0, 0),
                get_idx(2, 0, 1),
            ]
        } else if move_to_make.contains(Move::L) {
            [
                get_idx(0, 0, 1),
                get_idx(0, 1, 2),
                get_idx(0, 2, 1),
                get_idx(0, 1, 0),
            ]
        } else if move_to_make.contains(Move::B) {
            [
                get_idx(0, 1, 0),
                get_idx(1, 2, 0),
                get_idx(2, 1, 0),
                get_idx(1, 0, 0),
            ]
        } else {
            panic!("This shouldn't be possible")
        };

        if move_to_make.is_prime() {
            let mut r_edges: [usize; 4] = [0, 0, 0, 0];
            let mut i = 0;
            for e in edges.iter().rev() {
                r_edges[i] = e.clone();
                i += 1;
            }

            r_edges
        } else {
            edges
        }
    }

    fn get_corner_idx_for_move(&self, move_to_make: Move) -> [usize; 4] {
        let corners = if move_to_make.contains(Move::U) {
            [
                get_idx(0, 2, 0),
                get_idx(0, 2, 2),
                get_idx(2, 2, 2),
                get_idx(2, 2, 0),
            ]
        } else if move_to_make.contains(Move::R) {
            [
                get_idx(2, 0, 2),
                get_idx(2, 0, 0),
                get_idx(2, 2, 0),
                get_idx(2, 2, 2),
            ]
        } else if move_to_make.contains(Move::F) {
            [
                get_idx(0, 2, 2),
                get_idx(0, 0, 2),
                get_idx(2, 0, 2),
                get_idx(2, 2, 2),
            ]
        } else if move_to_make.contains(Move::D) {
            [
                get_idx(0, 0, 2),
                get_idx(0, 0, 0),
                get_idx(2, 0, 0),
                get_idx(2, 0, 2),
            ]
        } else if move_to_make.contains(Move::L) {
            [
                get_idx(0, 0, 0),
                get_idx(0, 0, 2),
                get_idx(0, 2, 2),
                get_idx(0, 2, 0),
            ]
        } else if move_to_make.contains(Move::B) {
            [
                get_idx(0, 0, 0),
                get_idx(0, 2, 0),
                get_idx(2, 2, 0),
                get_idx(2, 0, 0),
            ]
        } else {
            panic!("This shouldn't be possible")
        };

        if move_to_make.is_prime() {
            let mut r_corners: [usize; 4] = [0, 0, 0, 0];
            let mut i = 0;
            for e in corners.iter().rev() {
                r_corners[i] = e.clone();
                i += 1;
            }

            r_corners
        } else {
            corners
        }
    }

    pub fn execute_move(&mut self, move_to_make: Move) {
        let edges = self.get_edge_idx_for_move(move_to_make.clone());
        let corners = self.get_corner_idx_for_move(move_to_make.clone());

        let sign = move_to_make.get_sign();
        let rotation = move_to_make.create_rotation((PI / 2.) * sign);

        let mut buffer = self.pieces[edges[0]].clone();
        buffer.rotation = rotation * buffer.rotation;
        for i in 0..edges.len() - 1 {
            self.pieces[edges[i]] = self.pieces[edges[i + 1]].clone();
            self.pieces[edges[i]].rotation = rotation * self.pieces[edges[i]].rotation;
        }
        self.pieces[edges[3]] = buffer;

        let mut buffer = self.pieces[corners[0]].clone();
        buffer.rotation = rotation * buffer.rotation;
        for i in 0..corners.len() - 1 {
            self.pieces[corners[i]] = self.pieces[corners[i + 1]].clone();
            self.pieces[corners[i]].rotation = rotation * self.pieces[corners[i]].rotation;
        }
        self.pieces[corners[3]] = buffer;
    }

    pub fn is_solved(&self) -> bool {
        for piece in &self.pieces {
            if !piece.rotation.is_near_identity() {
                return false;
            }
        }

        true
    }
}

fn get_idx(x: usize, y: usize, z: usize) -> usize {
    let idx = (z * XMAX * YMAX) + (y * XMAX) + x;

    if idx < 13 {
        return idx;
    }

    idx - 1
}

fn get_3d_index(piece_index: usize) -> (usize, usize, usize) {
    let mut idx = if piece_index < 13 {
        piece_index
    } else {
        piece_index + 1
    };

    let z = idx / (XMAX * YMAX);
    idx -= z * XMAX * YMAX;
    let y = idx / XMAX;
    let x = idx % XMAX;
    (x, y, z)
}

fn get_piece_position(piece_index: usize, size: f32) -> Vec3 {
    let piece_size = size / 3.0;
    let piece_dist = size / 75.;
    let half_size = size / 2.0 + piece_size / 2. + piece_dist / 2.;
    let displacement = (size - piece_size) + piece_dist;
    let (x, y, z) = get_3d_index(piece_index);

    vec3(
        displacement * x as f32 - half_size,
        displacement * y as f32 - half_size,
        displacement * z as f32 - half_size,
    )
}

fn add_piece_verticies(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    piece_index: usize,
    piece: &Piece,
    size: f32,
) {
    let piece_center = get_piece_position(piece_index, size);
    let piece_color = &piece.color;
    let index_offset = vertices.len() as u16;
    let piece_size = size / 3.0;
    let float_index = piece_index as f32 + 0.5;

    let front_normal = piece.rotation.mul_vec3(FRONT_NORMAL);
    let back_normal = piece.rotation.mul_vec3(-FRONT_NORMAL);
    let right_normal = piece.rotation.mul_vec3(RIGHT_NORMAL);
    let left_normal = piece.rotation.mul_vec3(-RIGHT_NORMAL);
    let up_normal = piece.rotation.mul_vec3(UP_NORMAL);
    let down_normal = piece.rotation.mul_vec3(-UP_NORMAL);

    vertices.extend([
        // Front face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        // Back face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        // Top face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        // Bottom face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        // Right face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        // Left face
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece
                    .rotation
                    .mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
    ]);

    indices.extend([
        0 + index_offset,
        1 + index_offset,
        2 + index_offset,
        0 + index_offset,
        2 + index_offset,
        3 + index_offset, // front
        4 + index_offset,
        5 + index_offset,
        6 + index_offset,
        4 + index_offset,
        6 + index_offset,
        7 + index_offset, // back
        8 + index_offset,
        9 + index_offset,
        10 + index_offset,
        8 + index_offset,
        10 + index_offset,
        11 + index_offset, // top
        12 + index_offset,
        13 + index_offset,
        14 + index_offset,
        12 + index_offset,
        14 + index_offset,
        15 + index_offset, // bottom
        16 + index_offset,
        17 + index_offset,
        18 + index_offset,
        16 + index_offset,
        18 + index_offset,
        19 + index_offset, // right
        20 + index_offset,
        21 + index_offset,
        22 + index_offset,
        20 + index_offset,
        22 + index_offset,
        23 + index_offset, // left
    ]);
}
