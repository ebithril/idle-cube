use crate::moves::Move;
use macroquad::prelude::*;
use std::collections::VecDeque;

const UP_NORMAL: Vec3 = vec3(0., 1., 0.);
const RIGHT_NORMAL: Vec3 = vec3(1., 0., 0.);
const FRONT_NORMAL: Vec3 = vec3(0., 0., 1.);

const XMAX: usize = 3;
const YMAX: usize = 3;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

impl Direction {
    fn to_vec3(self) -> Vec3 {
        match self {
            Self::Up => UP_NORMAL,
            Self::Down => -UP_NORMAL,
            Self::Left => -RIGHT_NORMAL,
            Self::Right => RIGHT_NORMAL,
            Self::Front => FRONT_NORMAL,
            Self::Back => -FRONT_NORMAL,
        }
    }

    fn rotate_x(self, sign: i8) -> Self {
        match (self, sign) {
            (Self::Up, 1) | (Self::Down, -1) => Self::Front,
            (Self::Front, 1) | (Self::Back, -1) => Self::Down,
            (Self::Down, 1) | (Self::Up, -1) => Self::Back,
            (Self::Back, 1) | (Self::Front, -1) => Self::Up,
            _ => self,
        }
    }

    fn rotate_y(self, sign: i8) -> Self {
        match (self, sign) {
            (Self::Front, 1) | (Self::Back, -1) => Self::Right,
            (Self::Right, 1) | (Self::Left, -1) => Self::Back,
            (Self::Back, 1) | (Self::Front, -1) => Self::Left,
            (Self::Left, 1) | (Self::Right, -1) => Self::Front,
            _ => self,
        }
    }

    fn rotate_z(self, sign: i8) -> Self {
        match (self, sign) {
            (Self::Right, 1) | (Self::Left, -1) => Self::Up,
            (Self::Up, 1) | (Self::Down, -1) => Self::Left,
            (Self::Left, 1) | (Self::Right, -1) => Self::Down,
            (Self::Down, 1) | (Self::Up, -1) => Self::Right,
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Orientation {
    right: Direction,
    up: Direction,
    front: Direction,
}

impl Orientation {
    const IDENTITY: Self = Self {
        right: Direction::Right,
        up: Direction::Up,
        front: Direction::Front,
    };

    fn rotate(self, move_to_make: Move) -> Self {
        if move_to_make.is_double() {
            let quarter_turn = move_to_make & !Move::Double;
            return self.rotate(quarter_turn).rotate(quarter_turn);
        }

        let sign = move_to_make.get_quarter_turn_sign();
        let rotate_direction = |direction: Direction| {
            if move_to_make.intersects(Move::U | Move::D) {
                direction.rotate_y(sign)
            } else if move_to_make.intersects(Move::R | Move::L) {
                direction.rotate_x(sign)
            } else {
                direction.rotate_z(sign)
            }
        };

        Self {
            right: rotate_direction(self.right),
            up: rotate_direction(self.up),
            front: rotate_direction(self.front),
        }
    }

    fn to_quat(self) -> Quat {
        let rotation = Mat3::from_cols(
            self.right.to_vec3(),
            self.up.to_vec3(),
            self.front.to_vec3(),
        );
        Quat::from_mat3(&rotation)
    }

    fn transform(self, direction: Direction) -> Direction {
        match direction {
            Direction::Right => self.right,
            Direction::Left => self.right.opposite(),
            Direction::Up => self.up,
            Direction::Down => self.up.opposite(),
            Direction::Front => self.front,
            Direction::Back => self.front.opposite(),
        }
    }
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }
}

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
    home_index: usize,
    color: PieceColor,
    orientation: Orientation,
}

#[derive(Clone)]
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

        for (home_index, color) in piece_colors.into_iter().enumerate() {
            pieces.push(Piece {
                home_index,
                color,
                orientation: Orientation::IDENTITY,
            });
        }

