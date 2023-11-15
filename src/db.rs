use const_format::concatcp;
use rusqlite::{named_params, Connection, OpenFlags};
use std::path::Path;

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
const GET_BY_ID_GAMES_QSQL: &str = concatcp!(
    "SELECT * FROM ",
    GAMES_TABLE,
    " where id = :id order by id asc"
);
const INSERT_INTO_GAMES_QSQL: &str = concatcp!(
    "INSERT INTO ",
    GAMES_TABLE,
    " (id, pgn, notes) VALUES (:id, :pgn, :notes)"
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
        for sql in [GAMES_DDSQL] {
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
    pub fn query_by_id(db: Db, id: u32) -> Option<Game> {
        let mut stmt = db
            .conn
            .prepare(GET_BY_ID_GAMES_QSQL)
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
