use anyhow::Result;
use rusqlite::Connection;

use crate::frame::{self, Frame};

pub fn run(conn: &Connection) -> Result<()> {
    match frame::get_current(conn)? {
        Some(frame) => print_status(&frame),
        None => println!("Not tracking"),
    }
    Ok(())
}

fn print_status(frame: &Frame) {
    let duration = Frame::format_duration(frame.duration());
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    println!("{}{} ({})", frame.project, tags_str, duration);
}
