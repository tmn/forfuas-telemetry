use r2d2::Error;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::collections::HashMap;

pub mod utils;

pub struct CallsignService {
    pool: r2d2::Pool<SqliteConnectionManager>,
    cache: HashMap<String, String>,
}

impl CallsignService {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::new(manager)?;

        let mut cache = HashMap::new();

        // Preload all callsigns at startup
        if let Ok(conn) = pool.get() {
            if let Ok(mut stmt) = conn.prepare("SELECT serial_number, callsign FROM uav_callsigns") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                }) {
                    for row in rows.flatten() {
                        cache.insert(row.0, row.1);
                    }
                }
            }
        }

        println!("Loaded {} callsigns from database", cache.len());

        Ok(Self { pool, cache })
    }

    pub fn get_callsign(&mut self, sn: &str) -> Option<String> {
        if let Some(callsign) = self.cache.get(sn) {
            return Some(callsign.clone());
        }

        let conn = self.pool.get().ok()?;
        let result: Option<String> = conn.query_row(
            "SELECT callsign FROM uav_callsigns WHERE serial_number = ?1",
            params![sn],
            |row| row.get(0),
        ).ok();

        if let Some(ref callsign) = result {
            self.cache.insert(sn.to_string(), callsign.clone());
        }

        result
    }
}