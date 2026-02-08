use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local, TimeZone};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Debug)]
pub struct Frame {
    pub id: i64,
    pub project: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub tags: Vec<String>,
}

impl Frame {
    pub fn duration(&self) -> Duration {
        let end = self.end_time.unwrap_or_else(Local::now);
        end.signed_duration_since(self.start_time)
    }

    pub fn format_duration(d: Duration) -> String {
        let hours = d.num_hours();
        let mins = d.num_minutes() % 60;
        let secs = d.num_seconds() % 60;
        if hours > 0 {
            format!("{hours}h {mins}m")
        } else if mins > 0 {
            format!("{mins}m {secs}s")
        } else {
            format!("{secs}s")
        }
    }
}

pub fn get_current(conn: &Connection) -> Result<Option<Frame>> {
    conn.query_row(
        "SELECT id, project, start_time, end_time, tags FROM frames WHERE end_time IS NULL",
        [],
        |row| {
            let id: i64 = row.get(0)?;
            let project: String = row.get(1)?;
            let start_ts: i64 = row.get(2)?;
            let end_ts: Option<i64> = row.get(3)?;
            let tags_str: Option<String> = row.get(4)?;

            Ok(Frame {
                id,
                project,
                start_time: Local.timestamp_opt(start_ts, 0).unwrap(),
                end_time: end_ts.map(|ts| Local.timestamp_opt(ts, 0).unwrap()),
                tags: tags_str
                    .map(|s| s.split(',').map(String::from).collect())
                    .unwrap_or_default(),
            })
        },
    )
    .optional()
    .context("failed to query current frame")
}

pub fn start(conn: &Connection, project: &str, tags: &[String]) -> Result<Frame> {
    let now = Local::now();
    let tags_str = if tags.is_empty() {
        None
    } else {
        Some(tags.join(","))
    };

    conn.execute(
        "INSERT INTO frames (project, start_time, tags) VALUES (?1, ?2, ?3)",
        params![project, now.timestamp(), tags_str],
    )
    .context("failed to insert frame")?;

    let id = conn.last_insert_rowid();
    Ok(Frame {
        id,
        project: project.to_string(),
        start_time: now,
        end_time: None,
        tags: tags.to_vec(),
    })
}

pub fn stop(conn: &Connection, frame_id: i64) -> Result<()> {
    let now = Local::now();
    conn.execute(
        "UPDATE frames SET end_time = ?1 WHERE id = ?2",
        params![now.timestamp(), frame_id],
    )
    .context("failed to stop frame")?;
    Ok(())
}
