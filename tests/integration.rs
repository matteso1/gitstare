use std::path::{Path, PathBuf};
use std::process::Command;

/// Helper: create a temp directory with a unique name
fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gitstare_test_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper: run a git command in a directory
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("failed to run git");
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        // Some commands (like checkout -b on existing branch) fail acceptably
        if !stderr.contains("already exists") {
            panic!(
                "git {:?} failed in {}: {}",
                args,
                dir.display(),
                stderr
            );
        }
    }
}

/// Helper: create a file and commit it
fn commit_file(dir: &Path, filename: &str, content: &str, msg: &str) {
    std::fs::write(dir.join(filename), content).unwrap();
    git(dir, &["add", filename]);
    git(dir, &["commit", "-m", msg]);
}

/// Helper: init a repo with an initial commit
fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-b", "main"]);
    commit_file(dir, "README.md", "# test", "Initial commit");
}

// ============================================================================
// SCANNER TESTS
// ============================================================================

#[test]
fn scanner_finds_repos() {
    let root = tmp_dir("scanner_finds");
    init_repo(&root.join("repo-a"));
    init_repo(&root.join("repo-b"));
    init_repo(&root.join("subdir/repo-c"));

    let repos = gitstare::scanner::scan(&[root.clone()], 4, &[]);
    let names: Vec<String> = repos
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert!(names.contains(&"repo-a".to_string()));
    assert!(names.contains(&"repo-b".to_string()));
    assert!(names.contains(&"repo-c".to_string()));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn scanner_respects_depth_limit() {
    let root = tmp_dir("scanner_depth");
    init_repo(&root.join("a/b/c/d/deep-repo"));

    // depth 2 should not find it (a/b/c/d/deep-repo/.git is 5 levels deep)
    let repos = gitstare::scanner::scan(&[root.clone()], 2, &[]);
    let names: Vec<String> = repos
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(!names.contains(&"deep-repo".to_string()));

    // depth 6 should find it
    let repos = gitstare::scanner::scan(&[root.clone()], 6, &[]);
    let names: Vec<String> = repos
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(names.contains(&"deep-repo".to_string()));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn scanner_ignores_patterns() {
    let root = tmp_dir("scanner_ignore");
    init_repo(&root.join("good-repo"));
    init_repo(&root.join("node_modules/bad-repo"));
    init_repo(&root.join("target/another-bad"));

    let repos = gitstare::scanner::scan(
        &[root.clone()],
        4,
        &["node_modules".into(), "target".into()],
    );
    let names: Vec<String> = repos
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert!(names.contains(&"good-repo".to_string()));
    assert!(!names.contains(&"bad-repo".to_string()));
    assert!(!names.contains(&"another-bad".to_string()));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn scanner_handles_empty_input() {
    let repos = gitstare::scanner::scan(&[], 4, &[]);
    assert!(repos.is_empty());
}

#[test]
fn scanner_handles_nonexistent_path() {
    let repos = gitstare::scanner::scan(
        &[PathBuf::from("/nonexistent/path/that/doesnt/exist")],
        4,
        &[],
    );
    assert!(repos.is_empty());
}

// ============================================================================
// GIT READER TESTS
// ============================================================================

#[test]
fn git_reads_clean_repo() {
    let root = tmp_dir("git_clean");
    init_repo(&root);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    assert_eq!(repos.len(), 1);

    let repo = &repos[0];
    assert_eq!(repo.branch, "main");
    assert!(repo.is_clean());
    assert_eq!(repo.modified, 0);
    assert_eq!(repo.untracked, 0);
    assert!(repo.last_commit_ts.is_some());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_reads_dirty_repo() {
    let root = tmp_dir("git_dirty");
    init_repo(&root);

    // Create a modified tracked file
    std::fs::write(root.join("README.md"), "modified content").unwrap();
    // Create an untracked file
    std::fs::write(root.join("new_file.txt"), "untracked").unwrap();

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];

    assert!(!repo.is_clean());
    assert!(repo.modified > 0 || repo.untracked > 0);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_reads_detached_head() {
    let root = tmp_dir("git_detached");
    init_repo(&root);
    commit_file(&root, "second.txt", "second", "Second commit");

    // Detach HEAD at first commit
    let output = Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    let first = String::from_utf8(output.stdout).unwrap().trim().to_string();
    git(&root, &["checkout", &first]);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];
    assert!(repo.branch.starts_with("detached"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_reads_multiple_branches() {
    let root = tmp_dir("git_branches");
    init_repo(&root);
    git(&root, &["checkout", "-b", "feature-a"]);
    commit_file(&root, "a.txt", "a", "Feature A");
    git(&root, &["checkout", "main"]);
    git(&root, &["checkout", "-b", "feature-b"]);
    commit_file(&root, "b.txt", "b", "Feature B");
    git(&root, &["checkout", "main"]);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];

    assert!(repo.branches.len() >= 3); // main, feature-a, feature-b
    let branch_names: Vec<&str> = repo.branches.iter().map(|b| b.name.as_str()).collect();
    assert!(branch_names.contains(&"main"));
    assert!(branch_names.contains(&"feature-a"));
    assert!(branch_names.contains(&"feature-b"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_counts_stale_branches() {
    let root = tmp_dir("git_stale");
    init_repo(&root);
    git(&root, &["checkout", "-b", "old-branch"]);

    // Create a backdated commit
    let out = Command::new("git")
        .args(["commit", "--allow-empty", "-m", "old commit"])
        .current_dir(&root)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .env("GIT_AUTHOR_DATE", "2020-01-01T00:00:00")
        .env("GIT_COMMITTER_DATE", "2020-01-01T00:00:00")
        .output()
        .unwrap();
    assert!(out.status.success());

    git(&root, &["checkout", "main"]);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];

    // old-branch should be stale (30 day threshold, commit is from 2020)
    assert!(repo.stale_branches > 0);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_reads_recent_commits() {
    let root = tmp_dir("git_commits");
    init_repo(&root);
    for i in 1..=5 {
        commit_file(
            &root,
            &format!("file{i}.txt"),
            &format!("content {i}"),
            &format!("Commit number {i}"),
        );
    }

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];

    // Should have 6 commits total (1 initial + 5 more)
    assert_eq!(repo.recent_commits.len(), 6);
    // Most recent first
    assert!(repo.recent_commits[0].message.contains("Commit number 5"));

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_handles_empty_repo() {
    let root = tmp_dir("git_empty");
    std::fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-b", "main"]);

    // Should not panic, just skip the empty repo
    let repos = gitstare::git::read_all(&[root.clone()], 30);
    // Empty repo might fail to read (no HEAD), so either 0 results or a valid entry
    // If it does read, the branch will be "HEAD" or "main" depending on git2 behavior
    if !repos.is_empty() {
        let repo = &repos[0];
        assert!(
            repo.branch == "main" || repo.branch == "HEAD",
            "Expected 'main' or 'HEAD', got: {}",
            repo.branch
        );
    }

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_reads_ahead_behind() {
    let root = tmp_dir("git_ahead");

    // Create bare remote
    let bare = root.join("remote.git");
    std::fs::create_dir_all(&bare).unwrap();
    Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .current_dir(&bare)
        .output()
        .unwrap();

    // Create local repo
    let local = root.join("local");
    init_repo(&local);
    git(&local, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git(&local, &["push", "-u", "origin", "main"]);

    // Make local commits (ahead)
    commit_file(&local, "local.txt", "local", "Local work");

    let repos = gitstare::git::read_all(&[local.clone()], 30);
    let repo = &repos[0];
    assert_eq!(repo.ahead, 1);
    assert_eq!(repo.behind, 0);

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn git_status_string_formats() {
    let root = tmp_dir("git_status_str");
    init_repo(&root);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];
    assert_eq!(repo.status_string(), "clean");
    assert_eq!(repo.ahead_behind_string(), "+0/-0");

    std::fs::remove_dir_all(&root).ok();
}

// ============================================================================
// CACHE TESTS
// ============================================================================

#[test]
fn cache_round_trips() {
    // Create a repo to get real RepoInfo data
    let root = tmp_dir("cache_rt");
    init_repo(&root);
    commit_file(&root, "test.txt", "test", "Test commit");

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    assert!(!repos.is_empty());

    let cache = gitstare::cache::Cache::open().unwrap();
    cache.save(&repos).unwrap();

    let loaded = cache.load().unwrap();
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.len(), repos.len());
    assert_eq!(loaded[0].name, repos[0].name);
    assert_eq!(loaded[0].branch, repos[0].branch);

    std::fs::remove_dir_all(&root).ok();
}

// ============================================================================
// CONFIG TESTS
// ============================================================================

#[test]
fn config_loads_defaults() {
    let cfg = gitstare::config::Config::load().unwrap();
    assert!(!cfg.scan_paths.is_empty());
    assert!(cfg.max_depth > 0);
    assert!(!cfg.ignore.is_empty());
    assert!(cfg.stale_threshold > 0);
}

// ============================================================================
// RELATIVE TIME TESTS
// ============================================================================

#[test]
fn relative_time_formatting() {
    let root = tmp_dir("git_reltime");
    init_repo(&root);

    let repos = gitstare::git::read_all(&[root.clone()], 30);
    let repo = &repos[0];

    // Just committed, should say something like "now" or "Xm ago" or "Xs ago"
    let rel = repo.last_commit_relative();
    assert!(
        rel.contains("now") || rel.contains("m ago") || rel.contains("just"),
        "Expected recent timestamp, got: {}",
        rel
    );

    std::fs::remove_dir_all(&root).ok();
}

// ============================================================================
// PARALLEL SCANNING STRESS TEST
// ============================================================================

#[test]
fn scanner_handles_many_repos() {
    let root = tmp_dir("scanner_stress");

    // Create 20 repos
    for i in 0..20 {
        let repo_dir = root.join(format!("repo-{i:03}"));
        init_repo(&repo_dir);
    }

    let repos = gitstare::scanner::scan(&[root.clone()], 4, &[]);
    assert_eq!(repos.len(), 20);

    let infos = gitstare::git::read_all(&repos, 30);
    assert_eq!(infos.len(), 20);

    // All should be clean
    for info in &infos {
        assert!(info.is_clean());
        assert_eq!(info.branch, "main");
    }

    std::fs::remove_dir_all(&root).ok();
}
