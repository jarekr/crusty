use clap::Parser;
use std::path::{PathBuf, Path};
use colored::*;

mod db;
use db::Db;

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

    println!("This is where I am")
}
