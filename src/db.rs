use const_format::concatcp;
use rusqlite::{named_params, Connection, Error, OpenFlags};
use std::path::Path;

use lzma;

const GAMES_TABLE: &str = "games";
const GAMES_DDSQL: &str = concatcp!(
    //TODO consider using BLOB for pgn text?
    "CREATE TABLE IF NOT EXISTS ",
    GAMES_TABLE,
    " (
        id INTEGER PRIMARY KEY,
        pgn TEXT,
        notes TEXT
        event TEXT,
        site TEXT,
        date TEXT,
        round TEXT,
        white TEXT,
        black TEXT,
        result TEXT,
        current_position TEXT,
        timezone TEXT,
        eco TEXT,
        eco_url TEXT,
        utc_date TEXT,
        utc_time TEXT,
        white_elo TEXT,
        black_elo TEXT,
        time_control TEXT,
        termination TEXT,
        variant TEXT,
        start_time TEXT,
        end_time TEXT,
        link TEXT)"
);
const GET_BY_ID_GAMES_SQL: &str = concatcp!(
    "SELECT * FROM ",
    GAMES_TABLE,
    " where id = :id order by id asc"
);
const INSERT_INTO_GAMES_SQL: &str = concatcp!(
    "INSERT INTO ",
    GAMES_TABLE,
    "( pgn,
       notes,
       event,
       site,
       date,
       round,
       white,
       black,
       result,
       current_position,
       timezone,
       eco,
       eco_url,
       utc_date,
       utc_time,
       white_elo,
       black_elo,
       time_control,
       termination,
       variant,
       start_time,
       end_time,
       link )
    VALUES (
       :pgn,
       :notes,
       :event,
       :site,
       :date,
       :round,
       :white,
       :black,
       :result,
       :current_position,
       :timezone,
       :eco,
       :eco_url,
       :utc_date,
       :utc_time,
       :white_elo,
       :black_elo,
       :time_control,
       :termination,
       :variant,
       :start_time,
       :end_time,
       :link )"
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

const GAME_POS_TABLE: &str = "game_positions";
const GAME_POS_DDSQL: &str = concatcp!(
    "CREATE TABLE IF NOT EXISTS ",
    GAME_POS_TABLE,
    " (
            id       INTEGER PRIMARY KEY,
            game_id  INTEGER NOT NULL,
            pos_id   INTEGER NOT NULL,
            FOREIGN KEY(game_id) REFERENCES ",
    GAMES_TABLE,
    "(id),
            FOREIGN KEY(pos_id) REFERENCES ",
    POSITIONS_TABLE,
    "(id)
    )"
);
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
            FOREIGN KEY(r12id) REFERENCES ",
    R12_TABLE,
    "(id),
            FOREIGN KEY(r34id) REFERENCES ",
    R34_TABLE,
    "(id),
            FOREIGN KEY(r56id) REFERENCES ",
    R56_TABLE,
    "(id),
            FOREIGN KEY(r78id) REFERENCES ",
    R78_TABLE,
    "(id)
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
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
    pub result: String,
    pub current_position: String,
    pub timezone: String,
    pub eco: String,
    pub eco_url: String,
    pub utc_date: String,
    pub utc_time: String,
    pub white_elo: String,
    pub black_elo: String,
    pub time_control: String,
    pub termination: String,
    pub variant: String,
    pub start_time: String,
    pub end_time: String,
    pub link: String,
}

impl Game {
    pub fn insert(db: &Db, game: &Game) -> Result<usize, Error> {
        //let mut comped = lzma::compress(pgn.as_bytes());
        let conn = db.connect();
        let mut stmt = conn.prepare(INSERT_INTO_GAMES_SQL).expect("prepare failed");
        stmt.execute(named_params! { ":pgn": game.pgn, ":notes": game.notes, ":event": game.event, ":site": game.site,
        ":date": game.date, ":round": game.round, ":white": game.white, ":black": game.black, ":result": game.result,
        ":current_position": game.current_position, ":timezone": game.timezone, ":eco": game.eco, ":eco_url": game.eco_url,
        ":utc_date": game.utc_date, ":utc_time": game.utc_time, ":white_elo": game.white_elo, ":black_elo": game.black_elo,
        ":time_control": game.time_control, ":termination": game.termination, ":variant": game.variant,
        ":start_time": game.start_time, ":end_time": game.end_time, ":link":  game.link})
    }
    pub fn query_by_id(db: &Db, id: u32) -> Option<Game> {
        let conn = db.connect();
        let mut stmt = conn.prepare(GET_BY_ID_GAMES_SQL).expect("prepare failed");
        let mut row_iter = stmt.query(named_params! {":id": id.to_string()}).unwrap();

        match row_iter.next().expect("next failed") {
            Some(row) => Some(Game {
                id: row.get(0).unwrap(),
                pgn: row.get(1).unwrap(),
                notes: row.get(3).unwrap(),
                event: row.get(4).unwrap(),
                site: row.get(5).unwrap(),
                date: row.get(6).unwrap(),
                round: row.get(7).unwrap(),
                white: row.get(8).unwrap(),
                black: row.get(9).unwrap(),
                result: row.get(10).unwrap(),
                current_position: row.get(11).unwrap(),
                timezone: row.get(12).unwrap(),
                eco: row.get(13).unwrap(),
                eco_url: row.get(14).unwrap(),
                utc_date: row.get(15).unwrap(),
                utc_time: row.get(16).unwrap(),
                white_elo: row.get(17).unwrap(),
                black_elo: row.get(18).unwrap(),
                time_control: row.get(19).unwrap(),
                termination: row.get(20).unwrap(),
                variant: row.get(21).unwrap(),
                start_time: row.get(22).unwrap(),
                end_time: row.get(23).unwrap(),
                link: row.get(24).unwrap(),
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
