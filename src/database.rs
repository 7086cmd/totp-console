use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TOTPEntry {
    pub(crate) id: Option<i64>,
    pub(crate) name: String,
    pub(crate) secret: String,
    pub(crate) issuer: Option<String>,
    pub(crate) created_at: String,
}

// Database management
pub struct TOTPDatabase {
    conn: Connection,
}

impl TOTPDatabase {
    pub(crate) fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS totp_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                secret TEXT NOT NULL,
                issuer TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub(crate) fn add_entry(&self, entry: &TOTPEntry) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO totp_entries (name, secret, issuer, created_at) VALUES (?1, ?2, ?3, ?4)",
            [&entry.name, &entry.secret, &entry.issuer.as_deref().unwrap_or("").to_string(), &now],
        )?;

        println!("✅ Added TOTP entry: {}", entry.name);
        Ok(())
    }

    pub(crate) fn get_all_entries(&self) -> SqliteResult<Vec<TOTPEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, secret, issuer, created_at FROM totp_entries ORDER BY name"
        )?;

        let entries = stmt.query_map([], |row| {
            Ok(TOTPEntry {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                secret: row.get(2)?,
                issuer: {
                    let issuer: String = row.get(3)?;
                    if issuer.is_empty() { None } else { Some(issuer) }
                },
                created_at: row.get(4)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    pub(crate) fn get_entry_by_name(&self, name: &str) -> SqliteResult<Option<TOTPEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, secret, issuer, created_at FROM totp_entries WHERE name = ?1"
        )?;

        let mut entries = stmt.query_map([name], |row| {
            Ok(TOTPEntry {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                secret: row.get(2)?,
                issuer: {
                    let issuer: String = row.get(3)?;
                    if issuer.is_empty() { None } else { Some(issuer) }
                },
                created_at: row.get(4)?,
            })
        })?;

        match entries.next() {
            Some(entry) => Ok(Some(entry?)),
            None => Ok(None),
        }
    }

    pub(crate) fn delete_entry(&self, name: &str) -> SqliteResult<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM totp_entries WHERE name = ?1",
            [name],
        )?;

        Ok(rows_affected > 0)
    }
}
