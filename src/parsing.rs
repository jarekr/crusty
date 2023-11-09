use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RustyConfig {
    pub version: u8,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}
use std::{path::Path, iter};

use bilge::prelude::*;

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

#[bitsize(128)]
#[derive(DebugBits, Default)]
pub struct Position{
    pub board: [PieceInPlay; 64]
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

        let mut pos: Position = Position::default();

        for (rank, pieces) in position_parts.iter().rev().enumerate() {
            for pc in pieces.chars() {
                let pieces: &[PieceInPlay] = match pc {
                    WHITE_PAWN_C => &[WHITE_PAWN][..1],
                    WHITE_KNIGHT_C => &[WHITE_KNIGHT][..1],
                    WHITE_BISHOP_C => &[WHITE_BISHOP][..1],
                    WHITE_QUEEN_C => &[WHITE_QUEEN][..1],
                    WHITE_KING_C => &[WHITE_KING][..1],
                    WHITE_ROOK_C => &[WHITE_ROOK][..1],
                    BLACK_PAWN_C => &[BLACK_PAWN][..1],
                    BLACK_KNIGHT_C => &[BLACK_KNIGHT][..1],
                    BLACK_BISHOP_C => &[BLACK_BISHOP][..1],
                    BLACK_QUEEN_C => &[BLACK_QUEEN][..1],
                    BLACK_KING_C => &[BLACK_KING][..1],
                    BLACK_ROOK_C => &[BLACK_ROOK][..1],
                    pc if pc.is_ascii_digit() => {
                        let digit = pc.to_digit(10).unwrap();
                        match digit {
                            1 => &[EMPTY][..1],
                            2 => &[EMPTY, EMPTY][..2],
                            3 => &[EMPTY, EMPTY, EMPTY][..3],
                            4 => &[EMPTY, EMPTY, EMPTY, EMPTY][..4],
                            5 => &[EMPTY, EMPTY, EMPTY, EMPTY, EMPTY][..5],
                            6 => &[EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY][..6],
                            7 => &[EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY][..7],
                            8 => &[EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY, EMPTY][..8],
                            _ => panic!("invalid digit in fen")
                        }
                    },
                    other => panic!("error, invalid char in fen: {}", other),
                };
                let mut count = 0;
                for piece in pieces.iter() {
                    pos.board()[count] = *piece;
                    count += 1;
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

const WHITE_ROOK: PieceInPlay   = PieceInPlay { value: 0b0000_0000_0000_0001 };
const WHITE_KNIGHT: PieceInPlay = PieceInPlay::new(Piece::Knight, Side::White);
const WHITE_BISHOP: PieceInPlay = PieceInPlay::new(Piece::Bishop, Side::White);
const WHITE_QUEEN: PieceInPlay  = PieceInPlay::new(Piece::Queen, Side::White);
const WHITE_KING: PieceInPlay   = PieceInPlay::new(Piece::King, Side::White);
const WHITE_PAWN: PieceInPlay   = PieceInPlay::new(Piece::Pawn, Side::White);
const BLACK_ROOK: PieceInPlay   = PieceInPlay::new(Piece::Rook, Side::Black);
const BLACK_KNIGHT: PieceInPlay = PieceInPlay::new(Piece::Knight, Side::Black);
const BLACK_BISHOP: PieceInPlay = PieceInPlay::new(Piece::Bishop, Side::Black);
const BLACK_QUEEN: PieceInPlay  = PieceInPlay::new(Piece::Queen, Side::Black);
const BLACK_KING: PieceInPlay   = PieceInPlay::new(Piece::King, Side::Black);
const BLACK_PAWN: PieceInPlay   = PieceInPlay::new(Piece::Pawn, Side::Black);
const EMPTY: PieceInPlay = PieceInPlay::new(Piece::Empty, Side::Black);

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
pub struct InventoryItem {
    pub id: u32,
    pub name: String,
    pub ipv4: u32,
    pub hostname: String,
    pub notes: String,
}

impl InventoryItem {
    pub fn queryById(db: Db, id: u32) -> Option<InventoryItem> {
        let mut stmt = db.conn
            .prepare("SELECT * from inventory where id = :id order by id asc").expect("perpare failed");
        let mut result = stmt.query(named_params!{":id": id}).expect("query failed");
        match result.next().expect("next failed on select result") {
            Some(row) => Some(InventoryItem {
                   id: row.get(0).unwrap(),
                   ipv4: row.get(1).unwrap(),
                   name: row.get(2).unwrap(),
                   hostname: row.get(3).unwrap(),
                   notes: row.get(4).unwrap(),
            }),
            None => None
        }
    }
}
