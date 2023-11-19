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
        notes TEXT,
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
        opening TEXT,
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
       opening,
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
       :opening,
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
            turn     INTEGER NOT NULL,
            CONSTRAINT uniq_game_pos UNIQUE (game_id, pos_id, turn),
            FOREIGN KEY(game_id) REFERENCES ",
    GAMES_TABLE,
    "(id),
            FOREIGN KEY(pos_id) REFERENCES ",
    POSITIONS_TABLE,
    "(id)
    )"
);
const INSERT_GAMES_POS_SQL: &str = concatcp!(
    "INSERT INTO ",
    GAME_POS_TABLE,
    " ( game_id, pos_id, turn ) VALUES ( :game_id, :pos_id, :turn )"
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
            GAME_POS_DDSQL,
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
    pub id: i64,
    pub pgn: Option<String>,
    pub notes: Option<String>,
    pub event: String,
    pub site: String,
    pub date: Option<String>,
    pub round: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub current_position: Option<String>,
    pub timezone: Option<String>,
    pub eco: Option<String>,
    pub eco_url: Option<String>,
    pub opening: Option<String>,
    pub utc_date: Option<String>,
    pub utc_time: Option<String>,
    pub white_elo: Option<String>,
    pub black_elo: Option<String>,
    pub time_control: Option<String>,
    pub termination: Option<String>,
    pub variant: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub link: Option<String>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            id: 0,
            pgn: None,
            notes: None,
            event: "".to_string(),
            site: "".to_string(),
            date: None,
            round: None,
            white: None,
            black: None,
            result: None,
            current_position: None,
            timezone: None,
            eco: None,
            eco_url: None,
            opening: None,
            utc_date: None,
            utc_time: None,
            white_elo: None,
            black_elo: None,
            time_control: None,
            termination: None,
            variant: None,
            start_time: None,
            end_time: None,
            link: None,
        }
    }
    pub fn insert(db: &Db, game: &Game) -> Result<i64, Error> {
        //let mut comped = lzma::compress(pgn.as_bytes());
        let conn = db.connect();
        let mut stmt = conn.prepare(INSERT_INTO_GAMES_SQL).expect("prepare failed");
        stmt.insert(named_params! { ":pgn": game.pgn, ":notes": game.notes, ":event": game.event, ":site": game.site,
        ":date": game.date, ":round": game.round, ":white": game.white, ":black": game.black, ":result": game.result,
        ":current_position": game.current_position, ":timezone": game.timezone, ":eco": game.eco, ":eco_url": game.eco_url,
        ":opening": game.opening, ":utc_date": game.utc_date, ":utc_time": game.utc_time, ":white_elo": game.white_elo,
        ":black_elo": game.black_elo, ":time_control": game.time_control, ":termination": game.termination,
        ":variant": game.variant, ":start_time": game.start_time, ":end_time": game.end_time, ":link":  game.link})
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
                opening: row.get(15).unwrap(),
                utc_date: row.get(16).unwrap(),
                utc_time: row.get(17).unwrap(),
                white_elo: row.get(18).unwrap(),
                black_elo: row.get(19).unwrap(),
                time_control: row.get(20).unwrap(),
                termination: row.get(21).unwrap(),
                variant: row.get(22).unwrap(),
                start_time: row.get(23).unwrap(),
                end_time: row.get(24).unwrap(),
                link: row.get(25).unwrap(),
            }),
            None => None,
        }
    }
}

pub struct Position {
    pub id: i64,
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
                .insert(named_params! {":row": v as i64 })
            {
                Ok(id) => id,
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
            .insert(named_params! {":r12id": r12id, ":r34id": r34id, ":r56id": r56id, ":r78id": r78id })
            {
                Ok(id) => id,
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

pub struct GamePosition {
    pub id: i64,
    pub game_id: i64,
    pub pos_id: i64,
    pub turn: u16,
}

impl GamePosition {
    pub fn insert(db: &Db, game_id: i64, position_ids: Vec<i64>) -> Result<(), Error> {
        let mut conn = db.connect();
        let trans = conn.transaction().expect("failed to start transaction");
        for (turn, pos_id) in position_ids.iter().enumerate() {
            trans
                .prepare(INSERT_GAMES_POS_SQL)
                .expect("prepare failed")
                .insert(named_params! {":game_id": game_id, ":pos_id": pos_id, ":turn": turn})
                .expect("insert failed");
        }
        trans.commit()
    }
}