        Self {
            pieces,
            alg: VecDeque::new(),
            move_time: 1.0,
            alg_time: 0.,
        }
    }

    pub fn queue_moves(&mut self, moves: impl IntoIterator<Item = Move>) {
        self.alg.extend(moves);
    }

    pub fn is_idle(&self) -> bool {
        self.alg.is_empty()
    }

    pub fn daisy_count(&self) -> usize {
        const DAISY_POSITIONS: [(usize, usize, usize); 4] =
            [(1, 0, 0), (0, 0, 1), (2, 0, 1), (1, 0, 2)];

        DAISY_POSITIONS
            .into_iter()
            .filter(|&(x, y, z)| {
                let piece = &self.pieces[get_idx(x, y, z)];
                is_white_edge(piece)
                    && piece.orientation.transform(Direction::Up) == Direction::Down
            })
            .count()
    }

    pub fn is_daisy_complete(&self) -> bool {
        self.daisy_count() == 4
    }

    pub fn white_cross_count(&self) -> usize {
        const CROSS_POSITIONS: [(usize, usize, usize); 4] =
            [(1, 2, 0), (0, 2, 1), (2, 2, 1), (1, 2, 2)];

        CROSS_POSITIONS
            .into_iter()
            .filter(|&(x, y, z)| {
                let position = get_idx(x, y, z);
                let piece = &self.pieces[position];
                piece.home_index == position
                    && piece.orientation.transform(Direction::Up) == Direction::Up
            })
            .count()
    }

    pub fn is_white_cross_complete(&self) -> bool {
        self.white_cross_count() == 4
    }

    pub fn first_layer_corner_count(&self) -> usize {
        const CORNER_POSITIONS: [(usize, usize, usize); 4] =
            [(0, 2, 0), (2, 2, 0), (0, 2, 2), (2, 2, 2)];

        CORNER_POSITIONS
            .into_iter()
            .filter(|&(x, y, z)| {
                let position = get_idx(x, y, z);
                let piece = &self.pieces[position];
                piece.home_index == position && piece.orientation == Orientation::IDENTITY
            })
            .count()
    }

    pub fn is_first_layer_complete(&self) -> bool {
        self.is_white_cross_complete() && self.first_layer_corner_count() == 4
    }

    pub fn second_layer_edge_count(&self) -> usize {
        const EDGE_POSITIONS: [(usize, usize, usize); 4] =
            [(0, 1, 0), (2, 1, 0), (0, 1, 2), (2, 1, 2)];

        EDGE_POSITIONS
            .into_iter()
            .filter(|&(x, y, z)| {
                let position = get_idx(x, y, z);
                let piece = &self.pieces[position];
                piece.home_index == position && piece.orientation == Orientation::IDENTITY
            })
            .count()
    }

    pub fn is_second_layer_complete(&self) -> bool {
        self.is_first_layer_complete() && self.second_layer_edge_count() == 4
    }

    pub fn yellow_cross_count(&self) -> usize {
        const YELLOW_EDGE_POSITIONS: [(usize, usize, usize); 4] =
            [(1, 0, 0), (0, 0, 1), (2, 0, 1), (1, 0, 2)];

        YELLOW_EDGE_POSITIONS
            .into_iter()
            .filter(|&(x, y, z)| {
                let piece = &self.pieces[get_idx(x, y, z)];
                is_yellow_edge(piece)
                    && piece.orientation.transform(Direction::Down) == Direction::Down
            })
            .count()
    }

    pub fn is_yellow_cross_complete(&self) -> bool {
        self.is_second_layer_complete() && self.yellow_cross_count() == 4
    }

    pub(crate) fn second_layer_edges_on_down_count(&self) -> usize {
        self.pieces
            .iter()
            .enumerate()
            .filter(|(position, piece)| {
                is_second_layer_edge(piece) && get_3d_index(*position).1 == 0
            })
            .count()
    }

    pub(crate) fn first_unsolved_layer_corner_sides(
        &self,
    ) -> Option<(usize, Direction, Direction)> {
        const CORNER_POSITIONS: [(usize, usize, usize); 4] =
            [(0, 2, 0), (2, 2, 0), (0, 2, 2), (2, 2, 2)];

        CORNER_POSITIONS.into_iter().find_map(|(x, y, z)| {
            let position = get_idx(x, y, z);
            let piece = &self.pieces[position];
            if piece.home_index == position && piece.orientation == Orientation::IDENTITY {
                return None;
            }

            let x_side = if x == 0 {
                Direction::Left
            } else {
                Direction::Right
            };
            let z_side = if z == 0 {
                Direction::Back
            } else {
                Direction::Front
            };
            Some((position, x_side, z_side))
        })
    }

    pub(crate) fn piece_state_key(
        &self,
        home_index: usize,
    ) -> (usize, Direction, Direction, Direction) {
        let (position, piece) = self
            .pieces
            .iter()
            .enumerate()
            .find(|(_, piece)| piece.home_index == home_index)
            .expect("every cubie identity must be present");
        (
            position,
            piece.orientation.right,
            piece.orientation.up,
            piece.orientation.front,
        )
    }

    pub(crate) fn is_piece_solved(&self, home_index: usize) -> bool {
        let piece = &self.pieces[home_index];
        piece.home_index == home_index && piece.orientation == Orientation::IDENTITY
    }

    pub(crate) fn piece_layer(&self, home_index: usize) -> usize {
        self.piece_position(home_index).1
    }

    pub(crate) fn piece_position(&self, home_index: usize) -> (usize, usize, usize) {
        get_3d_index(self.piece_state_key(home_index).0)
    }

    pub(crate) fn home_position(&self, home_index: usize) -> (usize, usize, usize) {
        get_3d_index(home_index)
    }

    pub(crate) fn white_edge_home_side_at(
        &self,
        x: usize,
        y: usize,
        z: usize,
    ) -> Option<Direction> {
        let piece = &self.pieces[get_idx(x, y, z)];
        if !is_white_edge(piece) || piece.orientation.transform(Direction::Up) != Direction::Down {
            return None;
        }

        let (home_x, _, home_z) = get_3d_index(piece.home_index);
        match (home_x, home_z) {
            (0, 1) => Some(Direction::Left),
            (2, 1) => Some(Direction::Right),
            (1, 0) => Some(Direction::Back),
            (1, 2) => Some(Direction::Front),
            _ => None,
        }
    }

    pub(crate) fn state_key(&self) -> Vec<(usize, Direction, Direction, Direction)> {
        self.pieces
            .iter()
            .map(|piece| {
                (
                    piece.home_index,
                    piece.orientation.right,
                    piece.orientation.up,
                    piece.orientation.front,
                )
            })
            .collect()
    }

    pub fn create_mesh(&self) -> Mesh {
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u16>::new();

        for (i, piece) in self.pieces.iter().enumerate() {
            add_piece_verticies(&mut vertices, &mut indices, i, piece, 1.0);
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
            self.execute_move(current_move.unwrap());
        }
    }

    fn get_rotation_angle(&self, expected_move: Move) -> Mat4 {
        let current_move = match self.alg.front() {
            Some(m) => *m,
            None => return Mat4::IDENTITY,
        };

        if !current_move.contains(expected_move) {
            return Mat4::IDENTITY;
        }

        let sign = current_move.get_sign();

        let rot_angle = current_move.get_angle() * self.alg_time / self.move_time;

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
            for (i, e) in edges.iter().rev().enumerate() {
                r_edges[i] = *e;
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
            for (i, e) in corners.iter().rev().enumerate() {
                r_corners[i] = *e;
            }

            r_corners
        } else {
            corners
        }
    }

    pub fn execute_move(&mut self, move_to_make: Move) {
        if move_to_make.is_double() {
            let quarter_turn = move_to_make & !Move::Double;
            self.execute_move(quarter_turn);
            self.execute_move(quarter_turn);
            return;
        }

        let edges = self.get_edge_idx_for_move(move_to_make);
        let corners = self.get_corner_idx_for_move(move_to_make);

        let mut buffer = self.pieces[edges[0]].clone();
        buffer.orientation = buffer.orientation.rotate(move_to_make);
        for i in 0..edges.len() - 1 {
            self.pieces[edges[i]] = self.pieces[edges[i + 1]].clone();
            self.pieces[edges[i]].orientation =
                self.pieces[edges[i]].orientation.rotate(move_to_make);
        }
        self.pieces[edges[3]] = buffer;

        let mut buffer = self.pieces[corners[0]].clone();
        buffer.orientation = buffer.orientation.rotate(move_to_make);
        for i in 0..corners.len() - 1 {
            self.pieces[corners[i]] = self.pieces[corners[i + 1]].clone();
            self.pieces[corners[i]].orientation =
                self.pieces[corners[i]].orientation.rotate(move_to_make);
        }
        self.pieces[corners[3]] = buffer;
    }

    pub fn is_solved(&self) -> bool {
        self.pieces.iter().enumerate().all(|(position, piece)| {
            piece.home_index == position && piece.orientation == Orientation::IDENTITY
        })
    }
}

