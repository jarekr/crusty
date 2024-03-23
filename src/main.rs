// "standard library"
use std::fs;

use std::path::{Path, PathBuf};
use std::time::{Instant};

use std::io::prelude::*;

use std::collections::btree_map::BTreeMap;

// third party modules
use clap::Parser;
use colored::*;
use pgn_reader::BufferedReader;

/*
Import our modules here
 */
// DB module
mod db;
use db::{Db, Game};

// Persistance module
mod persistance;
use persistance::{PositionSegment, PositionTrie};

// Parsing module
mod parsing;
use parsing::{BitPosition, GameVisitor};


// Execution module?

mod execution;

#[derive(Parser)]
struct Args {
    config_path: PathBuf,
}
/*
   Tool to read in a pgn file(s) and process the games within
   Processing includes:

   - saving games to a database
     - Allow searching by game attributes such as dates, player names, platform,
     - store PGN in full text as well as some often-used parsed-out attributes
     - experimentally, store a list of positions, which is really a list of
     "pointers to positions" plus some extra meta-data for that position that is
     game-specific.

    - maintain a large, distributed table which can be indexed into useing "pointers to positions".
    For each position found within each game, genereate a "pointer to position" for this position and
         a) save it to the game db
         b) store the position / pointer relationship somehow in this table

    Given the position is a 256-bit / 32 byte "number", "pointer to position"
    should be something substantially smaller.

    Candidates are
        a) i64 - easily maps to SQLite integer column type
        b) u32 - 1/8th of a position, seams like a reasonable hash size

    DS candidates:
    a) trie
    b) multi-level hashmap
    c) arrays?

    parser front-end:
    cli which accepts a pgn file or a directory of pgn files to process
*/

fn parse_in_parallel() {

    // given a list of directories, generate a list of .pgn files
    //
    // order the .pgn files by size, and given N cores, create N segments and map
    // roughly equally-sized amounts (bytes per game or something) to each segment.
    //
    // create N threads, within each thread run an executor over one of the batches
    //  of .pgn files generated previosly
    //
    // await all executors
    //
    // upon executor failure, error; note previous game within file as last-known-good
    // for that file, mark file as to-be-cleaned
    // save segment to dirty file mapping in dirties table
    // move on to next file in list
    //
    // write out segments
    //
    // check status of segments
    //
    // merge segments, remapping segment_id/offsets in affected tables
    //   and merging those as well
}

