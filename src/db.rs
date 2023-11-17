use const_format::concatcp;
use rusqlite::{named_params, Connection, OpenFlags, Error};
use std::{path::Path, vec, error::Error};


const GAMES_TABLE: &str = "games";
const GAMES_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    GAMES_TABLE,
    " (
            id       INTEGER PRIMARY KEY,
            pgn      TEXT,
            notes    TEXT
        )"
);
const GET_BY_ID_GAMES_SQL: &str = concatcp!(
    "SELECT * FROM ",
    GAMES_TABLE,
    " where id = :id order by id asc"
);
const INSERT_INTO_GAMES_SQL: &str = concatcp!(
    "INSERT INTO ",
    GAMES_TABLE,
    " (pgn, notes) VALUES (:pgn, :notes)"
);

const POSITIONS_TABLE: &str = "positions";

const POSITIONS_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    POSITIONS_TABLE,
    " (
            id       INTEGER PRIMARY KEY,
            r12      INTEGER,
            r34      INTEGER,
            r56      INTEGER,
            r78      INTEGER
      )"
);

const INSERT_INTO_POSITIONS_SQL: &str = concatcp!(
    "INSERT INTO ",
    POSITIONS_TABLE,
    " (r12, r34, r56, r78) VALUES (:r12, :r34, :r56, :r78)"
);

const GET_ALL_POSITIONS_SQL: &str = concatcp!(
    "SELECT id, r12, r34, r56, r78 FROM ",
    POSITIONS_TABLE,
    " ORDER BY id"
);

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn new(dbpath: &Path) -> Db {
        match Connection::open_with_flags(
            dbpath,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(conn) => Db { conn },
            Err(why) => panic!("{}", why),
        }
    }

    pub fn init_schema(&self) -> () {
        for sql in [GAMES_DDSQL, POSITIONS_DDSQL] {
            self.create_schema(sql);
        }
    }

    fn create_schema(&self, sql: &str) -> usize {
        match self.conn.execute(sql, ()) {
            Ok(res) => res,
            Err(why) => panic!("schema create failed: {}", why),
        }
    }
}

pub struct Game {
    pub id: u32,
    pub pgn: String,
    pub notes: String,
}

impl Game {
    pub fn query_by_id(db: &Db, id: u32) -> Option<Game> {
        let mut stmt = db
            .conn
            .prepare(GET_BY_ID_GAMES_SQL)
            .expect("prepare failed");
        let mut row_iter = stmt.query(named_params! {":id": id.to_string()}).unwrap();

        match row_iter.next().expect("next failed") {
            Some(row) => Some(Game {
                id: row.get(0).unwrap(),
                pgn: row.get(1).unwrap(),
                notes: row.get(3).unwrap(),
            }),
            None => None,
        }
    }
}

pub struct Position {
    pub id: u32,
    pub r12: u64,
    pub r34: u64,
    pub r56: u64,
    pub r78: u64,
}

const MAX_SQLITE_INT: u64 = 2u64.pow(63) - 1;
impl Position {

    //fn convert_to(x: BitArray(uint=x, length=64).int if x > MAX_SQLITE_INT else x
    //        convert_hash_from = lambda x: BitArray(int=x, length=64).uint if x < 0 else x

    pub fn insert(db: &Db, r12: u64, r34: u64, r56: u64, r78: u64) -> Result<usize, Error> {
        let mut stmt = db
            .conn
            .prepare(INSERT_INTO_POSITIONS_SQL)
            .expect("prepare failed");

        stmt.execute(named_params! {":r12": r12 as i64, ":r34": r34 as i64, ":r56": r56 as i64, ":r78": r78 as i64 })
    }
    pub fn get_all(db: &Db) -> Result<Vec<(u64, u64, u64, u64)>, Error> {
        let mut stmt = db
        .conn
        .prepare(GET_ALL_POSITIONS_SQL)
        .expect("failed to prepare get_all_positions_sql");

        stmt.query_map([], |row|{
            Ok()
        }
    }
}
