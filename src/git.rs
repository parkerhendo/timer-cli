use std::path::Path;
use std::process::Command;

/// Returns (repo_name, branch) if in a git repository, None otherwise.
pub fn get_context() -> Option<(String, String)> {
    let toplevel = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !toplevel.status.success() {
        return None;
    }

    let repo_path = String::from_utf8_lossy(&toplevel.stdout);
    let repo_name = Path::new(repo_path.trim())
        .file_name()?
        .to_str()?
        .to_string();

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if !branch_output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // TODO: Handle detached HEAD - currently returns "HEAD" as the tag
    // Could use short SHA instead: git rev-parse --short HEAD

    Some((repo_name, branch))
}
