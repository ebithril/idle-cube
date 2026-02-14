use ::rand::distr::{Distribution, StandardUniform};
use ::rand::Rng;
use bitflags::bitflags;
use macroquad::math::Quat;

bitflags! {
    #[derive(Clone, Eq, PartialEq)]
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
        const Prime = 0b10000000;
    }
}

impl Move {
    pub fn is_prime(&self) -> bool {
        (self.clone() & Self::Prime) == Self::Prime
    }

    pub fn get_prime_sign(&self) -> f32 {
        if self.contains(Move::U) || self.contains(Move::R) || self.contains(Move::F) {
            1.
        } else if self.contains(Move::D) || self.contains(Move::L) || self.contains(Move::B) {
            -1.
        } else {
            panic!("This should never happen");
        }
    }

    pub fn get_sign(&self) -> f32 {
        if self.is_prime() {
            self.get_prime_sign()
        } else {
            -1. * self.get_prime_sign()
        }
    }

    pub fn create_rotation(&self, angle: f32) -> Quat {
        if self.contains(Move::U) || self.contains(Move::D) {
            Quat::from_rotation_y(angle)
        } else if self.contains(Move::R) || self.contains(Move::L) {
            Quat::from_rotation_x(angle)
        } else if self.contains(Move::F) || self.contains(Move::B) {
            Quat::from_rotation_z(angle)
        } else {
            panic!("This should never happen");
        }
    }
}

impl Distribution<Move> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Move {
        match rng.random_range(0..12) {
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
            _ => Move::BPrime,
        }
    }
}
