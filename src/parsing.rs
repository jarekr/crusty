use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RustyConfig {
    pub version: u8,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

use bilge::prelude::*;

use once_cell::sync::Lazy;

// starting postion
// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
// ... after 1. e4
// rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
// ... after 1. ...c5
// rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2
// ... after 2. Nf3
// rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2

// 64 position, each position in 1 of 13 states: pawn, rook, (k)night, bishop, queen, king, x2 for other color, empty
// side to move = +1
// castling = white k/q, black k/q, = 3 + 3 + 1= 6 possible
// en passant square = 0-64
// halfmove clock: 1-50? used for the 50 move rule
// full move number: 1-100?

#[bitsize(3)]
#[derive(Debug, PartialEq, FromBits, Default, Clone)]
pub enum Piece {
    Rook, Knight, Bishop, Queen, King, Pawn,
    #[fallback]
    #[default]
    Empty,
}

#[bitsize(1)]
#[derive(Debug, PartialEq, FromBits, Default, Clone)]
pub enum Side {
    White,
    #[default]
    Black
}

#[bitsize(4)]
#[derive(DebugBits, PartialEq, FromBits, Default, Clone)]
pub struct PieceInPlay {
    pub piece: Piece,
    pub side: Side,
}

impl PieceInPlay {
    pub fn news(p: Piece, s: Side) -> PieceInPlay {
        PieceInPlay::new(p, s)
    }
}

#[derive(Debug)]
pub struct Position{
    pub board: [&'static Lazy<PieceInPlay>; 64]
}

impl Position {
    pub fn parse_from_str(fen: &str) -> Result<Position, &str> {

        let parts: Vec<_> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err("not enough parts");
        }
        let position_parts: Vec<_> = parts[0].split('/').collect();
        if position_parts.len() != 8 {
            return Err("not enough rows")
        }

        let mut pos: Position = Position { board:
            [
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
            ]
        };

        for (rank, pieces) in position_parts.iter().rev().enumerate() {
            let mut idx = 0;
            for pc in pieces.chars() {
                if pc.is_ascii_digit() {
                    let digit = pc.to_digit(10).unwrap();
                    if digit < 1 || digit > 8 {
                        panic!("invalid digit in fen")
                    }
                    for _ in 0..digit {
                        idx  += 1;
                    }

                } else {
                    pos.board[idx * rank] = match pc {
                        WHITE_PAWN_C => &WHITE_PAWN,
                        WHITE_KNIGHT_C => &WHITE_KNIGHT,
                        WHITE_BISHOP_C => &WHITE_BISHOP,
                        WHITE_QUEEN_C => &WHITE_QUEEN,
                        WHITE_KING_C => &WHITE_KING,
                        WHITE_ROOK_C => &WHITE_ROOK,
                        BLACK_PAWN_C => &BLACK_PAWN,
                        BLACK_KNIGHT_C => &BLACK_KNIGHT,
                        BLACK_BISHOP_C => &BLACK_BISHOP,
                        BLACK_QUEEN_C => &BLACK_QUEEN,
                        BLACK_KING_C => &BLACK_KING,
                        BLACK_ROOK_C => &BLACK_ROOK,
                        other => panic!("error, invalid char in fen: {}", other),
                    };
                    idx += 1;
                }

            }
        }
        Ok(pos)
    }
}


// pieces
const WHITE_ROOK_C: char   = 'R';
const WHITE_KNIGHT_C: char = 'N';
const WHITE_BISHOP_C: char = 'B';
const WHITE_QUEEN_C: char  = 'Q';
const WHITE_KING_C: char   = 'K';
const WHITE_PAWN_C: char   = 'P';
const BLACK_ROOK_C: char   = 'r';
const BLACK_KNIGHT_C: char = 'n';
const BLACK_BISHOP_C: char = 'b';
const BLACK_QUEEN_C: char  = 'q';
const BLACK_KING_C: char   = 'k';
const BLACK_PAWN_C: char   = 'p';

static WHITE_ROOK: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Rook, Side::White) });
static WHITE_KNIGHT: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Knight, Side::White) });
static WHITE_BISHOP: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Bishop, Side::White) });
static WHITE_QUEEN: Lazy<PieceInPlay>  = Lazy::new( || { PieceInPlay::new(Piece::Queen, Side::White) });
static WHITE_KING: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::King, Side::White) });
static WHITE_PAWN: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Pawn, Side::White) });
static BLACK_ROOK: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Rook, Side::Black) });
static BLACK_KNIGHT: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Knight, Side::Black) });
static BLACK_BISHOP: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Bishop, Side::Black) });
static BLACK_QUEEN: Lazy<PieceInPlay>  = Lazy::new( || { PieceInPlay::new(Piece::Queen, Side::Black) });
static BLACK_KING: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::King, Side::Black) });
static BLACK_PAWN: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Pawn, Side::Black) });
static EMPTY: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Empty, Side::Black) });

// 4 bits = 16 == 12 pieces + 2
// 4 * 8slots == 32 bits per row
// 4 rows * 32 b/r == 128 bits to hold the board state
// move turn == 1 bit
// castling rights == 4 bits
// en passant square == 6 bits (or less...)
// halfmove_clock = 127 max? 7 bits
// fullmove_number 8 bits
// -------------------------------------
// 128 + 1 + 4 + 6 + 7 + 8 = 144 bits
// 144 bits = u64 + u64 + u16
// location = 6 bits?
//
// 6 + 4 = 10 bits * 32 = 320 bits or less
//
