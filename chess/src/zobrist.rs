use std::ops::{BitXor, BitXorAssign, Not};

use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;

//#################################################################################################
//
//                                       Zobrist table
//
//#################################################################################################

// The zobrist keys.
static mut KEYS: [[[Zobrist; 2]; 6]; 64] = [[[Zobrist::ZERO; 2]; 6]; 64];

// The xorshift* algorithm for 64 bits numbers, producing
// good enough pseudo-random numbers.
#[cold]
fn xorshift(seed: &mut u64) -> Zobrist {
    let mut x = *seed;
    x ^= x.wrapping_shl(13);
    x ^= x.wrapping_shr(7);
    x ^= x.wrapping_shl(17);
    *seed = x;
    Zobrist(x.wrapping_mul(0x2545F4914F6CDD1D))
}

// Initializes the zobrist keys at the beginning of the program.
#[cold]
pub(crate) unsafe fn init() {
    // Changing the seed may make the cuckoo init() non terminating.
    let mut seed = 0x0C3B301A1AF7EE42;

    for sq in Square::SQUARES {
        for piece in Piece::PIECES {
            KEYS[sq.idx()][piece.idx()][Color::White.idx()] = xorshift(&mut seed);
            KEYS[sq.idx()][piece.idx()][Color::Black.idx()] = xorshift(&mut seed);
        }
    }
}

//#################################################################################################
//
//                                        struct Zobrist
//
//#################################################################################################

/// A zobrist key, that may be used for hashing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zobrist(u64);

// ================================ pub impl

impl Zobrist {
    /// The zero of that type.
    pub const ZERO: Zobrist = Zobrist(0);

    /// First hash function for indexing the cuckoo table.
    #[inline(always)]
    pub fn h1(self) -> usize {
        (self.0.wrapping_shr(32) & 0x1FFF) as usize
    }

    /// Second hash function for indexing the cuckoo table.
    #[inline(always)]
    pub fn h2(self) -> usize {
        (self.0.wrapping_shr(48) & 0x1FFF) as usize
    }
}

// ================================ traits impl

impl From<(Color, Piece, Square)> for Zobrist {
    /// Hashes a color, piece, square triplet.
    #[inline(always)]
    fn from((color, piece, sq): (Color, Piece, Square)) -> Zobrist {
        unsafe {
            KEYS[sq.idx()][piece.idx()][color.idx()]
        }
    }
}

impl BitXor<Zobrist> for Zobrist {
    type Output = Zobrist;

    #[inline(always)]
    fn bitxor(self, rhs: Zobrist) -> Zobrist {
        Zobrist(self.0.bitxor(rhs.0))
    }
}

impl BitXorAssign<Zobrist> for Zobrist {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Zobrist) {
        self.0.bitxor_assign(rhs.0);
    }
}

impl Not for Zobrist {
    type Output = Zobrist;

    #[inline(always)]
    fn not(self) -> Zobrist {
        Zobrist(self.0.not())
    }
}