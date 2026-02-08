use anyhow::Result;
use chrono::{Duration, Local, NaiveDate, TimeZone};
use rusqlite::Connection;
use std::collections::HashMap;

use crate::frame::Frame;

pub fn run(
    conn: &Connection,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    by_tag: bool,
) -> Result<()> {
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

    let mut stmt = conn.prepare(
        "SELECT id, project, start_time, end_time, tags
         FROM frames
         WHERE start_time >= ?1 AND start_time <= ?2",
    )?;

    let frames: Vec<crate::frame::Frame> = stmt
        .query_map([from_ts, to_ts], |row| {
            let id: i64 = row.get(0)?;
            let project: String = row.get(1)?;
            let start_ts: i64 = row.get(2)?;
            let end_ts: Option<i64> = row.get(3)?;
            let tags_str: Option<String> = row.get(4)?;

            Ok(crate::frame::Frame {
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

    if frames.is_empty() {
        println!("No frames found");
        return Ok(());
    }

    if by_tag {
        print_by_tag(&frames);
    } else {
        print_by_project(&frames);
    }

    Ok(())
}

fn print_by_project(frames: &[crate::frame::Frame]) {
    let mut totals: HashMap<&str, Duration> = HashMap::new();

    for frame in frames {
        *totals.entry(&frame.project).or_insert(Duration::zero()) += frame.duration();
    }

    let mut sorted: Vec<_> = totals.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    let grand_total: Duration = sorted.iter().map(|(_, d)| *d).sum();

    println!("By project:");
    for (project, duration) in &sorted {
        println!("  {} {}", project, Frame::format_duration(*duration));
    }
    println!("Total: {}", Frame::format_duration(grand_total));
}

fn print_by_tag(frames: &[crate::frame::Frame]) {
    let mut totals: HashMap<&str, Duration> = HashMap::new();

    for frame in frames {
        if frame.tags.is_empty() {
            *totals.entry("(untagged)").or_insert(Duration::zero()) += frame.duration();
        } else {
            for tag in &frame.tags {
                *totals.entry(tag).or_insert(Duration::zero()) += frame.duration();
            }
        }
    }

    let mut sorted: Vec<_> = totals.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    println!("By tag:");
    for (tag, duration) in &sorted {
        let prefix = if *tag == "(untagged)" { "" } else { "+" };
        println!("  {}{} {}", prefix, tag, Frame::format_duration(*duration));
    }
}
