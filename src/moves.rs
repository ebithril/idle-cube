use ::rand::distr::{Distribution, StandardUniform};
use ::rand::Rng;
use bitflags::bitflags;

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
    pub fn is_prime(self) -> bool {
        (self & Self::Prime) == Self::Prime
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
