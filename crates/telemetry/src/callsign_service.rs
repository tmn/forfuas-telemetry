use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::collections::HashMap;

pub struct CallsignService {
    pool: r2d2::Pool<SqliteConnectionManager>,
    cache: HashMap<String, String>,
}

impl CallsignService {
    pub fn new(db_path: &str) -> Self {
        let manager = SqliteConnectionManager::file(db_path);
        let pool = r2d2::Pool::new(manager)?; // .expect("Failed to create DB pool");

        Ok(Self {
            pool,
            cache: HashMap::new(),
        })
    }

    pub fn get_callsign(&mut self, sn: &str) -> Option<String> {
        if let Some(callsign) = self.cache.get(sn) {
            return Some(callsign.clone());
        }

        let conn = self.pool.get().ok()?;
        let result: Option<String> = conn.query_row(
            "SELECT callsign FROM uavs WHERE sn = ?1",
            params![sn],
            |row| row.get(0),
        ).ok();

        if let Some(ref callsign) = result {
            self.cache.insert(sn.to_string(), callsign.clone());
        }

        result
    }
}
