use serde::{Serialize, Deserialize};
use std::{path::PathBuf, collections::HashMap};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RustyConfig {
    pub version: u8,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

use bilge::prelude::*;

use once_cell::sync::Lazy;

use pgn_reader::{Visitor, Skip, RawHeader, BufferedReader, SanPlus};

use shakmaty::{CastlingMode, Chess, Position};


struct FenVisitor {
    pub pos: Chess,
    pub fens: Vec<BitPosition>,
}

impl FenVisitor {
    pub fn new() -> FenVisitor {
        FenVisitor { pos: Chess::default(), fens: Vec::new() }
    }
}

impl Visitor for FenVisitor {
    type Result = Vec<BitPosition>

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn san(&mut self, san_plus: SanPlus) {
        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            self.pos.play_unchecked(&m);
        }
        let self.pos.board.intoIter()
    }


    fn end_game(&mut self) -> Self::Result {
        ::std::mem::Replace(&mut self.fens, Vec::new())
    }
}


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

    pub fn to_char(&self) -> char {
        let mut c = match self.piece() {
            Piece::Pawn => WHITE_PAWN_C,
            Piece::Rook => WHITE_ROOK_C,
            Piece::Bishop => WHITE_BISHOP_C,
            Piece::King => WHITE_KING_C,
            Piece::Queen => WHITE_QUEEN_C,
            Piece::Empty => '_',
            Piece::Knight => WHITE_KNIGHT_C,
        };
        if self.side() == Side::Black {
            c = c.to_lowercase().next().unwrap();
        }
        c
    }
}

#[derive(Debug)]
pub struct BitPosition{
    pub board: [&'static Lazy<PieceInPlay>; 64]
}

impl BitPosition {
    pub fn new() -> BitPosition {
        let mut pos: BitPosition = BitPosition { board:
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
        pos
    }

    pub fn parse_from_str(fen: &str) -> Result<BitPosition, &str> {

        let parts: Vec<_> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err("not enough parts");
        }
        let position_parts: Vec<_> = parts[0].split('/').collect();
        if position_parts.len() != 8 {
            return Err("not enough rows")
        }

        let mut pos: BitPosition = BitPosition::new();

        let mut idx = 0;
        for pieces in position_parts.iter() {
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
                    pos.board[idx] = match pc {
                        WHITE_PAWN_C   => &WHITE_PAWN,
                        WHITE_KNIGHT_C => &WHITE_KNIGHT,
                        WHITE_BISHOP_C => &WHITE_BISHOP,
                        WHITE_QUEEN_C  => &WHITE_QUEEN,
                        WHITE_KING_C   => &WHITE_KING,
                        WHITE_ROOK_C   => &WHITE_ROOK,
                        BLACK_PAWN_C   => &BLACK_PAWN,
                        BLACK_KNIGHT_C => &BLACK_KNIGHT,
                        BLACK_BISHOP_C => &BLACK_BISHOP,
                        BLACK_QUEEN_C  => &BLACK_QUEEN,
                        BLACK_KING_C   => &BLACK_KING,
                        BLACK_ROOK_C   => &BLACK_ROOK,
                        other => panic!("error, invalid char in fen: {}", other),
                    };
                    idx += 1;
                }

            }
        }
        Ok(pos)
    }

    pub fn to_bits(&self) -> (u64, u64, u64, u64) {
        let mut r12: u64 = 0;
        let mut r34: u64 = 0;
        let mut r56: u64 = 0;
        let mut r78: u64 = 0;
        let mut shiftamt = 0;
        for (idx, &sq) in self.board.iter().enumerate() {
            let bob = sq.value.value();
            if idx < 16   {
                r12 += (bob as u64) << shiftamt;
                //println!("1 idx: {}, value: {}, shiftby:{}", idx, bob, shiftamt);
            } else if idx < 32 {
                r34 += (bob as u64) << shiftamt - 64;
                //println!("2 idx: {}, value: {}, shiftby:{}", idx, bob, shiftamt - 64);
            } else if idx < 48 {
                r56 += (bob as u64) << shiftamt - 128;
                //println!("3 idx: {}, value: {}, shiftby:{}", idx, bob, shiftamt - 128);
            } else {
                r78 += (bob as u64) << shiftamt - 192;
                //println!("4 idx: {}, value: {}, shiftby:{}", idx, bob, shiftamt - 192);
            }
            shiftamt += 4;
        }
        (r12, r34, r56, r78)
    }

    fn from_bits2(val: u64) -> [u8; 16] {
        let mut offset = 0;

        let mut bob: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        for idx in 0..16 {
            let p = ((val >> offset) & 0xfu64) as u8;
            bob[idx] = p;
            offset += 4;

        }
        bob

    }

    pub fn from_bits(r12: u64, r34: u64, r56: u64, r78: u64) -> Result<BitPosition, &'static str> {
        let mut pos = BitPosition::new();

        for (idx, pn) in BitPosition::from_bits2(r12).iter().enumerate() {
            pos.board[idx] = VAL_TO_PIECE.get(pn).unwrap();
        }
        for (idx, pn) in BitPosition::from_bits2(r34).iter().enumerate() {
            pos.board[idx+16] = VAL_TO_PIECE.get(pn).unwrap();
        }
        for (idx, pn) in BitPosition::from_bits2(r56).iter().enumerate() {
            pos.board[idx+32] = VAL_TO_PIECE.get(pn).unwrap();
        }
        for (idx, pn) in BitPosition::from_bits2(r78).iter().enumerate() {
            pos.board[idx+48] = VAL_TO_PIECE.get(pn).unwrap();
        }

        Ok(pos)
    }
}
//
//8, 9,10, 11, 12, 10, 9, 8
// 1000 1001 1010 1011 1100 1010 1001 1000
// 10001001101010111100101010011000


