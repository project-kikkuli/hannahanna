use std::path::Path;
use std::process::{Command, Output};

fn run(program: &str, args: &[&str], repo: &Path, config: &Path) -> Output {
    Command::new(program)
        .args(args)
        .current_dir(repo)
        .env("HGRCPATH", config)
        .env("HGPLAIN", "1")
        .output()
        .unwrap()
}

fn success(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn mercurial_cli_creates_inspects_and_safely_removes_a_share() {
    if Command::new("hg").arg("--version").output().is_err() {
        eprintln!("Mercurial CLI check requires hg on PATH");
        return;
    }
    let temporary = tempfile::tempdir().unwrap();
    let repo = temporary.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    let config = temporary.path().join("hgrc");
    std::fs::write(
        &config,
        "[ui]\nusername = Synthetic Test <test@example.invalid>\n[extensions]\nshare =\n",
    )
    .unwrap();
    success(run("hg", &["init"], &repo, &config));
    std::fs::write(repo.join("sample.txt"), "synthetic original\n").unwrap();
    success(run("hg", &["add", "sample.txt"], &repo, &config));
    success(run("hg", &["commit", "-m", "Fixture"], &repo, &config));

    let hn = env!("CARGO_BIN_EXE_hn");
    success(run(hn, &["add", "child"], &repo, &config));
    let child = temporary.path().join("child");
    assert_eq!(
        std::fs::read_to_string(child.join("sample.txt")).unwrap(),
        "synthetic original\n"
    );
    assert!(success(run(hn, &["list"], &repo, &config)).contains("child"));
    assert!(success(run(hn, &["info", "child"], &repo, &config)).contains("mercurial"));

    std::fs::write(child.join("sample.txt"), "synthetic changed\n").unwrap();
    assert!(!run(hn, &["remove", "child"], &repo, &config)
        .status
        .success());
    assert!(child.exists());
    success(run(
        "hg",
        &["commit", "-m", "Child contribution"],
        &child,
        &config,
    ));
    success(run(hn, &["remove", "child"], &repo, &config));
    assert!(!child.exists());
    let log = success(run(
        "hg",
        &["log", "-r", "tip", "--template", "{desc}"],
        &repo,
        &config,
    ));
    assert_eq!(log, "Child contribution");
}
