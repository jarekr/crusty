use const_format::concatcp;
use rusqlite::{named_params, Connection, Error, OpenFlags, Transaction};
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

const R12_TABLE: &str = "R12";
const R12_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    R12_TABLE,
    " ( id INTEGER PRIMARY KEY, row INTEGER UNIQUE NOT NULL)"
);

const R34_TABLE: &str = "R34";
const R34_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    R34_TABLE,
    " ( id INTEGER PRIMARY KEY, row INTEGER UNIQUE NOT NULL)"
);

const R56_TABLE: &str = "R56";
const R56_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    R56_TABLE,
    " ( id INTEGER PRIMARY KEY, row INTEGER UNIQUE NOT NULL)"
);

const R78_TABLE: &str = "R78";
const R78_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    R78_TABLE,
    " ( id INTEGER PRIMARY KEY, row INTEGER UNIQUE NOT NULL)"
);

const INSERT_INTO_R12_SQL: &str = concatcp!("INSERT INTO ", R12_TABLE, " ( row ) VALUES (:row)");
const R12_GET_ID: &str = concatcp!("SELECT id FROM ", R12_TABLE, " WHERE row = :row");
const INSERT_INTO_R34_SQL: &str = concatcp!("INSERT INTO ", R34_TABLE, " ( row ) VALUES (:row)");
const R34_GET_ID: &str = concatcp!("SELECT id FROM ", R34_TABLE, " WHERE row = :row");
const INSERT_INTO_R56_SQL: &str = concatcp!("INSERT INTO ", R56_TABLE, " ( row ) VALUES (:row)");
const R56_GET_ID: &str = concatcp!("SELECT id FROM ", R56_TABLE, " WHERE row = :row");
const INSERT_INTO_R78_SQL: &str = concatcp!("INSERT INTO ", R78_TABLE, " ( row ) VALUES (:row)");
const R78_GET_ID: &str = concatcp!("SELECT id FROM ", R78_TABLE, " WHERE row = :row");

const POSITIONS_TABLE: &str = "positions";
const POSITIONS_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    POSITIONS_TABLE,
    " (
            id       INTEGER PRIMARY KEY,
            r12id    INTEGER,
            r34id    INTEGER,
            r56id    INTEGER,
            r78id    INTEGER,
            CONSTRAINT uniq_pos UNIQUE (r12id, r34id, r56id, r78id),
            FOREIGN KEY(r12id) REFERENCES R12(id),
            FOREIGN KEY(r34id) REFERENCES R34(id),
            FOREIGN KEY(r56id) REFERENCES R56(id),
            FOREIGN KEY(r78id) REFERENCES R78(id)
      )"
);

const INSERT_INTO_POSITIONS_SQL: &str = concatcp!(
    "INSERT INTO ",
    POSITIONS_TABLE,
    " (r12id, r34id, r56id, r78id) VALUES (:r12id, :r34id, :r56id, :r78id)"
);

const GET_POS_FOR_IDS_SQL: &str = concatcp!(
    "SELECT id FROM ",
    POSITIONS_TABLE,
    " WHERE r12id = :r12id AND r34id = :r34id AND r56id = :r56id AND r78id = :r78id"
);

const GET_ALL_POSITIONS_SQL: &str = concatcp!(
    "SELECT id, r12id, r34id, r56id, r78id FROM ",
    POSITIONS_TABLE
);

pub struct Db<'a> {
    path: &'a Path,
}

impl Db<'_> {
    pub fn new(dbpath: &Path) -> Db {
        Db { path: dbpath }
    }

    fn connect(&self) -> Connection {
        match Connection::open_with_flags(
            self.path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(conn) => conn,
            Err(why) => panic!("{}", why),
        }
    }

    pub fn init_schema(&self) -> () {
        let conn = self.connect();
        for sql in [
            GAMES_DDSQL,
            R12_DDSQL,
            R34_DDSQL,
            R56_DDSQL,
            R78_DDSQL,
            POSITIONS_DDSQL,
        ] {
            self.create_schema(&conn, sql);
        }
    }

    fn create_schema(&self, conn: &Connection, sql: &str) -> usize {
        match conn.execute(sql, ()) {
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
        let conn = db.connect();
        let mut stmt = conn.prepare(GET_BY_ID_GAMES_SQL).expect("prepare failed");
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

impl Position {
    pub fn insert(db: &Db, r12: u64, r34: u64, r56: u64, r78: u64) -> Result<i64, Error> {
        let mut conn = db.connect();
        let trans = conn.transaction().expect("error starting transaction");

        let get_id = |insert_sql: &str, get_sql: &str, v: u64| -> i64 {
            match trans
                .prepare(insert_sql)
                .expect("prepare r12 insert failed")
                .execute(named_params! {":row": v as i64 })
            {
                Ok(_) => trans.last_insert_rowid(),
                Err(_) => trans
                    .prepare(get_sql)
                    .expect("prepare failed")
                    .query_row([v as i64], |row| row.get(0))
                    .expect("query failed"),
            }
        };

        let r12id = get_id(INSERT_INTO_R12_SQL, R12_GET_ID, r12);
        let r34id = get_id(INSERT_INTO_R34_SQL, R34_GET_ID, r34);
        let r56id = get_id(INSERT_INTO_R56_SQL, R56_GET_ID, r56);
        let r78id = get_id(INSERT_INTO_R78_SQL, R78_GET_ID, r78);

        let pos_id = match trans
            .prepare(INSERT_INTO_POSITIONS_SQL)
            .expect("prepare failed")
            .execute(named_params! {":r12id": r12id, ":r34id": r34id, ":r56id": r56id, ":r78id": r78id })
            {
                Ok(_) => trans.last_insert_rowid(),
                Err(_) => trans
                    .prepare(GET_POS_FOR_IDS_SQL)
                    .expect("prepare failed")
                    .query_row(named_params! {":r12id": r12id, ":r34id": r34id, ":r56id": r56id, ":r78id": r78id }, |row| row.get(0))
                    .expect("query failed"),
            };

        trans.commit().expect("Transaction failed");
        Ok(pos_id)
    }

    pub fn get_all(db: &Db) -> Result<Vec<(u32, u64, u64, u64, u64)>, Error> {
        let conn = db.connect();
        let mut stmt = conn
            .prepare(GET_ALL_POSITIONS_SQL)
            .expect("failed to prepare get_all_positions_sql");

        stmt.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .unwrap()
        .collect()
    }
}
