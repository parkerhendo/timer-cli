use anyhow::Result;
use chrono::{Local, NaiveDate, TimeZone};
use rusqlite::Connection;

use crate::frame::Frame;

pub fn run(conn: &Connection, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<()> {
    let today = Local::now().date_naive();
    let from_date = from.unwrap_or(today);
    let to_date = to.unwrap_or(today);

    let from_ts = Local
        .from_local_datetime(&from_date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .timestamp();
    let to_ts = Local
        .from_local_datetime(&to_date.and_hms_opt(23, 59, 59).unwrap())
        .unwrap()
        .timestamp();

    let frames = query_frames(conn, from_ts, to_ts)?;

    if frames.is_empty() {
        println!("No frames found");
        return Ok(());
    }

    for frame in frames {
        print_frame(&frame);
    }

    Ok(())
}

fn query_frames(conn: &Connection, from_ts: i64, to_ts: i64) -> Result<Vec<Frame>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, start_time, end_time, tags
         FROM frames
         WHERE start_time >= ?1 AND start_time <= ?2
         ORDER BY start_time DESC",
    )?;

    let frames = stmt
        .query_map([from_ts, to_ts], |row| {
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
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(frames)
}

fn print_frame(frame: &Frame) {
    let duration = Frame::format_duration(frame.duration());
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    let time_range = format!(
        "{} - {}",
        frame.start_time.format("%H:%M"),
        frame
            .end_time
            .map(|t| t.format("%H:%M").to_string())
            .unwrap_or_else(|| "now".to_string())
    );
    println!(
        "[{}] {}{} ({}) {}",
        frame.id, frame.project, tags_str, duration, time_range
    );
}
