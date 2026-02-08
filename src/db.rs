use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub fn get_db_path() -> Result<PathBuf> {
    // Allow override for testing
    if let Ok(path) = std::env::var("TIMER_CLI_DB") {
        return Ok(PathBuf::from(path));
    }

    let proj = ProjectDirs::from("", "", "timer-cli")
        .context("could not determine home directory")?;
    let data_dir = proj.data_dir();
    fs::create_dir_all(data_dir).context("failed to create data directory")?;
    Ok(data_dir.join("frames.db"))
}

pub fn open() -> Result<Connection> {
    let path = get_db_path()?;
    let conn = Connection::open(&path)
        .with_context(|| format!("failed to open database at {}", path.display()))?;

    // Enable WAL mode for better concurrent access
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // Wait up to 5 seconds if database is locked
    conn.busy_timeout(std::time::Duration::from_secs(5))?;

    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS frames (
            id INTEGER PRIMARY KEY,
            project TEXT NOT NULL,
            start_time INTEGER NOT NULL,
            end_time INTEGER,
            tags TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_frames_project ON frames(project);
        CREATE INDEX IF NOT EXISTS idx_frames_start ON frames(start_time);
        ",
    )
    .context("failed to initialize database schema")?;
    Ok(())
}
