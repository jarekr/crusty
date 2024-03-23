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

// from a given .pgn file, create a 1:n segments, each segment consisting of
// a list of positions + 1 table of games.
// Games will reference positions by segment_id and byte offset
// (or equivalent) within segment.

//

fn create_segments_for_games(games_reader: BufferedReader<File>) -> Vec<PositionSegment> {
    let mut segments = Vec::<PositionSegment>::new();

    let mut pos_id: u64 = 0;
    let visitor = &mut GameVisitor::new();
    for visit in games_reader.into_iter(visitor) {
        // play through each move in the pgn and generate a BitPosition for each position reached
        // TODO handle bad pgn gracefully
        let result = visit.expect("failed to parse pgn");

        // TODO this needs to handle updates gracefully
        let _game_id = Game::insert(&db, &result.game).expect("game insert failed");

        segment.insert(r12, r34, r56, r78);
        pos_id += 1;
    }

    segments
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
