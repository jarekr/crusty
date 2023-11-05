use std::path::Path;
use rusqlite::{Connection, OpenFlags, named_params};


const INVENTORY_TABLE: &str = "inventory";
const INVENTORY_DDSQL: String = format!(
        "CREATE TABLE IF NOT EXISTS {table} (
            id       INTEGER PRIMARY KEY,
            name     TEXT,
            ipv4     INTEGER,
            hostname TEXT,
            notes    TEXT
        )", table=INVENTORY_TABLE);
const GET_BY_ID_QSQL: String = format!("SELECT * FROM {t} where id = :id order by id asc", t=INVENTORY_TABLE);
const GET_BY_

pub struct Db {
    conn: Connection
}

impl Db {

    pub fn new(dbpath: &Path) -> Connection {
        match Connection::open_with_flags(dbpath,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX) {
            Ok(conn) => conn,
            Err(why) => panic!("{}", why)

        }
    }

    fn create_schema(&self, sql: &str) -> usize {
        match self.conn.execute(sql, ()) {
            Ok(res) => res,
            Err(why) => panic!("schema create failed: {}", why)
        }
    }
}

pub struct InventoryRow {
    pub id: u32,
    pub name: String,
    pub ipv4: u32,
    pub hostname: String,
    pub notes: String,
}

impl InventoryRow {
    pub fn query_by_id(db: Db, id: u32) -> Option<InventoryRow> {
        let mut stmt = db.conn.prepare("SELECT * from inventory where id = :id order by id asc").expect("")
        let mut row_iter = stmt.query(named_params!{":id": id.to_string()}).unwrap();

        match row_iter.next().expect("next failed") {
            Some(row ) => {
                Some(InventoryRow {
                    id: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                    ipv4: row.get(2).unwrap(),
                    hostname: row.get(3).unwrap(),
                    notes: row.get(4).unwrap(),

                })
            }
            None => None
        }
    }
}