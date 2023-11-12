use clap::Parser;
use std::path::{PathBuf, Path};
use colored::*;

mod db;
use db::Db;

mod parsing;
use parsing::BitPosition;

#[derive(Parser)]
struct Args {
    config_path: PathBuf
}

fn main() {

    let args = Args::parse();
    println!("called with config_path: {}", args.config_path.display().to_string().green());
    let dbpath = Path::new("data.db");
    let db = Db::new(dbpath);
    db.init_schema();
    println!("This is where I am");
    let pos: BitPosition = BitPosition::parse_from_str("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2").unwrap();
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
