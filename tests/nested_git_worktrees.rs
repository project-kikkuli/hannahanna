mod common;

use common::TestRepo;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn hn_in(directory: &Path, args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_hn"))
        .args(args)
        .current_dir(directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn commands_discover_the_repository_from_worktree_subdirectories() {
    let repo = TestRepo::new();
    repo.hn(&["add", "parent"]).assert_success();
    let nested = repo.worktree_path("parent").join("src/deep");
    std::fs::create_dir_all(&nested).unwrap();
    hn_in(&nested, &["info", "--format=json"]);
}

#[test]
fn nested_creation_uses_sibling_directories_and_independent_parent_links() {
    let repo = TestRepo::new();
    repo.hn(&["add", "parent-a"]).assert_success();
    repo.hn(&["add", "parent-b"]).assert_success();
    hn_in(&repo.worktree_path("parent-a"), &["add", "child-a"]);
    assert!(
        repo.worktree_path("child-a").join(".git").is_file(),
        "child must be a sibling worktree, not a directory inside Git's metadata"
    );
    hn_in(&repo.worktree_path("parent-b"), &["add", "child-b"]);

    let result = repo.hn(&["list", "--format=json"]);
    result.assert_success();
    let list: Vec<Value> = serde_json::from_str(&result.stdout).unwrap();
    for item in list {
        let expected = match item["name"].as_str().unwrap() {
            "child-a" => serde_json::json!("parent-a"),
            "child-b" => serde_json::json!("parent-b"),
            _ => Value::Null,
        };
        assert_eq!(
            item["parent"], expected,
            "wrong parent for {}",
            item["name"]
        );
    }
    assert!(
        !repo
            .git(&["config", "--local", "--get", "worktree.parent"])
            .success
    );
}

#[test]
fn nested_child_can_merge_and_return_to_its_actual_parent() {
    let repo = TestRepo::new();
    repo.hn(&["add", "parent"]).assert_success();
    hn_in(&repo.worktree_path("parent"), &["add", "child"]);
    std::fs::write(
        repo.worktree_path("child").join("change.txt"),
        "child contribution\n",
    )
    .unwrap();
    repo.git_in_worktree("child", &["add", "change.txt"])
        .assert_success();
    repo.git_in_worktree("child", &["commit", "-m", "Child contribution"])
        .assert_success();
    hn_in(
        &repo.worktree_path("child"),
        &["return", "--merge", "--delete"],
    );
    assert!(!repo.worktree_path("child").exists());
    assert_eq!(
        std::fs::read_to_string(repo.worktree_path("parent").join("change.txt")).unwrap(),
        "child contribution\n"
    );
    assert!(!repo.path().join("change.txt").exists());
}
