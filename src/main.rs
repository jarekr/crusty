use clap::Parser;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
mod db;
use db::{Db, Position};

mod parsing;
use parsing::{BitPosition, FenVisitor};

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
    let contents = fs::read_to_string(args.config_path).expect("foo");
    let mut reader = BufferedReader::new_cursor(contents.into_bytes());

    let mut fenv = FenVisitor::new();
    let result = reader
        .read_game(&mut fenv)
        .expect("parsing game failed")
        .unwrap();

    let dbpath = Path::new("data.db");
    let db = Db::new(dbpath);
    db.init_schema();
    for bp in result {
        let (r12, r34, r56, r78) = bp.to_bits();
        //print_pos(&bp);
        match Position::insert(&db, r12, r34, r56, r78) {
            Err(why) => panic!("failed to insert: {}", why),
            Ok(_) => ()
        };
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
