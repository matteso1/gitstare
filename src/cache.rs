use crate::git::RepoInfo;
use anyhow::Result;
use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;

const CACHE_MAX_AGE_SECS: i64 = 3600; // 1 hour

pub struct Cache {
    conn: Connection,
}

impl Cache {
    pub fn open() -> Result<Self> {
        let path = cache_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS repos (
                id INTEGER PRIMARY KEY,
                data TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn load(&self) -> Result<Option<Vec<RepoInfo>>> {
        let mut stmt = self
            .conn
            .prepare("SELECT data, updated_at FROM repos WHERE id = 1")?;

        let result = stmt.query_row([], |row| {
            let data: String = row.get(0)?;
            let updated_at: i64 = row.get(1)?;
            Ok((data, updated_at))
        });

        match result {
            Ok((data, updated_at)) => {
                let now = chrono::Utc::now().timestamp();
                if now - updated_at > CACHE_MAX_AGE_SECS {
                    return Ok(None);
                }
                let repos: Vec<RepoInfo> = serde_json::from_str(&data).unwrap_or_default();
                if repos.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(repos))
                }
            }
            Err(_) => Ok(None),
        }
    }

    pub fn save(&self, repos: &[RepoInfo]) -> Result<()> {
        let data = serde_json::to_string(repos)?;
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR REPLACE INTO repos (id, data, updated_at) VALUES (1, ?1, ?2)",
            rusqlite::params![data, now],
        )?;
        Ok(())
    }
}

fn cache_path() -> PathBuf {
    ProjectDirs::from("", "", "gitstare")
        .map(|d| d.cache_dir().join("gitstare.db"))
        .unwrap_or_else(|| PathBuf::from("gitstare.db"))
}
