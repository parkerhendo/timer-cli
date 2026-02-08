use anyhow::Result;
use rusqlite::Connection;

pub fn run(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT project FROM frames ORDER BY project",
    )?;

    let projects: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    if projects.is_empty() {
        println!("No projects found");
    } else {
        for project in projects {
            println!("{}", project);
        }
    }

    Ok(())
}
