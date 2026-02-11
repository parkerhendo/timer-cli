mod commands;
mod db;
mod frame;
mod git;

use anyhow::Result;
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "timer-cli", version, about = "Track your time")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start tracking time on a project
    Start {
        /// Project name
        project: String,
        /// Tags (prefix with +)
        #[arg(value_parser = parse_tag)]
        tags: Vec<String>,
    },
    /// Stop the current frame
    Stop,
    /// Show current tracking status
    Status,
    /// Show recent frames
    Log {
        /// Start date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        from: Option<NaiveDate>,
        /// End date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        to: Option<NaiveDate>,
    },
    /// Cancel (delete) the current frame
    Cancel,
    /// Delete a frame by ID
    Delete {
        /// Frame ID to delete
        id: i64,
    },
    /// List all projects
    Projects,
    /// List all tags
    Tags,
    /// Show time report aggregated by project or tag
    Report {
        /// Start date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        from: Option<NaiveDate>,
        /// End date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        to: Option<NaiveDate>,
        /// Group by tag instead of project
        #[arg(long)]
        by_tag: bool,
    },
    /// Edit an existing frame
    Edit {
        /// Frame ID to edit
        id: i64,
        /// New project name
        #[arg(short, long)]
        project: Option<String>,
        /// New tags (prefix with +)
        #[arg(short, long, value_parser = parse_tag)]
        tags: Option<Vec<String>>,
        /// New start time (HH:MM or YYYY-MM-DD HH:MM)
        #[arg(short, long)]
        start: Option<String>,
        /// New end time (HH:MM or YYYY-MM-DD HH:MM)
        #[arg(short, long)]
        end: Option<String>,
    },
    /// Restart the last stopped frame
    Restart,
    /// Export all frames to JSON or CSV
    Export {
        /// Output format (json or csv)
        #[arg(short, long, default_value = "json")]
        format: commands::ExportFormat,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Sync timer with current git context (project=repo, tag=branch)
    Switch {
        /// Suppress output (for shell hooks)
        #[arg(short, long)]
        quiet: bool,
    },
}

fn parse_tag(s: &str) -> Result<String, String> {
    s.strip_prefix('+')
        .map(String::from)
        .ok_or_else(|| format!("tags must start with +: {s}"))
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| format!("invalid date format, expected YYYY-MM-DD: {s}"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut conn = db::open()?;

    match cli.command {
        Commands::Start { project, tags } => commands::start(&conn, &project, &tags),
        Commands::Stop => commands::stop(&conn),
        Commands::Status => commands::status(&conn),
        Commands::Log { from, to } => commands::log(&conn, from, to),
        Commands::Cancel => commands::cancel(&conn),
        Commands::Delete { id } => commands::delete(&conn, id),
        Commands::Projects => commands::projects(&conn),
        Commands::Tags => commands::tags(&conn),
        Commands::Report { from, to, by_tag } => commands::report(&conn, from, to, by_tag),
        Commands::Edit {
            id,
            project,
            tags,
            start,
            end,
        } => commands::edit(&conn, id, project, tags, start, end),
        Commands::Restart => commands::restart(&conn),
        Commands::Export { format } => commands::export(&conn, format),
        Commands::Completions { shell } => commands::completions(shell),
        Commands::Switch { quiet } => commands::switch(&mut conn, quiet),
    }
}
