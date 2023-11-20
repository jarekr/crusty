use clap::Parser;
use colored::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
mod db;
use db::{Db, Game, GamePosition, Position};

use std::time::{Duration, Instant};

use std::collections::{HashMap, HashSet};

mod parsing;
use parsing::{BitPosition, GameVisitor};

#[derive(Parser)]
struct Args {
    config_path: PathBuf,
}

use pgn_reader::BufferedReader;

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
    let iter = reader.into_iter(visitor);

    let mut game_count: u64 = 0;
    let mut pos_id: u64 = 0;
    let mut position_ids = Vec::new();

    let mut r12hm: HashMap<u64, u64> = HashMap::new();
    let mut r34hm: HashMap<u64, u64> = HashMap::new();
    let mut r56hm: HashMap<u64, u64> = HashMap::new();
    let mut r78hm: HashMap<u64, u64> = HashMap::new();

    let mut positions: HashSet<(u64, u64, u64, u64)> = HashSet::new();

    let start_time = Instant::now();

    for foo in iter {
        game_count += 1;
        let result = foo.expect("failed to parse pgn");

        //let game_id = Game::insert(&db, &result.game).expect("game insert failed");

        for gv in result.fens {
            let (r12, r34, r56, r78) = gv.to_bits();
            //print_pos(&bp);
            //let pos_id = match Position::insert(&db, r12, r34, r56, r78) {
            //    Err(why) => panic!("failed to insert: {}", why),
            //    Ok(id) => id,
            //};
            position_ids.push(pos_id);

            let r12id = match r12hm.insert(r12, pos_id) {
                Some(id) => id,
                None => pos_id,
            };
            let r34id = match r34hm.insert(r34, pos_id) {
                Some(id) => id,
                None => pos_id,
            };
            let r56id = match r56hm.insert(r56, pos_id) {
                Some(id) => id,
                None => pos_id,
            };
            let r78id = match r78hm.insert(r78, pos_id) {
                Some(id) => id,
                None => pos_id,
            };
            positions.insert((r12id, r34id, r56id, r78id));
            pos_id += 1;
        }

        //GamePosition::insert(&db, game_id, position_ids).expect("failed to inser game positions");

        if game_count % 1000 == 0 {
            let duration = start_time.elapsed().as_secs_f64();
            let games_per_sec = game_count as f64 / duration;
            println!(
                "games {: >6}\n  positions parsed {}\n    duration {: >6.2} sec, {:.2} games/s\n    positions {}\n    r12={}\n    r34={}\n    r56={}\n    r78={}\n",
                game_count, position_ids.len(), duration, games_per_sec, positions.len(), r12hm.len(), r34hm.len(), r56hm.len(), r78hm.len());
        }
    }

    let mut input = String::new();
    println!("-- stats --");
    let duration = start_time.elapsed().as_secs_f64();
    let games_per_sec = game_count as f64 / duration;
    println!(
        "games {: >6}\n  positions parsed {}\n    duration {: >6} sec, {: >.2} games/s\n    positions {}\n    r12={}\n    r34={}\n    r56={}\n    r78={}\n",
        game_count, position_ids.len(), duration, games_per_sec, positions.len(), r12hm.len(), r34hm.len(), r56hm.len(), r78hm.len());

    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");
    println!("You inputttedddd: {}", input);

    if args.config_path.ends_with("nevergoingtohappen") {
        bob();
    }
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
        let piece: char = sq.to_char();

        print!("{}", piece);
        if counter % 8 == 0 {
            println!();
        }
        counter += 1;
    }
}
