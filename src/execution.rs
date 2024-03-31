use std::fs::File;
// standard lib
use std::path::{Path, PathBuf};
use std::error::Error;

// 3rd party
use pgn_reader::BufferedReader;

// our modules
use crate::persistance::{self, Position};
use persistance::PositionSegment;

use crate::parsing;
use parsing::{BitPosition, GameVisitor};

use crate::db;
use db::{Db, Game};

// from a given .pgn file, create a 1:n segments, each segment consisting of
// a list of positions + 1 table of games.
// Games will reference positions by segment_id and byte offset
// (or equivalent) within segment.

//
pub fn games_for_buffs(games_reader: BufferedReader<File>) -> Vec<GameVisitor> {
    let mut games = Vec::<GameVisitor>::new();

    let mut pos_id: u64 = 0;
    let visitor = &mut GameVisitor::new();
    for visit in games_reader.into_iter(visitor) {
        // play through each move in the pgn and generate a BitPosition for each position reached
        // TODO handle bad pgn gracefully
        let result = visit.expect("failed to parse pgn");
        games.push(result);
    }
    games
}

pub fn game_visitor_to_positions(visitor: GameVisitor) {
    for bitpos in visitor.fens {
        let (r12,r34,r56, r78) = bitpos.to_bits();
        let pos = Position {
            r12,
            r34,
            r56,
            r78,
        };
    }
}

pub fn create_readers_for_dir(dir: &Path) -> Result<Vec<BufferedReader<File>>, Box<dyn Error>> {
    let mut readers = Vec::<BufferedReader<File>>::new();

    /*
    let fh: File = match File::open(dir) {
        Ok(myfile) => myfile,
        Err(e) => return Err(e),
    };
    */
    let pgns = std::fs::read_dir(dir)?
        .filter_map(|res| res.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            if path.extension().map_or(false, |ext| ext == "pgn") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for pgn in pgns {
        let reader = BufferedReader::new(std::fs::File::open(pgn.as_path()).expect("Failed to open pgn file"));
        readers.push(reader);
    }
    Ok(readers)
}
