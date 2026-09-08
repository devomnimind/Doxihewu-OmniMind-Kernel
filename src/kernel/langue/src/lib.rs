#![allow(non_local_definitions)]
use pyo3::prelude::*;
use memmap2::Mmap;
use rusqlite::{Connection, params};
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::path::{Path, PathBuf};
use chrono::Utc;

mod protocol;
mod tifinagh;
mod behaviors;

use protocol::TransatlanticProtocol;
use tifinagh::to_tifinagh;

/// Langue Kernel Interface — Handle-based file operations
/// 
/// Users see lexeme handles (e.g., "MA-MU-atã-me'ẽ"), kernel manages real paths.
/// Integrates Banto (WHAT) + Tupi-Guarani (HOW) linguistic protocol.
#[pyclass(unsendable)]
pub struct LangueKernelInterface {
    db: Connection,
    matrix: TransatlanticProtocol,
    #[allow(dead_code)]
    db_path: PathBuf,
}

#[pymethods]
impl LangueKernelInterface {
    #[new]
    pub fn new(db_path: &str, matrix_path: &str) -> PyResult<Self> {
        // Load transatlantic protocol matrix
        let matrix_str = std::fs::read_to_string(matrix_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!("Failed to load matrix: {}", e)
            ))?;
        
        let matrix: Value = serde_json::from_str(&matrix_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid JSON in matrix: {}", e)
            ))?;
        
        let protocol = TransatlanticProtocol::from_json(matrix)?;
        
        // Open SQLite database
        let db = Connection::open(db_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!("Failed to open database: {}", e)
            ))?;
        
        // Initialize schema
        Self::init_db(&db)?;
        
        Ok(LangueKernelInterface {
            db,
            matrix: protocol,
            db_path: PathBuf::from(db_path),
        })
    }
    
    /// Create a handle for a file path
    /// 
    /// Args:
    ///     lexeme_structure: Transatlantic lexeme (e.g., "MA-MU-atã-me'ẽ")
    ///     file_path: Real filesystem path
    ///     user_visible_name: Optional human-readable name
    ///     metadata_json: Optional JSON metadata string
    /// 
    /// Returns:
    ///     Handle string (SHA256 hex)
    pub fn create_handle(
        &mut self,
        lexeme_structure: &str,
        file_path: &str,
        user_visible_name: Option<&str>,
        metadata_json: Option<&str>,
    ) -> PyResult<String> {
        // Validate file exists
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
                format!("File not found: {}", file_path)
            ));
        }
        
        // Validate lexeme structure
        self.matrix.validate_lexeme(lexeme_structure)?;
        
        // Generate handle (SHA256 of lexeme + timestamp + path)
        let timestamp = Utc::now().timestamp();
        let handle_input = format!("{}:{}:{}", lexeme_structure, timestamp, file_path);
        let mut hasher = Sha256::new();
        hasher.update(handle_input.as_bytes());
        let handle = format!("{:x}", hasher.finalize());
        
        // Extract prefix and particles
        let (prefixo, particulas) = self.matrix.decompose_lexeme(lexeme_structure)?;
        
        // Convert to Tifinagh
        let tifinagh = to_tifinagh(lexeme_structure, &self.matrix)?;
        
        // Get file size
        let metadata = std::fs::metadata(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        let size_bytes = metadata.len();
        
        // Compute SHA256 of file content
        let mut file = File::open(path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        let mut file_hasher = Sha256::new();
        std::io::copy(&mut file, &mut file_hasher)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
        let sha256_current = format!("{:x}", file_hasher.finalize());
        
        // Insert into database
        self.db.execute(
            "INSERT INTO langue_handles (
                handle, lexeme_structure, current_path, original_path,
                prefixo_classe, particulas_tupi, script_tifinagh,
                sha256_current, size_bytes, created_at, last_accessed,
                relocation_count, kernel_visible, user_visible_name, metadata_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 1, ?, ?)",
            params![
                handle,
                lexeme_structure,
                file_path,
                file_path,
                prefixo,
                particulas.join(","),
                tifinagh,
                sha256_current,
                size_bytes as i64,
                timestamp,
                timestamp,
                user_visible_name.unwrap_or(lexeme_structure),
                metadata_json.unwrap_or("{}"),
            ],
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Database insert failed: {}", e)
        ))?;
        
        Ok(handle)
    }
    
    /// Read file by handle (zero-copy mmap for large files)
    /// 
    /// Args:
    ///     handle: Handle string (SHA256)
    ///     use_mmap: Use memory-mapped I/O for files > 1 MiB
    /// 
    /// Returns:
    ///     File contents as bytes
    pub fn read_by_handle<'py>(&mut self, py: Python<'py>, handle: &str, use_mmap: Option<bool>) -> PyResult<Bound<'py, pyo3::types::PyBytes>> {
        // Query path from database
        let (path, size_bytes): (String, i64) = self.db
            .query_row(
                "SELECT current_path, size_bytes FROM langue_handles WHERE handle = ?",
                params![handle],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Handle not found: {}", e)
            ))?;
        
        // Update last_accessed
        let timestamp = Utc::now().timestamp();
        self.db.execute(
            "UPDATE langue_handles SET last_accessed = ? WHERE handle = ?",
            params![timestamp, handle],
        ).ok();
        
        // Read file
        let use_mmap = use_mmap.unwrap_or(true);
        let threshold = 1024 * 1024; // 1 MiB
        
        if use_mmap && size_bytes > threshold {
            // Zero-copy mmap (same as nsh)
            let file = File::open(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            let mmap = unsafe { Mmap::map(&file) }
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            Ok(pyo3::types::PyBytes::new(py, &mmap))
        } else {
            // Standard read
            let data = std::fs::read(&path)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))?;
            Ok(pyo3::types::PyBytes::new(py, &data))
        }
    }
    
    /// Relocate file to new path (handle stays valid)
    /// 
    /// Args:
    ///     handle: Handle string
    ///     new_path: New filesystem path
    ///     kernel_agent: Agent performing relocation (e.g., "kernel_control_plane")
    pub fn relocate(&mut self, handle: &str, new_path: &str, kernel_agent: &str) -> PyResult<()> {
        // Query current path
        let old_path: String = self.db
            .query_row(
                "SELECT current_path FROM langue_handles WHERE handle = ?",
                params![handle],
                |row| row.get(0),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Handle not found: {}", e)
            ))?;
        
        // Move file
        std::fs::rename(&old_path, new_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(
                format!("Failed to relocate file: {}", e)
            ))?;
        
        // Update database
        let timestamp = Utc::now().timestamp();
        self.db.execute(
            "UPDATE langue_handles SET current_path = ?, relocation_count = relocation_count + 1 WHERE handle = ?",
            params![new_path, handle],
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        // Audit log
        self.db.execute(
            "INSERT INTO handle_operations (handle, operation, old_path, new_path, timestamp, kernel_agent)
             VALUES (?, 'relocate', ?, ?, ?, ?)",
            params![handle, old_path, new_path, timestamp, kernel_agent],
        ).ok();
        
        Ok(())
    }
    
    /// List visible handles (optionally filtered by prefix)
    /// 
    /// Args:
    ///     prefixo_filter: Optional Banto prefix filter (e.g., "MA-")
    /// 
    /// Returns:
    ///     List of (handle, lexeme, user_visible_name) tuples
    pub fn list_visible_handles(&self, prefixo_filter: Option<&str>) -> PyResult<Vec<(String, String, String)>> {
        let query = if let Some(prefix) = prefixo_filter {
            format!(
                "SELECT handle, lexeme_structure, user_visible_name FROM langue_handles 
                 WHERE kernel_visible = 1 AND prefixo_classe = '{}' ORDER BY created_at DESC",
                prefix
            )
        } else {
            "SELECT handle, lexeme_structure, user_visible_name FROM langue_handles 
             WHERE kernel_visible = 1 ORDER BY created_at DESC".to_string()
        };
        
        let mut stmt = self.db.prepare(&query)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?);
        }
        
        Ok(results)
    }
    
    /// Get statistics about handles
    /// 
    /// Returns:
    ///     Dictionary with statistics
    pub fn get_statistics(&self) -> PyResult<std::collections::HashMap<String, i64>> {
        let mut stats = std::collections::HashMap::new();
        
        // Total handles
        let total: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM langue_handles",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        stats.insert("total_handles".to_string(), total);
        
        // Total size
        let total_size: i64 = self.db.query_row(
            "SELECT SUM(size_bytes) FROM langue_handles",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        stats.insert("total_size_bytes".to_string(), total_size);
        
        // Total relocations
        let total_relocations: i64 = self.db.query_row(
            "SELECT SUM(relocation_count) FROM langue_handles",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        stats.insert("total_relocations".to_string(), total_relocations);
        
        Ok(stats)
    }
    
    /// Enforce kernel behaviors for a handle (-atã, -pema, -me'ẽ)
    pub fn enforce_behaviors(&self, handle: &str) -> PyResult<()> {
        let (path, lexeme): (String, String) = self.db
            .query_row(
                "SELECT current_path, lexeme_structure FROM langue_handles WHERE handle = ?",
                params![handle],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Handle not found: {}", e)
            ))?;
        
        behaviors::enforce(&path, &lexeme)?;
        Ok(())
    }
}

