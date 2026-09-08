// SPDX-License-Identifier: CC-BY-NC-ND-4.0
// See LICENSE for additional ethical restrictions (non-military, non-governmental, no derivatives).
//! storage.rs — Persistência SQLite WAL e Escrita Atômica de Estado

use rusqlite::{params, Connection};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

pub struct SovereignStorage {
    db_path: String,
}

impl SovereignStorage {
    pub fn new(db_path: &str) -> Self {
        let storage = Self {
            db_path: db_path.to_string(),
        };
        storage.init_db();
        storage
    }

    fn init_db(&self) {
        if let Ok(conn) = self.connect() {
            let _ = conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA busy_timeout = 30000;
                 CREATE TABLE IF NOT EXISTS sovereign_primary_rust_shadow_snapshots (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     recorded_at REAL NOT NULL,
                     cycle INTEGER NOT NULL,
                     integration_cycle INTEGER,
                     writer_source TEXT NOT NULL,
                     writer_role TEXT,
                     status TEXT NOT NULL,
                     pressure_level TEXT,
                     phi_iit_normalized REAL,
                     phi_iit_nats REAL,
                     sigma REAL,
                     epsilon REAL,
                     payload_json TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS idx_rust_shadow_time
                     ON sovereign_primary_rust_shadow_snapshots(recorded_at DESC);",
            );
        }
    }

    pub fn connect(&self) -> Result<Connection, rusqlite::Error> {
        if let Some(parent) = Path::new(&self.db_path).parent() {
            let _ = fs::create_dir_all(parent);
        }
        let conn = Connection::open(&self.db_path)?;
        conn.busy_timeout(std::time::Duration::from_millis(30000))?;
        Ok(conn)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn persist_snapshot(
        &self,
        cycle: u64,
        status: &str,
        pressure: &str,
        phi_norm: f64,
        phi_nats: f64,
        sigma: f64,
        epsilon: f64,
        payload: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.connect()?;
        // 2026-09-06: recorded_at como epoch REAL (Unix timestamp) — consistente
        // com o daemon Python e com o shadow Fase 2. O formato RFC3339 (TEXT)
        // poluía o banco com tipos mistos (mesmo padrão que corrompeu o somatic).
        let now = chrono::Utc::now().timestamp() as f64;
        let payload_str = serde_json::to_string(payload)?;

        conn.execute(
            "INSERT INTO sovereign_primary_rust_shadow_snapshots (
                recorded_at, cycle, integration_cycle, writer_source, writer_role,
                status, pressure_level, phi_iit_normalized, phi_iit_nats, sigma, epsilon, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                now,
                cycle as i64,
                cycle as i64,
                "sovereign_primary_rust_shadow",
                "shadow_governor",
                status,
                pressure,
                phi_norm,
                phi_nats,
                sigma,
                epsilon,
                payload_str,
            ],
        )?;

        Ok(())
    }

    pub fn atomic_write_json(path: &Path, payload: &Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let mut tmp_file = NamedTempFile::new_in(parent)?;
        let json_bytes = serde_json::to_vec_pretty(payload)?;
        tmp_file.write_all(&json_bytes)?;
        tmp_file.write_all(b"\n")?;
        tmp_file.flush()?;
        tmp_file.persist(path)?;
        Ok(())
    }
}
