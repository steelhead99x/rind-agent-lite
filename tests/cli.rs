//! The `rind-pin-check` binary against this repository and a scratch tree.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SHA: &str = "acc61b3892ceb6afecfa96c94132bf6e35bbf1fb";

fn exe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rind-pin-check"))
}

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rind-pin-check-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn checks_this_repository_from_a_subdirectory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = exe()
        .current_dir(root.join("skills/build"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, format!("pin {SHA} acc61b3 kixel-ai/rind\n"));
}

#[test]
fn lists_citations_for_an_explicit_directory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = exe().arg("list").arg(&root).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(&format!(
        "skills/bench/SKILL.md:18 commit kixel-ai/rind {SHA}"
    )));
    assert!(stdout.ends_with(&format!("pin {SHA} acc61b3 kixel-ai/rind\n")));
}

#[test]
fn fails_on_a_short_commit_link() {
    let dir = scratch();
    fs::write(
        dir.join("README.md"),
        format!("[`acc61b3`](https://github.com/kixel-ai/rind/commit/{SHA})\n"),
    )
    .unwrap();
    fs::create_dir(dir.join("skills")).unwrap();
    fs::write(
        dir.join("skills/note.md"),
        "https://github.com/kixel-ai/rind/blob/main/AGENTS.md\n",
    )
    .unwrap();
    let output = exe().arg(&dir).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("skills/note.md:1:"));
    assert!(stderr.contains("not a full lowercase commit SHA"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn usage_and_a_missing_repository() {
    let help = exe().arg("--help").output().unwrap();
    assert!(help.status.success());
    let help_out = String::from_utf8(help.stdout).unwrap();
    assert!(help_out.contains("rind-pin-check list"));

    let unknown = exe().arg("list").arg("extra").arg("nope").output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8(unknown.stderr)
        .unwrap()
        .contains("usage:"));

    let dir = scratch();
    let missing = exe().current_dir(&dir).output().unwrap();
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8(missing.stderr)
        .unwrap()
        .contains("README.md and skills/"));
    let _ = fs::remove_dir_all(&dir);
}
