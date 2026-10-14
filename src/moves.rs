use ::rand::distr::{Distribution, StandardUniform};
use ::rand::Rng;
use bitflags::bitflags;
use macroquad::math::Quat;

pub fn scramble_rng() -> ::rand::rngs::StdRng {
    use ::rand::SeedableRng;
    // Miniquad provides this clock on both native and browser platforms.
    ::rand::rngs::StdRng::seed_from_u64(macroquad::miniquad::date::now().to_bits())
}

bitflags! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Move : u32 {
        const U = 0b00000001;
        const UPrime = 0b10000001;
        const R = 0b00000010;
        const RPrime = 0b10000010;
        const F = 0b00000100;
        const FPrime = 0b10000100;
        const D = 0b00001000;
        const DPrime = 0b10001000;
        const L = 0b00010000;
        const LPrime = 0b10010000;
        const B = 0b00100000;
        const BPrime = 0b10100000;
        const U2 = 0b01000001;
        const R2 = 0b01000010;
        const F2 = 0b01000100;
        const D2 = 0b01001000;
        const L2 = 0b01010000;
        const B2 = 0b01100000;
        const Rw = 0b100000010;
        const RwPrime = 0b110000010;
        const Fw = 0b100000100;
        const FwPrime = 0b110000100;
        const Bw = 0b100100000;
        const BwPrime = 0b110100000;
        const Wide = 0b100000000;
        const M = 0b1000000000;
        const M2 = 0b1001000000;
        const MPrime = 0b1010000000;
        const S = 0b10000000000;
        const S2 = 0b10001000000;
        const SPrime = 0b10010000000;
        const Double = 0b01000000;
        const Prime = 0b10000000;
    }
}

impl Move {
    pub fn is_prime(&self) -> bool {
        (*self & Self::Prime) == Self::Prime
    }

    pub fn get_prime_sign(&self) -> f32 {
        if self.contains(Move::U)
            || self.contains(Move::R)
            || self.contains(Move::F)
            || self.contains(Move::S)
        {
            1.
        } else if self.contains(Move::D)
            || self.contains(Move::L)
            || self.contains(Move::B)
            || self.contains(Move::M)
        {
            -1.
        } else {
            panic!("This should never happen");
        }
    }

    pub fn get_sign(&self) -> f32 {
        if self.is_prime() {
            self.get_prime_sign()
        } else {
            -self.get_prime_sign()
        }
    }

    pub fn get_quarter_turn_sign(&self) -> i8 {
        self.get_sign() as i8
    }

    pub fn is_double(&self) -> bool {
        self.contains(Self::Double)
    }

    pub fn is_wide(&self) -> bool {
        self.contains(Self::Wide)
    }

    pub fn get_angle(&self) -> f32 {
        if self.is_double() {
            std::f32::consts::PI
        } else {
            std::f32::consts::FRAC_PI_2
        }
    }

    pub fn inverse(&self) -> Self {
        if self.is_double() {
            *self
        } else if self.is_prime() {
            *self & !Self::Prime
        } else {
            *self | Self::Prime
        }
    }

    pub fn same_face(&self, other: Self) -> bool {
        const FACES: Move = Move::U
            .union(Move::R)
            .union(Move::F)
            .union(Move::D)
            .union(Move::L)
            .union(Move::B)
            .union(Move::M)
            .union(Move::S);

        (*self & FACES) == (other & FACES)
    }

    pub fn create_rotation(&self, angle: f32) -> Quat {
        if self.contains(Move::U) || self.contains(Move::D) {
            Quat::from_rotation_y(angle)
        } else if self.contains(Move::R) || self.contains(Move::L) || self.contains(Move::M) {
            Quat::from_rotation_x(angle)
        } else if self.contains(Move::F) || self.contains(Move::B) || self.contains(Move::S) {
            Quat::from_rotation_z(angle)
        } else {
            panic!("This should never happen");
        }
    }
}

impl Distribution<Move> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Move {
        match rng.random_range(0..18) {
            0 => Move::U,
            1 => Move::UPrime,
            2 => Move::R,
            3 => Move::RPrime,
            4 => Move::F,
            5 => Move::FPrime,
            6 => Move::D,
            7 => Move::DPrime,
            8 => Move::L,
            9 => Move::LPrime,
            10 => Move::B,
            11 => Move::BPrime,
            12 => Move::U2,
            13 => Move::R2,
            14 => Move::F2,
            15 => Move::D2,
            16 => Move::L2,
            _ => Move::B2,
        }
    }
}

pub fn random_scramble<R: Rng + ?Sized>(rng: &mut R, length: usize) -> Vec<Move> {
    let mut scramble = Vec::with_capacity(length);

    while scramble.len() < length {
        let next_move: Move = rng.random();
        if scramble
            .last()
            .is_none_or(|previous: &Move| !previous.same_face(next_move))
        {
            scramble.push(next_move);
        }
    }

    scramble
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::rand::rngs::StdRng;
    use ::rand::SeedableRng;

    #[test]
    fn scrambles_never_repeat_a_face() {
        let mut rng = StdRng::seed_from_u64(0x5eed);

        for _ in 0..100 {
            let scramble = random_scramble(&mut rng, 20);
            assert_eq!(scramble.len(), 20);
            assert!(scramble
                .windows(2)
                .all(|moves| !moves[0].same_face(moves[1])));
        }
    }
}
