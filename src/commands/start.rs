use anyhow::{Result, bail};
use rusqlite::Connection;

use crate::frame::{self, Frame};

pub fn run(conn: &Connection, project: &str, tags: &[String]) -> Result<()> {
    if frame::get_current(conn)?.is_some() {
        bail!("already tracking - stop first");
    }

    let frame = frame::start(conn, project, tags)?;
    print_started(&frame);
    Ok(())
}

fn print_started(frame: &Frame) {
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    println!("Started {}{}", frame.project, tags_str);
}
