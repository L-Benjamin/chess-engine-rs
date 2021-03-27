use chess::{Color, Game, Piece};

// The values of all pieces (except the king), ordered by the value
// of the piece as an integer
const PIECE_VALUES: [f32; 5] = [
    1.0, 5.0, 3.2, 3.3, 9.0
];

const PAWNS: [f32; 64] = [
	0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
	0.05, 0.1, 0.1, -0.2, -0.2, 0.1, 0.1, 0.05,
	0.05, -0.05, -0.1, 0.0, 0.0, -0.1, -0.05, 0.05,
	0.0, 0.0, 0.0, 0.2, 0.2, 0.0, 0.0, 0.0,
	0.05, 0.05, 0.1, 0.25, 0.25, 0.1, 0.05, 0.05,
	0.1, 0.1, 0.2, 0.3, 0.3, 0.2, 0.1, 0.1,
	0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
	0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

const ROOKS: [f32; 64] = [
	0.0, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0, 0.0,
	-0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
	-0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
	-0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
	-0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
	-0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.05,
	0.05, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.05,
	0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];

const KNIGHTS: [f32; 64] = [
	-0.5, -0.4, -0.3, -0.3, -0.3, -0.3, -0.4, -0.5,
	-0.4, -0.2, 0.0, 0.05, 0.05, 0.0, -0.2, -0.4,
	-0.3, 0.05, 0.1, 0.15, 0.15, 0.1, 0.05, -0.3,
	-0.3, 0.0, 0.15, 0.2, 0.2, 0.15, 0.0, -0.3,
	-0.3, 0.05, 0.15, 0.2, 0.2, 0.15, 0.05, -0.3,
	-0.3, 0.0, 0.1, 0.15, 0.15, 0.1, 0.0, -0.3,
	-0.4, -0.2, 0.0, 0.0, 0.0, 0.0, -0.2, -0.4,
	-0.5, -0.4, -0.3, -0.3, -0.3, -0.3, -0.4, -0.5,
];

const BISHOPS: [f32; 64] = [
	-0.2, -0.1, -0.1, -0.1, -0.1, -0.1, -0.1, -0.2,
	-0.1, 0.05, 0.0, 0.0, 0.0, 0.0, 0.05, -0.1,
	-0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, -0.1,
	-0.1, 0.0, 0.1, 0.1, 0.1, 0.1, 0.0, -0.1,
	-0.1, 0.05, 0.05, 0.1, 0.1, 0.05, 0.05, -0.1,
	-0.1, 0.0, 0.05, 0.1, 0.1, 0.05, 0.0, -0.1,
	-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1,
	-0.2, -0.1, -0.1, -0.1, -0.1, -0.1, -0.1, -0.2,
];

const QUEENS: [f32; 64] = [
	-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2,
	-0.1, 0.0, 0.05, 0.0, 0.0, 0.0, 0.0, -0.1,
	-0.1, 0.05, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1,
	0.0, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05,
	-0.05, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.05,
	-0.1, 0.0, 0.05, 0.05, 0.05, 0.05, 0.0, -0.1,
	-0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.1,
	-0.2, -0.1, -0.1, -0.05, -0.05, -0.1, -0.1, -0.2,
];

const KINGS_EARLY: [f32; 64] = [
	0.2, 0.3, 0.1, 0.0, 0.0, 0.1, 0.3, 0.2,
	0.2, 0.2, 0.0, 0.0, 0.0, 0.0, 0.2, 0.2,
	-0.1, -0.2, -0.2, -0.2, -0.2, -0.2, -0.2, -0.1,
	-0.2, -0.3, -0.3, -0.4, -0.4, -0.3, -0.3, -0.2,
	-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
	-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
	-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
	-0.3, -0.4, -0.4, -0.5, -0.5, -0.4, -0.4, -0.3,
];


const KINGS_ENDGAME: [f32; 64] = [
	-0.5, -0.3, -0.3, -0.3, -0.3, -0.3, -0.3, -0.5,
	-0.3, -0.3, 0.0, 0.0, 0.0, 0.0, -0.3, -0.3,
	-0.3, -0.1, 0.2, 0.3, 0.3, 0.2, -0.1, -0.3,
	-0.3, -0.1, 0.3, 0.4, 0.4, 0.3, -0.1, -0.3,
	-0.3, -0.1, 0.3, 0.4, 0.4, 0.3, -0.1, -0.3,
	-0.3, -0.1, 0.2, 0.3, 0.3, 0.2, -0.1, -0.3,
	-0.3, -0.2, -0.1, 0.0, 0.0, -0.1, -0.2, -0.3,
	-0.5, -0.4, -0.3, -0.2, -0.2, -0.3, -0.4, -0.5,
];

const TABLES: [[f32; 64]; 5] = [
    PAWNS, ROOKS, KNIGHTS,
    BISHOPS, QUEENS
];

pub(crate) fn eval(game: &Game) -> f32 {
    let board = game.get_board();
    let mut score = 0.0;

    for &piece in &Piece::PIECES[0..5] {
        for sq in board.get_bitboard(Color::White, piece).iter_squares() {
            score += PIECE_VALUES[piece as usize] + TABLES[piece as usize][sq as usize];
        }
        
        for sq in board.get_bitboard(Color::Black, piece).iter_squares() {
            score -= PIECE_VALUES[piece as usize] + TABLES[piece as usize][63 - sq as usize];
        }  
    }

    if game.is_endgame() {
        score += KINGS_ENDGAME[board.get_king_sq(Color::White) as usize];
        score -= KINGS_ENDGAME[63 - board.get_king_sq(Color::Black) as usize];
    } else {
        score += KINGS_EARLY[board.get_king_sq(Color::White) as usize];
        score -= KINGS_EARLY[63 - board.get_king_sq(Color::Black) as usize];
    }

    match game.get_color() {
        Color::White =>  score,
        Color::Black => -score,
    }
}