fn is_white_edge(piece: &Piece) -> bool {
    let (x, y, z) = get_3d_index(piece.home_index);
    y == 2 && [x, y, z].into_iter().filter(|&axis| axis != 1).count() == 2
}

fn is_second_layer_edge(piece: &Piece) -> bool {
    let (x, y, z) = get_3d_index(piece.home_index);
    y == 1 && x != 1 && z != 1
}

fn is_yellow_edge(piece: &Piece) -> bool {
    let (x, y, z) = get_3d_index(piece.home_index);
    y == 0 && [x, y, z].into_iter().filter(|&axis| axis != 1).count() == 2
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
    let piece_rotation = piece.orientation.to_quat();
    let index_offset = vertices.len() as u16;
    let piece_size = size / 3.0;
    let float_index = piece_index as f32 + 0.5;

    let front_normal = piece_rotation.mul_vec3(FRONT_NORMAL);
    let back_normal = piece_rotation.mul_vec3(-FRONT_NORMAL);
    let right_normal = piece_rotation.mul_vec3(RIGHT_NORMAL);
    let left_normal = piece_rotation.mul_vec3(-RIGHT_NORMAL);
    let up_normal = piece_rotation.mul_vec3(UP_NORMAL);
    let down_normal = piece_rotation.mul_vec3(-UP_NORMAL);

    vertices.extend([
        // Front face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.front.into(),
            normal: Vec4::new(front_normal.x, front_normal.y, front_normal.z, float_index),
        },
        // Back face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.back.into(),
            normal: Vec4::new(back_normal.x, back_normal.y, back_normal.z, float_index),
        },
        // Top face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.up.into(),
            normal: Vec4::new(up_normal.x, up_normal.y, up_normal.z, float_index),
        },
        // Bottom face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.down.into(),
            normal: Vec4::new(down_normal.x, down_normal.y, down_normal.z, float_index),
        },
        // Right face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, -piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(piece_size, -piece_size, piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.right.into(),
            normal: Vec4::new(right_normal.x, right_normal.y, right_normal.z, float_index),
        },
        // Left face
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, -piece_size)),
            uv: vec2(0., 0.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, -piece_size, piece_size)),
            uv: vec2(1., 0.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, piece_size)),
            uv: vec2(1., 1.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
        Vertex {
            position: piece_center
                + piece_rotation.mul_vec3(vec3(-piece_size, piece_size, -piece_size)),
            uv: vec2(0., 1.),
            color: piece_color.left.into(),
            normal: Vec4::new(left_normal.x, left_normal.y, left_normal.z, float_index),
        },
    ]);

    indices.extend([
        index_offset,
        1 + index_offset,
        2 + index_offset,
        index_offset,
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn new_cube_is_solved() {
        assert!(Cube::new().is_solved());
    }

    #[test]
    fn a_single_move_is_not_solved() {
        for move_to_make in MOVES {
            let mut cube = Cube::new();
            cube.execute_move(move_to_make);
            assert!(!cube.is_solved(), "{move_to_make:?}");
        }
    }

    #[test]
    fn a_move_followed_by_its_inverse_restores_the_cube() {
        for move_to_make in MOVES {
            let mut cube = Cube::new();
            cube.execute_move(move_to_make);
            cube.execute_move(move_to_make.inverse());
            assert!(cube.is_solved(), "{move_to_make:?}");
        }
    }

    #[test]
    fn four_quarter_turns_restore_the_cube() {
        for move_to_make in MOVES {
            let mut cube = Cube::new();
            for _ in 0..4 {
                cube.execute_move(move_to_make);
            }
            assert!(cube.is_solved(), "{move_to_make:?}");
        }
    }

    #[test]
    fn an_algorithm_followed_by_its_inverse_restores_the_cube() {
        let algorithm = [Move::R, Move::U, Move::RPrime, Move::UPrime, Move::F];
        let mut cube = Cube::new();

        for move_to_make in algorithm {
            cube.execute_move(move_to_make);
        }
        assert!(!cube.is_solved());

        for move_to_make in algorithm.into_iter().rev() {
            cube.execute_move(move_to_make.inverse());
        }
        assert!(cube.is_solved());
    }

    #[test]
    fn exact_orientation_matches_move_quaternion() {
        for move_to_make in MOVES {
            let exact = Orientation::IDENTITY.rotate(move_to_make).to_quat();
            let expected =
                move_to_make.create_rotation(move_to_make.get_angle() * move_to_make.get_sign());

            for axis in [RIGHT_NORMAL, UP_NORMAL, FRONT_NORMAL] {
                assert!(
                    exact
                        .mul_vec3(axis)
                        .abs_diff_eq(expected.mul_vec3(axis), 0.000_001),
                    "{move_to_make:?}"
                );
            }
        }
    }
}
