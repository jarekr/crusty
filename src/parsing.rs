use bilge::arbitrary_int::Number;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::HashMap, hash::{DefaultHasher, Hash, Hasher}, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct RustyConfig {
    pub version: u8,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
}

use bilge::prelude::*;

use once_cell::sync::Lazy;

use pgn_reader::{RawHeader, SanPlus, Nag, RawComment, Skip, Visitor};

use shakmaty::{san::Suffix, Chess, Position, Role};

use crate::db::Game;

pub struct GameVisitor {
    pub pos: Chess,
    pub fens: Vec<BitPosition>,
    pub game: Game,
    pub move_count: u8,
    pub side_to_move: Side,
    pub pgn_bytes: Vec::<u8>,
}

impl GameVisitor {
    pub fn new() -> GameVisitor {
        GameVisitor {
            pos: Chess::default(),
            fens: Vec::new(),
            game: Game::new(),
            move_count: 0,
            side_to_move: Side::White,
            pgn_bytes: Vec::new(),
        }
    }
}

impl Visitor for GameVisitor {
    type Result = GameVisitor;

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        self.pgn_bytes.push(OPEN_SQUARE_BRACKET);
        for byte in key {
            self.pgn_bytes.push(*byte);
        }
        self.pgn_bytes.push(SPACE);
        self.pgn_bytes.push(QUOTE);
        for byte in value.as_bytes() {
            self.pgn_bytes.push(*byte);
        }
        self.pgn_bytes.push(QUOTE);
        self.pgn_bytes.push(CLOSE_SQUARE_BRACKET);
        self.pgn_bytes.push(SPACE);

