use anyhow::{Error, Result};

use crate::color::Color;

//#################################################################################################
//
//                                        enum Piece
//
//#################################################################################################

/// Represents a piece.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Piece {
    Pawn = 0,
    Rook = 1,
    Knight = 2,
    Bishop = 3,
    Queen = 4,
    King = 5,
}

// ================================ pub impl

impl Piece {
    /// The list of all pieces, in order.
    pub const PIECES: [Piece; 6] = [
        Piece::Pawn, Piece::Rook, Piece::Knight, 
        Piece::Bishop, Piece::Queen, Piece::King,
    ];

    /// The pieces a pawn promotes to, in order from most to least interesting.
    pub const PROMOTES: [Piece; 4] = [
        Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight,
    ];

    /// Tries to parse a piece from a single char.
    pub fn from_char(c: char) -> Result<(Color, Piece), Error> {
        match c {
            'P' => Ok((Color::White, Piece::Pawn)),
            'R' => Ok((Color::White, Piece::Rook)),
            'N' => Ok((Color::White, Piece::Knight)),
            'B' => Ok((Color::White, Piece::Bishop)),
            'Q' => Ok((Color::White, Piece::Queen)),
            'K' => Ok((Color::White, Piece::King)),
            'p' => Ok((Color::Black, Piece::Pawn)),
            'r' => Ok((Color::Black, Piece::Rook)),
            'n' => Ok((Color::Black, Piece::Knight)),
            'b' => Ok((Color::Black, Piece::Bishop)),
            'q' => Ok((Color::Black, Piece::Queen)),
            'k' => Ok((Color::Black, Piece::King)),
            _ => Err(Error::msg("unrecognized piece literal")),
        }
    }

    /// Gives the char corresponding to a piece of this color:
    /// Upper case for white, lower case for black.
    pub fn as_char(self, color: Color) -> char {
        match (color, self) {
            (Color::White, Piece::Pawn)   => 'P',
            (Color::White, Piece::Rook)   => 'R',
            (Color::White, Piece::Knight) => 'N',
            (Color::White, Piece::Bishop) => 'B',
            (Color::White, Piece::Queen)  => 'Q',
            (Color::White, Piece::King)   => 'K',
            (Color::Black, Piece::Pawn)   => 'p',
            (Color::Black, Piece::Rook)   => 'r',
            (Color::Black, Piece::Knight) => 'n',
            (Color::Black, Piece::Bishop) => 'b',
            (Color::Black, Piece::Queen)  => 'q',
            (Color::Black, Piece::King)   => 'k',
        }
    }
}

// ================================ pub(crate) impl

impl Piece {
    /// Returns the piece corresponding to that number, assumes 0 <= i < 6
    pub(crate) unsafe fn from_unchecked(i: u8) -> Piece {
        *Piece::PIECES.get_unchecked(i as usize)
    }
}

// ================================ traits impl

impl From<u8> for Piece {
    /// Creates a piece from a number. See codes in number definition.
    #[inline]
    fn from(i: u8) -> Piece {
        Piece::PIECES[i as usize]
    }
}

impl From<Piece> for usize {
    /// Use the piece as an index.
    #[inline]
    fn from(piece: Piece) -> usize {
        piece as usize
    }
}