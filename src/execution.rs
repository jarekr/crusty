
// standard lib
use std::fs::File;
use std::path::Path;

// 3rd party
use pgn_reader::BufferedReader;

// our modules
use crate::persistance;
use persistance::PositionSegment;

use crate::parsing;



// from a given .pgn file, create a 1:n segments, each segment consisting of
// a list of positions + 1 table of games.
// Games will reference positions by segment_id and byte offset
// (or equivalent) within segment.

//
fn create_segments_for_games(_games_reader: BufferedReader<File>) -> Vec<PositionSegment> {
    let segments = Vec::<PositionSegment>::new();

    segments
}


fn create_readers_for_dir(_path: &Path) -> Vec<BufferedReader<File>> {
    let readers = Vec::<BufferedReader<File>>::new();

    readers
}