        match std::str::from_utf8(key) {
            Ok(HEADER_EVENT) => self.game.event = value.decode_utf8_lossy().to_string(),
            Ok(HEADER_SITE) => self.game.site = value.decode_utf8_lossy().to_string(),
            Ok(HEADER_DATE) => self.game.date = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_ROUND) => self.game.round = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_WHITE) => self.game.white = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_BLACK) => self.game.black = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_RESULT) => self.game.result = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_CURRENT_POSITION) => {
                self.game.current_position = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_TIMEZONE) => self.game.timezone = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_ECO) => self.game.eco = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_ECO_URL) => self.game.eco_url = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_UTC_DATE) => self.game.utc_date = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_UTC_TIME) => self.game.utc_time = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_WHITE_ELO) => {
                self.game.white_elo = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_BLACK_ELO) => {
                self.game.black_elo = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_TIME_CONTROL) => {
                self.game.time_control = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_TERMINATION) => {
                self.game.termination = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_VARIANT) => self.game.variant = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_START_TIME) => {
                self.game.start_time = Some(value.decode_utf8_lossy().to_string())
            }
            Ok(HEADER_END_TIME) => self.game.end_time = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_LINK) => self.game.link = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_OPENING) => self.game.opening = Some(value.decode_utf8_lossy().to_string()),
            Ok(HEADER_WHITE_RATING_DIFF)=> (),
            Ok(HEADER_BLACK_RATING_DIFF)=> (),
            Ok(HEADER_ANNOTATOR) => (),
            Ok(HEADER_BLACK_TITLE) => (),
            Ok(HEADER_WHITE_TITLE) => (),
            Ok(HEADER_FEN) => (),
            Ok(HEADER_SETUP) => (),
            Ok(HEADER_CHAPTER_MODE) => (),
            Ok(other) => println!("unknown header key: {}", other),
            Err(_why) => println!("Caught error convertying header key to utf8"),
        };
    }

    fn nag(&mut self, nag: Nag) {
        for byte in nag.to_string().as_bytes() {
            self.pgn_bytes.push(*byte);
        }
        self.pgn_bytes.push(SPACE);
    }

    fn comment(&mut self, comment: RawComment<'_>) {
        self.pgn_bytes.push(OPEN_CURLY_BRACE);
        for byte in comment.as_bytes() {
            self.pgn_bytes.push(*byte);
        }
        self.pgn_bytes.push(CLOSE_CURLY_BRACE);
        self.pgn_bytes.push(SPACE);
    }

    fn san(&mut self, san_plus: SanPlus) {
        let mut postfix: &str;
        if self.side_to_move == Side::White {
            self.move_count += 1;
            postfix = ". ";
            self.side_to_move = Side::Black;
        } else {
            postfix = "... ";
            self.side_to_move = Side::White;
        }

        for byte in format!("{}{}", self.move_count, postfix).as_bytes() {
            self.pgn_bytes.push(*byte);
        }

        for byte in san_plus.san.to_string().as_bytes() {
            self.pgn_bytes.push(*byte);
        }

        if san_plus.suffix.is_some() {
            if san_plus.suffix.unwrap() == Suffix::Check {
                self.pgn_bytes.push(PLUS);
            } else {
                self.pgn_bytes.push(POUND);
            }
        }

        self.pgn_bytes.push(SPACE);

        if let Ok(m) = san_plus.san.to_move(&self.pos) {
            self.pos.play_unchecked(&m);
        }
        let mut biter = self.pos.board().clone().into_iter();

        let mut bp = BitPosition::new();

        for (square, piece) in biter.by_ref() {
            let isblack = piece.color.is_black();
            let i: usize = square.into();
            bp.board[i] = match piece.role {
                Role::Bishop => {
                    if isblack {
                        &BLACK_BISHOP
                    } else {
                        &WHITE_BISHOP
                    }
                }
                Role::Knight => {
                    if isblack {
                        &BLACK_KNIGHT
                    } else {
                        &WHITE_KNIGHT
                    }
                }
                Role::Rook => {
                    if isblack {
                        &BLACK_ROOK
                    } else {
                        &WHITE_ROOK
                    }
                }
                Role::King => {
                    if isblack {
                        &BLACK_KING
                    } else {
                        &WHITE_KING
                    }
                }
                Role::Queen => {
                    if isblack {
                        &BLACK_QUEEN
                    } else {
                        &WHITE_QUEEN
                    }
                }
                Role::Pawn => {
                    if isblack {
                        &BLACK_PAWN
                    } else {
                        &WHITE_PAWN
                    }
                }
            };
        }
        self.fens.push(bp);
    }

    fn end_game(&mut self) -> Self::Result {
        match String::from_utf8(std::mem::take(self.pgn_bytes.as_mut())) {
            Ok(pgnstring) => {
                let mut hasher = DefaultHasher::new();
                self.game.pgn = Some(pgnstring);
                self.game.pgn.hash(&mut hasher);
                self.game.hash = hasher.finish() as i64;
            },
            Err(e) => println!("Error convering pgn_bytes to str: {}", e),
        };
        std::mem::replace(self, GameVisitor::new())
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
pub enum BitPiece {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
    #[fallback]
    #[default]
    Empty,
}

#[bitsize(1)]
#[derive(Debug, PartialEq, FromBits, Default, Clone)]
pub enum Side {
    White,
    #[default]
    Black,
}

#[bitsize(4)]
#[derive(DebugBits, PartialEq, FromBits, Default, Clone)]
pub struct PieceInPlay {
    pub piece: BitPiece,
    pub side: Side,
}

impl PieceInPlay {
    pub fn to_char(&self) -> char {
        let c = match self.piece() {
            BitPiece::Pawn => WHITE_PAWN_C,
            BitPiece::Rook => WHITE_ROOK_C,
            BitPiece::Bishop => WHITE_BISHOP_C,
            BitPiece::King => WHITE_KING_C,
            BitPiece::Queen => WHITE_QUEEN_C,
            BitPiece::Empty => '_',
            BitPiece::Knight => WHITE_KNIGHT_C,
        };
        match self.side() {
            Side::Black => c.to_lowercase().next().unwrap(),
            Side::White => c,
        }
    }
}

#[derive(Debug)]
pub struct BitPosition {
    pub board: [&'static Lazy<PieceInPlay>; 64],
}

impl BitPosition {
    pub fn new() -> BitPosition {
        BitPosition {
            board: [
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY, &EMPTY,
                &EMPTY, &EMPTY, &EMPTY, &EMPTY,
            ],
        }
    }

    pub fn parse_from_str(fen: &str) -> Result<BitPosition, &str> {
        let parts: Vec<_> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err("not enough parts");
        }
        let position_parts: Vec<_> = parts[0].split('/').collect();
        if position_parts.len() != 8 {
            return Err("not enough rows");
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
                        idx += 1;
                    }
                } else {
                    pos.board[idx] = match pc {
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

    pub fn to_bits(&self) -> (u64, u64, u64, u64) {
        let mut r12: u64 = 0;
        let mut r34: u64 = 0;
        let mut r56: u64 = 0;
        let mut r78: u64 = 0;
        let mut shiftamt = 0;
        for (idx, &sq) in self.board.iter().enumerate() {
            let bob = sq.value.value();
            if idx < 16 {
                r12 += (bob as u64) << shiftamt;
            } else if idx < 32 {
                r34 += (bob as u64) << shiftamt - 64;
            } else if idx < 48 {
                r56 += (bob as u64) << shiftamt - 128;
            } else {
                r78 += (bob as u64) << shiftamt - 192;
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
            pos.board[idx + 16] = VAL_TO_PIECE.get(pn).unwrap();
        }
        for (idx, pn) in BitPosition::from_bits2(r56).iter().enumerate() {
            pos.board[idx + 32] = VAL_TO_PIECE.get(pn).unwrap();
        }
        for (idx, pn) in BitPosition::from_bits2(r78).iter().enumerate() {
            pos.board[idx + 48] = VAL_TO_PIECE.get(pn).unwrap();
        }

        Ok(pos)
    }
}
//
//8, 9,10, 11, 12, 10, 9, 8
// 1000 1001 1010 1011 1100 1010 1001 1000
// 10001001101010111100101010011000

// pieces
pub const WHITE_ROOK_C: char = 'R';
pub const WHITE_KNIGHT_C: char = 'N';
pub const WHITE_BISHOP_C: char = 'B';
pub const WHITE_QUEEN_C: char = 'Q';
pub const WHITE_KING_C: char = 'K';
pub const WHITE_PAWN_C: char = 'P';
pub const BLACK_ROOK_C: char = 'r';
pub const BLACK_KNIGHT_C: char = 'n';
pub const BLACK_BISHOP_C: char = 'b';
pub const BLACK_QUEEN_C: char = 'q';
pub const BLACK_KING_C: char = 'k';
pub const BLACK_PAWN_C: char = 'p';

//pub const HEADER_
pub const HEADER_EVENT: &str = "Event";
pub const HEADER_SITE: &str = "Site";
pub const HEADER_DATE: &str = "Date";
pub const HEADER_ROUND: &str = "Round";
pub const HEADER_WHITE: &str = "White";
pub const HEADER_BLACK: &str = "Black";
pub const HEADER_RESULT: &str = "Result";
pub const HEADER_CURRENT_POSITION: &str = "CurrentPosition";
pub const HEADER_TIMEZONE: &str = "Timezone";
pub const HEADER_ECO: &str = "ECO";
pub const HEADER_ECO_URL: &str = "ECOUrl";
pub const HEADER_UTC_DATE: &str = "UTCDate";
pub const HEADER_UTC_TIME: &str = "UTCTime";
pub const HEADER_WHITE_ELO: &str = "WhiteElo";
pub const HEADER_BLACK_ELO: &str = "BlackElo";
pub const HEADER_TIME_CONTROL: &str = "TimeControl";
pub const HEADER_TERMINATION: &str = "Termination";
pub const HEADER_VARIANT: &str = "Variant";
pub const HEADER_START_TIME: &str = "StartTime";
pub const HEADER_END_TIME: &str = "EndTime";
pub const HEADER_LINK: &str = "Link";
pub const HEADER_OPENING: &str = "Opening";
// headers unique to lichess.org
pub const HEADER_WHITE_RATING_DIFF: &str = "WhiteRatingDiff";
pub const HEADER_BLACK_RATING_DIFF: &str = "BlackRatingDiff";
pub const HEADER_ANNOTATOR: &str = "Annotator";
pub const HEADER_WHITE_TITLE: &str = "WhiteTitle";
pub const HEADER_BLACK_TITLE: &str = "BlackTitle";
pub const HEADER_FEN: &str = "FEN";
pub const HEADER_SETUP: &str = "SetUp";
pub const HEADER_CHAPTER_MODE: &str = "ChapterMode";

pub static WHITE_ROOK: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Rook, Side::White));
pub static WHITE_KNIGHT: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Knight, Side::White));
pub static WHITE_BISHOP: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Bishop, Side::White));
pub static WHITE_QUEEN: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Queen, Side::White));
pub static WHITE_KING: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::King, Side::White));
pub static WHITE_PAWN: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Pawn, Side::White));
pub static BLACK_ROOK: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Rook, Side::Black));
pub static BLACK_KNIGHT: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Knight, Side::Black));
pub static BLACK_BISHOP: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Bishop, Side::Black));
pub static BLACK_QUEEN: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Queen, Side::Black));
pub static BLACK_KING: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::King, Side::Black));
pub static BLACK_PAWN: Lazy<PieceInPlay> =
    Lazy::new(|| PieceInPlay::new(BitPiece::Pawn, Side::Black));
