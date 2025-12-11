use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn cargo_bin() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--");
    cmd
}

#[test]
fn test_help_and_version() {
    let mut help = cargo_bin();
    help.arg("--help");
    help.assert()
        .success()
        .stdout(predicate::str::contains("Usage: sandbag"));

    let mut version = cargo_bin();
    version.arg("--version");
    version
        .assert()
        .success()
        .stdout(predicate::str::contains("sandbag "));
}

#[test]
fn test_list_commands() {
    let mut list = cargo_bin();
    list.arg("list");
    list.assert()
        .success()
        .stdout(predicate::str::contains("Supported linters:"));

    let mut list_detailed = cargo_bin();
    list_detailed.args(["list", "--detailed"]);
    list_detailed
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(detailed info not yet implemented)",
        ));
}

#[test]
fn test_add_dry_run_common_rules() {
    // markdownlint
    let mut add_md = cargo_bin();
    add_md.args(["add", "MD033: No inline HTML", "--dry-run"]);
    add_md
        .assert()
        .success()
        .stdout(predicate::str::contains("Rule: MD033"))
        .stdout(predicate::str::contains("DRY RUN"));

    // eslint
    let mut add_eslint = cargo_bin();
    add_eslint.args([
        "add",
        "1:10 error no-unused-vars x is assigned a value but never used",
        "--dry-run",
    ]);
    add_eslint
        .assert()
        .success()
        .stdout(predicate::str::contains("Rule: no-unused-vars"));

    // prettier
    let mut add_prettier = cargo_bin();
    add_prettier.args(["add", "prettier/prettier: Missing semicolon", "--dry-run"]);
    add_prettier
        .assert()
        .success()
        .stdout(predicate::str::contains("Rule: prettier/prettier"));
}

#[test]
fn test_scan_smoke() {
    let mut scan = cargo_bin();
    scan.args(["scan", "--limit", "2"]);
    scan.assert()
        .success()
        .stdout(predicate::str::contains("Top"));
}

#[test]
fn test_info_md033() {
    let mut info = cargo_bin();
    info.args(["info", "MD033"]);
    info.assert()
        .success()
        .stdout(predicate::str::contains("Description: No inline HTML"));
}

#[test]
fn test_batch_dry_run() {
    let path = "/tmp/sandbag_batch_smoke.txt";
    std::fs::write(
        path,
        "MD033: No inline HTML\n1:10 error no-unused-vars x is assigned a value but never used\n",
    )
    .unwrap();

    let mut batch = cargo_bin();
    batch.args(["batch", path, "--dry-run"]);
    batch
        .assert()
        .success()
        .stdout(predicate::str::contains("Rule: MD033"))
        .stdout(predicate::str::contains("Rule: no-unused-vars"));
}

#[test]
fn test_add_real_apply_markdownlint() {
    // Run in the current workspace so cargo is available; write config locally then clean it up
    let config = std::path::Path::new(".markdownlint.json");
    let original = std::fs::read_to_string(config).ok();
    std::fs::write(config, "{\n  \"default\": true\n}\n").unwrap();

    let mut cmd = cargo_bin();
    cmd.args(["add", "MD033: No inline HTML"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(config).unwrap();
    assert!(content.contains("\"MD033\": false"));

    // Restore previous content if any
    if let Some(orig) = original {
        let _ = std::fs::write(config, orig);
    }
}

#[test]
fn test_add_real_apply_eslint() {
    // Write a minimal .eslintrc.json, apply no-console off
    let config = std::path::Path::new(".eslintrc.json");
    let original = std::fs::read_to_string(config).ok();
    std::fs::write(config, "{\n  \"rules\": {}\n}\n").unwrap();

    let mut cmd = crate::cargo_bin();
    cmd.args(["add", "1:1 error no-console unexpected console statement"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(config).unwrap();
    assert!(content.contains("\"no-console\": \"off\""));

    if let Some(orig) = original {
        let _ = std::fs::write(config, orig);
    }
}

#[test]
fn test_add_real_apply_prettier() {
    let config = std::path::Path::new(".prettierrc.json");
    let original = std::fs::read_to_string(config).ok();
    std::fs::write(config, "{\n}\n").unwrap();

    let mut cmd = crate::cargo_bin();
    cmd.args(["add", "prettier/prettier: Missing semicolon"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(config).unwrap();
    assert!(
        content.contains("\"semi\": false") || content.contains("Handled rule prettier/prettier")
    );

    if let Some(orig) = original {
        let _ = std::fs::write(config, orig);
    }
}
