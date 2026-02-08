use anyhow::Result;
use chrono::Local;
use rusqlite::Connection;
use serde::Serialize;

use crate::frame::timestamp_to_local;

#[derive(Serialize)]
struct ExportFrame {
    id: i64,
    project: String,
    start_time: String,
    end_time: Option<String>,
    tags: Vec<String>,
    duration_seconds: i64,
}

pub fn run(conn: &Connection, format: ExportFormat) -> Result<()> {
    let frames = query_all_frames(conn)?;

    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&frames)?;
            println!("{}", json);
        }
        ExportFormat::Csv => {
            println!("id,project,start_time,end_time,tags,duration_seconds");
            for f in frames {
                println!(
                    "{},{},{},{},{},{}",
                    f.id,
                    escape_csv(&f.project),
                    f.start_time,
                    f.end_time.unwrap_or_default(),
                    escape_csv(&f.tags.join(",")),
                    f.duration_seconds
                );
            }
        }
    }

    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn query_all_frames(conn: &Connection) -> Result<Vec<ExportFrame>> {
    let mut stmt = conn.prepare(
        "SELECT id, project, start_time, end_time, tags FROM frames ORDER BY start_time",
    )?;

    let now = Local::now();
    let frames = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let project: String = row.get(1)?;
            let start_ts: i64 = row.get(2)?;
            let end_ts: Option<i64> = row.get(3)?;
            let tags_str: Option<String> = row.get(4)?;

            let start_time = timestamp_to_local(start_ts);
            let end_time = end_ts.map(timestamp_to_local);

            let duration_seconds = end_time
                .unwrap_or(now)
                .signed_duration_since(start_time)
                .num_seconds();

            Ok(ExportFrame {
                id,
                project,
                start_time: start_time.format("%Y-%m-%dT%H:%M:%S").to_string(),
                end_time: end_time.map(|t| t.format("%Y-%m-%dT%H:%M:%S").to_string()),
                tags: tags_str
                    .map(|s| s.split(',').map(String::from).collect())
                    .unwrap_or_default(),
                duration_seconds,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(frames)
}

#[derive(Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "csv" => Ok(ExportFormat::Csv),
            _ => Err(format!("unknown format: {s} (expected json or csv)")),
        }
    }
}
