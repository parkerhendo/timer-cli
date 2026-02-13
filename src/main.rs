mod commands;
mod db;
mod frame;
mod git;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
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
        /// Date or date range (YYYY-MM-DD, MM-DD, today, yesterday). One date shows that day, two dates define a range.
        #[arg(value_parser = parse_date_flexible)]
        dates: Vec<NaiveDate>,
        /// Start date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        from: Option<NaiveDate>,
        /// End date (YYYY-MM-DD)
        #[arg(short, long, value_parser = parse_date)]
        to: Option<NaiveDate>,
        /// Show all entries (ignore date range)
        #[arg(short, long)]
        all: bool,
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
        /// Show all entries (ignore date range)
        #[arg(short, long)]
        all: bool,
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

/// Parse a date string relative to `today`.
/// Accepts: "today", "yesterday", YYYY-MM-DD, MM-DD, or M-D.
/// When year is omitted, uses the year from `today`.
fn parse_date_relative(s: &str, today: NaiveDate) -> Result<NaiveDate, String> {
    // Relative names
    match s.to_lowercase().as_str() {
        "today" => return Ok(today),
        "yesterday" => {
            return today
                .pred_opt()
                .ok_or_else(|| "cannot represent yesterday".to_string());
        }
        _ => {}
    }
    // Try full date first (YYYY-MM-DD)
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }
    // Try month-day without year (MM-DD or M-D)
    let default_year = today.year();
    let with_year = format!("{default_year}-{s}");
    NaiveDate::parse_from_str(&with_year, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(&with_year, "%Y-%-m-%-d"))
        .map_err(|_| format!("invalid date format, expected YYYY-MM-DD, MM-DD, today, or yesterday: {s}"))
}

/// Entry point for clap's value_parser.
fn parse_date_flexible(s: &str) -> Result<NaiveDate, String> {
    parse_date_relative(s, Local::now().date_naive())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut conn = db::open()?;

    match cli.command {
        Commands::Start { project, tags } => commands::start(&conn, &project, &tags),
        Commands::Stop => commands::stop(&conn),
        Commands::Status => commands::status(&conn),
        Commands::Log { dates, from, to, all } => {
            let (from, to) = match dates.len() {
                0 => (from, to),
                1 => (Some(from.unwrap_or(dates[0])), Some(to.unwrap_or(dates[0]))),
                2 => (Some(from.unwrap_or(dates[0])), Some(to.unwrap_or(dates[1]))),
                _ => {
                    anyhow::bail!("too many date arguments (expected at most 2)");
                }
            };
            commands::log(&conn, from, to, all)
        }
        Commands::Cancel => commands::cancel(&conn),
        Commands::Delete { id } => commands::delete(&conn, id),
        Commands::Projects => commands::projects(&conn),
        Commands::Tags => commands::tags(&conn),
        Commands::Report { from, to, by_tag, all } => commands::report(&conn, from, to, by_tag, all),
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()
    }

    #[test]
    fn parse_full_date() {
        let d = parse_date_relative("2025-01-15", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn parse_month_day_zero_padded() {
        let d = parse_date_relative("01-15", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
    }

    #[test]
    fn parse_month_day_no_padding() {
        let d = parse_date_relative("1-5", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 5).unwrap());
    }

    #[test]
    fn parse_month_day_mixed_padding() {
        let d = parse_date_relative("1-15", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
    }

    #[test]
    fn parse_full_date_ignores_default_year() {
        let d = parse_date_relative("2024-12-25", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2024, 12, 25).unwrap());
    }

    #[test]
    fn parse_today() {
        let d = parse_date_relative("today", today()).unwrap();
        assert_eq!(d, today());
    }

    #[test]
    fn parse_today_case_insensitive() {
        let d = parse_date_relative("Today", today()).unwrap();
        assert_eq!(d, today());
    }

    #[test]
    fn parse_yesterday() {
        let d = parse_date_relative("yesterday", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 14).unwrap());
    }

    #[test]
    fn parse_yesterday_case_insensitive() {
        let d = parse_date_relative("Yesterday", today()).unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 14).unwrap());
    }

    #[test]
    fn parse_invalid_date_returns_error() {
        assert!(parse_date_relative("not-a-date", today()).is_err());
        assert!(parse_date_relative("13-32", today()).is_err());
        assert!(parse_date_relative("", today()).is_err());
    }
}
