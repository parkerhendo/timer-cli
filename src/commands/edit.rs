use anyhow::{Context, Result};
use chrono::{Local, LocalResult, NaiveDateTime, TimeZone};
use rusqlite::{Connection, OptionalExtension, params};

pub fn run(
    conn: &Connection,
    id: i64,
    project: Option<String>,
    tags: Option<Vec<String>>,
    start: Option<String>,
    end: Option<String>,
) -> Result<()> {
    // Verify frame exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM frames WHERE id = ?1", [id], |_| Ok(true))
        .optional()
        .context("failed to check if frame exists")?
        .is_some();

    if !exists {
        anyhow::bail!("frame {} not found", id);
    }

    if let Some(ref proj) = project {
        conn.execute("UPDATE frames SET project = ?1 WHERE id = ?2", params![proj, id])?;
    }

    if let Some(ref t) = tags {
        let tags_str = if t.is_empty() { None } else { Some(t.join(",")) };
        conn.execute("UPDATE frames SET tags = ?1 WHERE id = ?2", params![tags_str, id])?;
    }

    if let Some(ref s) = start {
        let ts = parse_datetime(s).context("invalid start time")?;
        conn.execute("UPDATE frames SET start_time = ?1 WHERE id = ?2", params![ts, id])?;
    }

    if let Some(ref e) = end {
        let ts = parse_datetime(e).context("invalid end time")?;
        conn.execute("UPDATE frames SET end_time = ?1 WHERE id = ?2", params![ts, id])?;
    }

    println!("Updated frame {}", id);
    Ok(())
}

fn parse_datetime(s: &str) -> Result<i64> {
    // Try formats: "HH:MM" (today), "YYYY-MM-DD HH:MM"
    if let Ok(time) = chrono::NaiveTime::parse_from_str(s, "%H:%M") {
        let today = Local::now().date_naive();
        let dt = today.and_time(time);
        return local_datetime_to_timestamp(&dt);
    }

    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M") {
        return local_datetime_to_timestamp(&dt);
    }

    anyhow::bail!("expected HH:MM or YYYY-MM-DD HH:MM")
}

fn local_datetime_to_timestamp(dt: &chrono::NaiveDateTime) -> Result<i64> {
    match Local.from_local_datetime(dt) {
        LocalResult::Single(local_dt) => Ok(local_dt.timestamp()),
        LocalResult::Ambiguous(earliest, _) => Ok(earliest.timestamp()),
        LocalResult::None => anyhow::bail!("invalid time (may be during DST transition)"),
    }
}
