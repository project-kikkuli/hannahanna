mod common;

use common::{CommandResult, TestRepo};
use serde_json::Value;

fn json(result: CommandResult) -> Value {
    assert!(result.success, "{}", result.stderr);
    serde_json::from_str(&result.stdout).expect("stdout must contain only JSON")
}

#[test]
fn machine_output_supports_the_companion_session_lifecycle() {
    let repo = TestRepo::new();
    repo.create_config("hooks:\n  post_create: echo hook-output\n");
    let created = json(repo.hn(&["add", "feature", "--from", "main", "--format=json"]));
    assert_eq!(created["name"], "feature");
    assert_eq!(created["branch"], "feature");
    assert_eq!(created["vcs_type"], "git");
    assert_eq!(created["commit"].as_str().unwrap().len(), 40);
    let path = std::path::Path::new(created["path"].as_str().unwrap());
    assert_eq!(
        path.canonicalize().unwrap(),
        repo.worktree_path("feature").canonicalize().unwrap()
    );

    let info = json(repo.hn(&["info", "feature", "--format=json"]));
    assert_eq!(created, info);
    let list = json(repo.hn(&["list", "--format=json"]));
    assert!(list.as_array().unwrap().contains(&info));

    let removed = repo.hn(&["remove", "feature"]);
    assert!(removed.success, "{}", removed.stderr);
    let list = json(repo.hn(&["list", "--format=json"]));
    assert_eq!(list.as_array().unwrap().len(), 1);
    let missing = repo.hn(&["info", "feature", "--format=json"]);
    assert!(!missing.success);
    assert!(missing.stdout.is_empty());
}

#[test]
fn machine_list_is_current_and_empty_tag_filter_is_an_array() {
    let repo = TestRepo::new();
    json(repo.hn(&["list", "--format=json"]));
    repo.create_and_commit("new.txt", "new", "new commit");
    let list = json(repo.hn(&["list", "--format=json"]));
    assert_eq!(
        list[0]["commit"],
        repo.git(&["rev-parse", "HEAD"]).stdout.trim()
    );
    assert_eq!(
        json(repo.hn(&["list", "--tag", "missing", "--format=json"])),
        serde_json::json!([])
    );
}

#[test]
fn exact_removal_does_not_delete_a_similarly_named_worktree() {
    let repo = TestRepo::new();
    repo.hn(&["add", "task-other"]).assert_success();
    let result = repo.hn(&["remove", "task", "--exact"]);
    assert!(!result.success);
    assert!(repo.worktree_path("task-other").exists());
    repo.hn(&["remove", "task-other", "--exact"])
        .assert_success();
    assert!(!repo.worktree_path("task-other").exists());
}
