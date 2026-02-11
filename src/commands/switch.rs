use anyhow::Result;
use rusqlite::Connection;

use crate::frame::{self, Frame};
use crate::git;

pub fn run(conn: &mut Connection, quiet: bool) -> Result<()> {
    let Some((repo_name, branch)) = git::get_context() else {
        // Not in a git repo - silent exit
        return Ok(());
    };

    let current = frame::get_current(conn)?;

    // Check if already tracking the same project+tag
    if let Some(ref frame) = current {
        if frame.project == repo_name && frame.tags.first() == Some(&branch) {
            // Already tracking this context - no-op
            return Ok(());
        }
    }

    // Use transaction to make stop+start atomic
    let tx = conn.transaction()?;

    // Stop current frame if exists
    if let Some(ref frame) = current {
        frame::stop(&tx, frame.id)?;
        if !quiet {
            print_stopped(frame);
        }
    }

    // Start new frame with git context
    let new_frame = frame::start(&tx, &repo_name, &[branch])?;
    if !quiet {
        print_started(&new_frame);
    }

    tx.commit()?;
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

fn print_started(frame: &Frame) {
    let tags_str = if frame.tags.is_empty() {
        String::new()
    } else {
        format!(" +{}", frame.tags.join(" +"))
    };
    println!("Started {}{}", frame.project, tags_str);
}