pub static EMPTY: Lazy<PieceInPlay> = Lazy::new(|| PieceInPlay::new(BitPiece::Empty, Side::Black));

pub static OPEN_SQUARE_BRACKET: u8 = 91;
pub static CLOSE_SQUARE_BRACKET: u8 = 93;
pub static OPEN_CURLY_BRACE: u8 = 123;
pub static CLOSE_CURLY_BRACE: u8 = 125;
pub static SPACE: u8 = 32;
pub static QUOTE: u8 = 34;
pub static PLUS: u8 = 43;
pub static POUND: u8 = 35;

pub static VAL_TO_PIECE: Lazy<HashMap<u8, &Lazy<PieceInPlay>>> = Lazy::new(|| {
    let mut m: HashMap<u8, &Lazy<PieceInPlay>> = HashMap::new();
    let pieces: [&'static Lazy<PieceInPlay>; 13] = [
        &WHITE_ROOK,
        &WHITE_KNIGHT,
        &WHITE_BISHOP,
        &WHITE_QUEEN,
        &WHITE_KING,
        &WHITE_PAWN,
        &BLACK_ROOK,
        &BLACK_KNIGHT,
        &BLACK_BISHOP,
        &BLACK_QUEEN,
        &BLACK_KING,
        &BLACK_PAWN,
        &EMPTY,
    ];
    for p in pieces {
        m.insert(p.value.value(), p);
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
