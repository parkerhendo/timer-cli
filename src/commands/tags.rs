use anyhow::Result;
use rusqlite::Connection;
use std::collections::BTreeSet;

pub fn run(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT DISTINCT tags FROM frames WHERE tags IS NOT NULL")?;

    let all_tags: BTreeSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .flat_map(|tags_str| {
            tags_str
                .split(',')
                .map(String::from)
                .collect::<Vec<_>>()
        })
        .collect();

    if all_tags.is_empty() {
        println!("No tags found");
    } else {
        for tag in all_tags {
            println!("+{}", tag);
        }
    }

    Ok(())
}
