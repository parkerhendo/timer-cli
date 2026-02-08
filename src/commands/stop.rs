use anyhow::Result;
use rusqlite::Connection;

use crate::frame::{self, Frame};

pub fn run(conn: &Connection) -> Result<()> {
    let frame = frame::get_current(conn)?
        .ok_or_else(|| anyhow::anyhow!("not tracking"))?;

    frame::stop(conn, frame.id)?;
    print_stopped(&frame);
    Ok(())
}

fn print_stopped(frame: &Frame) {
    let duration = Frame::format_duration(frame.duration());
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    println!("Stopped {}{} ({})", frame.project, tags_str, duration);
}
