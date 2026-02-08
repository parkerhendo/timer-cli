use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection, id: i64) -> Result<()> {
    let changes = conn.execute("DELETE FROM frames WHERE id = ?1", [id])?;

    if changes == 0 {
        anyhow::bail!("frame {} not found", id);
    }

    println!("Deleted frame {}", id);
    Ok(())
}
