use anyhow::Result;
use rusqlite::Connection;

use crate::frame::{self, Frame};

pub fn run(conn: &Connection) -> Result<()> {
    if frame::get_current(conn)?.is_some() {
        anyhow::bail!("already tracking - stop first");
    }

    let last = get_last_frame(conn)?
        .ok_or_else(|| anyhow::anyhow!("no previous frame to restart"))?;

    let new_frame = frame::start(conn, &last.project, &last.tags)?;
    print_started(&new_frame);
    Ok(())
}

fn get_last_frame(conn: &Connection) -> Result<Option<Frame>> {
    use chrono::{Local, TimeZone};

    conn.query_row(
        "SELECT id, project, start_time, end_time, tags
         FROM frames
         WHERE end_time IS NOT NULL
         ORDER BY end_time DESC
         LIMIT 1",
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
    .map_err(Into::into)
}

use rusqlite::OptionalExtension;

fn print_started(frame: &Frame) {
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    println!("Started {}{}", frame.project, tags_str);
}
