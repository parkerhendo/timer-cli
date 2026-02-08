use anyhow::Result;
use rusqlite::Connection;

use crate::frame;

pub fn run(conn: &Connection) -> Result<()> {
    let current = frame::get_current(conn)?
        .ok_or_else(|| anyhow::anyhow!("not tracking"))?;

    conn.execute("DELETE FROM frames WHERE id = ?1", [current.id])?;

    let tags_str = if current.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", current.tags.join(" +"))
    };
    println!("Cancelled {}{}", current.project, tags_str);
    Ok(())
}