impl LangueKernelInterface {
    fn init_db(db: &Connection) -> PyResult<()> {
        db.execute(
            "CREATE TABLE IF NOT EXISTS langue_handles (
                handle TEXT PRIMARY KEY,
                lexeme_structure TEXT NOT NULL,
                current_path TEXT NOT NULL,
                original_path TEXT NOT NULL,
                prefixo_classe TEXT NOT NULL,
                particulas_tupi TEXT NOT NULL,
                script_tifinagh TEXT NOT NULL,
                sha256_current TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                relocation_count INTEGER NOT NULL DEFAULT 0,
                kernel_visible INTEGER NOT NULL DEFAULT 1,
                user_visible_name TEXT NOT NULL,
                metadata_json TEXT NOT NULL DEFAULT '{}'
            )",
            [],
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        db.execute(
            "CREATE TABLE IF NOT EXISTS handle_operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                handle TEXT NOT NULL,
                operation TEXT NOT NULL,
                old_path TEXT,
                new_path TEXT,
                timestamp INTEGER NOT NULL,
                kernel_agent TEXT NOT NULL,
                phi_ecosystem REAL,
                psi_trace TEXT,
                FOREIGN KEY (handle) REFERENCES langue_handles(handle)
            )",
            [],
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        // Indices
        db.execute("CREATE INDEX IF NOT EXISTS idx_prefixo ON langue_handles(prefixo_classe)", []).ok();
        db.execute("CREATE INDEX IF NOT EXISTS idx_created ON langue_handles(created_at DESC)", []).ok();
        db.execute("CREATE INDEX IF NOT EXISTS idx_ops_handle ON handle_operations(handle)", []).ok();
        
        Ok(())
    }
}

#[pymodule]
fn omnimind_langue(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LangueKernelInterface>()?;
    Ok(())
}
