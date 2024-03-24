// standard lib
use std::fs::File;
use std::path::Path;

// 3rd party
use pgn_reader::BufferedReader;

// our modules
use crate::persistance;
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
fn games_for_buffs(games_reader: BufferedReader<File>) -> Vec<GameVisitor> {
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

fn game_visitor_to_positions(visitor: GameVisitor) {
    for bitpos in visitor.fens {
        let (r12,r34,r56, r78) = bitpos.to_bits();
    }
}

fn create_readers_for_dir(dir: &Path) -> Result<Vec<BufferedReader<File>>, Box<dyn Error>> {
    let readers = Vec::<BufferedReader<File>>::new();

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
        let reader = BufferedReader::new(pgn.to_path_buf());
        readers.push(reader);
    }
    Ok(readers)
}