// pieces
pub const WHITE_ROOK_C: char   = 'R';
pub const WHITE_KNIGHT_C: char = 'N';
pub const WHITE_BISHOP_C: char = 'B';
pub const WHITE_QUEEN_C: char  = 'Q';
pub const WHITE_KING_C: char   = 'K';
pub const WHITE_PAWN_C: char   = 'P';
pub const BLACK_ROOK_C: char   = 'r';
pub const BLACK_KNIGHT_C: char = 'n';
pub const BLACK_BISHOP_C: char = 'b';
pub const BLACK_QUEEN_C: char  = 'q';
pub const BLACK_KING_C: char   = 'k';
pub const BLACK_PAWN_C: char   = 'p';

pub static WHITE_ROOK: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Rook, Side::White) });
pub static WHITE_KNIGHT: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Knight, Side::White) });
pub static WHITE_BISHOP: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Bishop, Side::White) });
pub static WHITE_QUEEN: Lazy<PieceInPlay>  = Lazy::new( || { PieceInPlay::new(Piece::Queen, Side::White) });
pub static WHITE_KING: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::King, Side::White) });
pub static WHITE_PAWN: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Pawn, Side::White) });
pub static BLACK_ROOK: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Rook, Side::Black) });
pub static BLACK_KNIGHT: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Knight, Side::Black) });
pub static BLACK_BISHOP: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Bishop, Side::Black) });
pub static BLACK_QUEEN: Lazy<PieceInPlay>  = Lazy::new( || { PieceInPlay::new(Piece::Queen, Side::Black) });
pub static BLACK_KING: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::King, Side::Black) });
pub static BLACK_PAWN: Lazy<PieceInPlay>   = Lazy::new( || { PieceInPlay::new(Piece::Pawn, Side::Black) });
pub static EMPTY: Lazy<PieceInPlay> = Lazy::new( || { PieceInPlay::new(Piece::Empty, Side::Black) });

pub static VAL_TO_PIECE: Lazy<HashMap<u8, &Lazy<PieceInPlay>>> = Lazy::new(|| {
    let mut m: HashMap<u8, &Lazy<PieceInPlay>> = HashMap::new();
    let pieces: [&'static Lazy<PieceInPlay>; 13] = [
        &WHITE_ROOK, &WHITE_KNIGHT, &WHITE_BISHOP, &WHITE_QUEEN, &WHITE_KING, &WHITE_PAWN,
        &BLACK_ROOK, &BLACK_KNIGHT, &BLACK_BISHOP, &BLACK_QUEEN, &BLACK_KING, &BLACK_PAWN,
        &EMPTY
    ];
    for p in pieces {
        m.insert( p.value.value(), p);
    }
    m
});

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