fn main() {
    let args = Args::parse();
    println!(
        "called with arg : {}",
        args.config_path.display().to_string().green()
    );

    let dbpath = Path::new("data.db");
    let db = Db::new(dbpath);
    db.init_schema();

    let pgn_file = fs::File::open(&args.config_path).expect("failed to open file");
    let reader = BufferedReader::new(pgn_file);

    let visitor = &mut GameVisitor::new();

    let mut game_count: u64 = 1;
    let mut pos_id: u64 = 0;
    //let mut position_ids = Vec::new();

    let _million = 10usize.pow(6);
    let _thousand = 10usize.pow(3);
    //let million = 10usize;
    let _r12hm: BTreeMap<u64, u64> = BTreeMap::new();
    let _r34hm: BTreeMap<u64, u64> = BTreeMap::new();
    let _r56hm: BTreeMap<u64, u64> = BTreeMap::new();
    let _r78hm: BTreeMap<u64, u64> = BTreeMap::new();
    let _r12id_latest: u64 = 1;
    let _r34id_latest: u64 = 1;
    let _r56id_latest: u64 = 1;
    let _r78id_latest: u64 = 1;

    //let mut positions: HashSet<(u64, u64, u64, u64)> = HashSet::with_capacity(10 * thousand);
    //let mut positions2: HashSet<(u64, u64, u64, u64)> = HashSet::with_capacity(100 * million);
    let mut positions_parsed = 0;

    let start_time = Instant::now();

    let ptrie = PositionTrie::new();

    let mut segment = PositionSegment::new("segment1.db");

    for visit in reader.into_iter(visitor) {
        // play through each move in the pgn and generate a BitPosition for each position reached
        // TODO handle bad pgn gracefully
        let result = visit.expect("failed to parse pgn");

        // TODO this needs to handle updates gracefully
        let _game_id = Game::insert(&db, &result.game).expect("game insert failed");

        // proces the resulting fens
        // TODO do this in parallel
        for gv in result.fens {
            let (r12, r34, r56, r78) = gv.to_bits();
            positions_parsed += 1;
            //TODO: using trie or some other ds, ensure that segment list is kept in order (before writing it out?)
            //
            //let res = ptrie.insert(&PositionSegment::calculate_position_tree_address(r12, r34, r56, r78));
            //print_pos(&bp);
            //let pos_id = match Position::insert(&db, r12, r34, r56, r78) {
            //    Err(why) => panic!("failed to insert: {}", why),
            //    Ok(id) => id,
            //};
            //position_ids.push(pos_id);

            //let r12id = match r12hm.get(&r12) {
            //    Some(id) => id,
            //    None => { r12hm.insert(r12, r12id_latest); r12id_latest += 1; r12hm.get(&r12).unwrap() },
            //};
            /*
            let r12id: u64 = *r12hm.entry(r12).or_insert(r12id_latest);
            if r12id == r12id_latest { r12id_latest += 1; }
            let r34id: u64 = *r34hm.entry(r34).or_insert(r34id_latest);
            if r34id == r34id_latest { r34id_latest += 1; }
            let r56id: u64 = *r56hm.entry(r56).or_insert(r56id_latest);
            if r56id == r56id_latest { r56id_latest += 1; }
            let r78id: u64 = *r78hm.entry(r78).or_insert(r78id_latest);
            if r78id == r78id_latest { r78id_latest += 1; }
            */


            //println!("Inserting position {} {} {} {} ({}, {}, {}, {})", *r12id, *r34id, *r56id, *r78id, r12, r34, r56, r78);
            //positions.insert((r12id, r34id, r56id, r78id));
            segment.insert(r12, r34, r56, r78);
            pos_id += 1;
        }

        // inserting positions into (sqlite) db proved extremely slow
        // plan now is to save small-ish metadata in db and game positions in some distributed blob store.
        // What we really need is not to store the positions, but to have a way to derive the position from the position reference
        //  with minimal effort / in minimal time
        //
        //GamePosition::insert(&db, game_id, position_ids).expect("failed to inser game positions");

        if game_count % 1000 == 0 {
            let duration = start_time.elapsed().as_secs_f64();
            let games_per_sec = game_count as f64 / duration;

            // println!(
            //     "games {: >6}\n  positions parsed {}\n    duration {: >6.2} sec, {:.2} games/s\n    positions {}\n    r12={}\n    r34={}\n    r56={}\n    r78={}\n",
            //     game_count, position_ids.len(), duration, games_per_sec, positions.len(), r12hm.len(), r34hm.len(), r56hm.len(), r78hm.len());
            println!(
                "games {: >6}\n  positions parsed {}\n    duration {: >6.2} sec, {:.2} games/s\n    positions {}\n\n",
                 game_count, positions_parsed, duration, games_per_sec, positions_parsed);
        }

        game_count += 1;
        if game_count > 2_000_000 {
            break;
        }
    }

    let _input = String::new();
    println!("-- stats --");
    let duration = start_time.elapsed().as_secs_f64();
    let games_per_sec = (game_count - 1) as f64 / duration;
    println!(
        "games {: >6}\n  positions parsed {}\n    duration {: >6.2} sec, {:.2} games/s\n    positions {}",
        game_count - 1, positions_parsed, duration, games_per_sec, positions_parsed);

    println!("--- ptrie ---");
    ptrie.statt();
    println!("writing segment file, {} positions", segment.len());
    segment.write();
    println!("writing segment file");
    /*
    println!("writing r12 file");
    let mut r12file = File::create("r12.db").expect("failed to create r12db");
    for (count, value) in r12hm.keys().enumerate() {
        r12file.write_all(&value.to_be_bytes()).expect("Write failed!");
    }
    drop(r12file);

    println!("writing r34 file");
    let mut r34file = File::create("r34.db").expect("failed to create r34db");
    for (count, value) in r34hm.keys().enumerate() {
        r34file.write_all(&value.to_be_bytes()).expect("Write failed!");
    }
    drop(r34file);

    println!("writing r56 file");
    let mut r56file = File::create("r56.db").expect("failed to create r56db");
    for (count, value) in r56hm.keys().enumerate() {
        r56file.write_all(&value.to_be_bytes()).expect("Write failed!");
    }
    drop(r56file);

    println!("writing r78 file");
    let mut r78file = File::create("r78.db").expect("failed to create r78db");
    for (count, value) in r78hm.keys().enumerate() {
        r78file.write_all(&value.to_be_bytes()).expect("Write failed!");
    }
    drop(r78file);

    println!("press enter to continue....");
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    println!("You inputttedddd: {}", input);

    if args.config_path.ends_with("nevergoingtohappen") {
        bob();
    }
    */
}

fn bob() {
    let args = Args::parse();
    println!(
        "called with config_path: {}",
        args.config_path.display().to_string().green()
    );
    let dbpath = Path::new("data.db");
    let db = Db::new(dbpath);
    db.init_schema();
    println!("This is where I am");
    let pos: BitPosition = BitPosition::parse_from_str(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
    )
    .unwrap();
    print_pos(&pos);
    let (r1, r2, r3, r4) = pos.to_bits();
    dbg!(r1, r2, r3, r4);
    let newpos = BitPosition::from_bits(r1, r2, r3, r4).unwrap();
    print_pos(&newpos);
}

fn print_pos(p: &BitPosition) {
    let mut counter = 1;
    for &sq in p.board.iter() {
        print!("{}", sq.to_char());
        if counter % 8 == 0 {
            println!();
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_position_trie_Address() {
        let pt_add = PositionTrieAddress {
            value: [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        };
    }

    #[test]
    fn test_create_trie() {

        let mut ptree = PositionTrie::new();
        let pt_add = PositionSegment::calculate_position_tree_address(1, 2, 3, 4);
        let res = ptree.insert(&pt_add);
        // new position should not have any level matches
        println!("insert pt_add, res {}", res);
        assert!(res == 0);

        // inserting the same position results in hits to all levels (e.g. 15)
        let res2 = ptree.insert(&pt_add);
        println!("insert pt_add, res2 {}", res2);
        assert!(res2 == 15);

        // changing the position somewhat will result in a level between 0 and 15, exclusive
        let pt2_add = PositionSegment::calculate_position_tree_address(1, 7, 8, 9);
        let res3 = ptree.insert(&pt2_add);
        println!("insert pt2_add, res3 {}", res3);
        assert!(res3 == 7);

        ptree.statt();

    }
}