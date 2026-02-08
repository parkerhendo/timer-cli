use std::process::Command;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDb {
    path: PathBuf,
}

impl TestDb {
    fn new() -> Self {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!("timer-cli-test-{}-{}.db", std::process::id(), id));
        // Ensure clean slate
        let _ = fs::remove_file(&path);
        Self { path }
    }

    fn cli(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_timer-cli"));
        cmd.env("TIMER_CLI_DB", &self.path);
        cmd
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn test_status_not_tracking() {
    let db = TestDb::new();
    let output = db.cli().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Not tracking"));
}

#[test]
fn test_start_stop_flow() {
    let db = TestDb::new();

    // Start
    let output = db.cli()
        .args(["start", "myproject", "+rust"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Started myproject +rust"));

    // Status shows tracking
    let output = db.cli().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("myproject +rust"));

    // Stop
    let output = db.cli().arg("stop").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Stopped myproject +rust"));

    // Status shows not tracking
    let output = db.cli().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Not tracking"));
}

#[test]
fn test_double_start_fails() {
    let db = TestDb::new();

    // First start
    let _ = db.cli()
        .args(["start", "project1"])
        .output()
        .expect("failed to run");

    // Second start should fail
    let output = db.cli()
        .args(["start", "project2"])
        .output()
        .expect("failed to run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already tracking"));
}

#[test]
fn test_stop_when_not_tracking_fails() {
    let db = TestDb::new();

    let output = db.cli().arg("stop").output().expect("failed to run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not tracking"));
}

#[test]
fn test_cancel() {
    let db = TestDb::new();

    // Start
    let _ = db.cli()
        .args(["start", "tocancel"])
        .output()
        .expect("failed to run");

    // Cancel
    let output = db.cli().arg("cancel").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cancelled tocancel"));

    // Status shows not tracking
    let output = db.cli().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Not tracking"));
}

#[test]
fn test_projects_and_tags() {
    let db = TestDb::new();

    // Create some frames
    let _ = db.cli().args(["start", "proj1", "+tag1"]).output();
    let _ = db.cli().arg("stop").output();
    let _ = db.cli().args(["start", "proj2", "+tag2"]).output();
    let _ = db.cli().arg("stop").output();

    // Projects
    let output = db.cli().arg("projects").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj1"));
    assert!(stdout.contains("proj2"));

    // Tags
    let output = db.cli().arg("tags").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("+tag1"));
    assert!(stdout.contains("+tag2"));
}

#[test]
fn test_restart() {
    let db = TestDb::new();

    // Create and stop a frame
    let _ = db.cli().args(["start", "lastproj", "+lasttag"]).output();
    let _ = db.cli().arg("stop").output();

    // Restart
    let output = db.cli().arg("restart").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Started lastproj +lasttag"));

    // Status confirms
    let output = db.cli().arg("status").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("lastproj +lasttag"));
}

#[test]
fn test_tag_parsing() {
    let db = TestDb::new();

    // Valid tags
    let output = db.cli()
        .args(["start", "proj", "+valid"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let _ = db.cli().arg("stop").output();

    // Invalid tag (no +)
    let output = db.cli()
        .args(["start", "proj", "invalid"])
        .output()
        .expect("failed to run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("tags must start with +"));
}

#[test]
fn test_delete() {
    let db = TestDb::new();

    // Create a frame
    let _ = db.cli().args(["start", "todelete"]).output();
    let _ = db.cli().arg("stop").output();

    // Delete it
    let output = db.cli().args(["delete", "1"]).output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Deleted frame 1"));

    // Verify it's gone
    let output = db.cli().arg("log").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No frames found"));
}

#[test]
fn test_delete_nonexistent_fails() {
    let db = TestDb::new();

    let output = db.cli().args(["delete", "999"]).output().expect("failed to run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("frame 999 not found"));
}

#[test]
fn test_log() {
    let db = TestDb::new();

    // Create frames
    let _ = db.cli().args(["start", "proj1", "+tag1"]).output();
    let _ = db.cli().arg("stop").output();

    let output = db.cli().arg("log").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj1"));
    assert!(stdout.contains("+tag1"));
}

#[test]
fn test_report() {
    let db = TestDb::new();

    // Create frames
    let _ = db.cli().args(["start", "proj1"]).output();
    let _ = db.cli().arg("stop").output();

    let output = db.cli().arg("report").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj1"));
    assert!(stdout.contains("Total:"));
}

#[test]
fn test_edit() {
    let db = TestDb::new();

    // Create frame
    let _ = db.cli().args(["start", "original"]).output();
    let _ = db.cli().arg("stop").output();

    // Edit project name
    let output = db.cli()
        .args(["edit", "1", "--project", "renamed"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());

    // Verify
    let output = db.cli().arg("log").output().expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("renamed"));
    assert!(!stdout.contains("original"));
}

#[test]
fn test_export_json() {
    let db = TestDb::new();

    // Create a frame
    let _ = db.cli().args(["start", "exporttest", "+tag1"]).output();
    let _ = db.cli().arg("stop").output();

    let output = db.cli()
        .args(["export", "--format", "json"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"project\": \"exporttest\""));
    assert!(stdout.contains("\"tags\""));
    assert!(stdout.contains("duration_seconds"));
}

#[test]
fn test_export_csv() {
    let db = TestDb::new();

    // Create a frame
    let _ = db.cli().args(["start", "csvtest"]).output();
    let _ = db.cli().arg("stop").output();

    let output = db.cli()
        .args(["export", "--format", "csv"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("id,project,start_time"));
    assert!(stdout.contains("csvtest"));
}

#[test]
fn test_completions() {
    let db = TestDb::new();

    let output = db.cli()
        .args(["completions", "bash"])
        .output()
        .expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("timer-cli"));
}

#[test]
fn test_timer_alias() {
    let db = TestDb::new();

    // Use timer binary
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_timer"));
    cmd.env("TIMER_CLI_DB", &db.path);

    let output = cmd.arg("status").output().expect("failed to run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Not tracking"));
}
