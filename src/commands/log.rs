use anyhow::Result;
use chrono::{Local, LocalResult, NaiveDate, TimeZone};
use rusqlite::Connection;

use crate::frame::{timestamp_to_local, Frame};

pub fn run(conn: &Connection, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<()> {
    let today = Local::now().date_naive();
    let from_date = from.unwrap_or(today);
    let to_date = to.unwrap_or(today);

    let from_ts = date_to_start_timestamp(from_date);
    let to_ts = date_to_end_timestamp(to_date);

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

fn date_to_start_timestamp(date: NaiveDate) -> i64 {
    let dt = date.and_hms_opt(0, 0, 0).unwrap();
    match Local.from_local_datetime(&dt) {
        LocalResult::Single(local_dt) => local_dt.timestamp(),
        LocalResult::Ambiguous(earliest, _) => earliest.timestamp(),
        LocalResult::None => Local::now().timestamp(), // fallback
    }
}

fn date_to_end_timestamp(date: NaiveDate) -> i64 {
    let dt = date.and_hms_opt(23, 59, 59).unwrap();
    match Local.from_local_datetime(&dt) {
        LocalResult::Single(local_dt) => local_dt.timestamp(),
        LocalResult::Ambiguous(_, latest) => latest.timestamp(),
        LocalResult::None => Local::now().timestamp(), // fallback
    }
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
                start_time: timestamp_to_local(start_ts),
                end_time: end_ts.map(timestamp_to_local),
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
