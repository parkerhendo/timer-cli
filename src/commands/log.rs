use anyhow::Result;
use chrono::{Local, LocalResult, NaiveDate, TimeZone};
use rusqlite::Connection;

use crate::frame::{timestamp_to_local, Frame};

struct DisplayRow {
    id: String,
    project: String,
    tags: String,
    duration: String,
    time_range: String,
}

impl DisplayRow {
    fn from_frame(frame: &Frame) -> Self {
        let tags = if frame.tags.is_empty() {
            String::new()
        } else {
            format!("+{}", frame.tags.join(" +"))
        };
        let time_range = format!(
            "{} - {}",
            frame.start_time.format("%H:%M"),
            frame
                .end_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "now".to_string())
        );
        Self {
            id: frame.id.to_string(),
            project: frame.project.clone(),
            tags,
            duration: Frame::format_duration(frame.duration()),
            time_range,
        }
    }
}

pub fn run(conn: &Connection, from: Option<NaiveDate>, to: Option<NaiveDate>, all: bool) -> Result<()> {
    let frames = if all {
        query_all_frames(conn)?
    } else {
        let today = Local::now().date_naive();
        let from_date = from.unwrap_or(today);
        let to_date = to.unwrap_or(today);
        let from_ts = date_to_start_timestamp(from_date);
        let to_ts = date_to_end_timestamp(to_date);
        query_frames(conn, from_ts, to_ts)?
    };

    if frames.is_empty() {
        println!("No frames found");
        return Ok(());
    }

    // Two-pass: collect display data, then print with alignment
    let rows: Vec<DisplayRow> = frames.iter().map(DisplayRow::from_frame).collect();

    let max_id = rows.iter().map(|r| r.id.len()).max().unwrap_or(0);
    let max_project = rows.iter().map(|r| r.project.len()).max().unwrap_or(0);
    let max_tags = rows.iter().map(|r| r.tags.len()).max().unwrap_or(0);
    let max_duration = rows.iter().map(|r| r.duration.len()).max().unwrap_or(0);

    for row in &rows {
        println!(
            "[{:>id_w$}] {:<proj_w$}  {:<tag_w$}  {:>dur_w$}  {}",
            row.id, row.project, row.tags, row.duration, row.time_range,
            id_w = max_id,
            proj_w = max_project,
            tag_w = max_tags,
            dur_w = max_duration,
        );
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
        .query_map([from_ts, to_ts], |row| row_to_frame(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(frames)
}

fn query_all_frames(conn: &Connection) -> Result<Vec<Frame>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, start_time, end_time, tags
         FROM frames
         ORDER BY start_time DESC",
    )?;

    let frames = stmt
        .query_map([], |row| row_to_frame(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(frames)
}

fn row_to_frame(row: &rusqlite::Row) -> rusqlite::Result<Frame> {
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
}
