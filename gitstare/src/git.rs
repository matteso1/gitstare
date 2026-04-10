use chrono::{DateTime, Utc};
use git2::{BranchType, Repository, StatusOptions};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: PathBuf,
    pub name: String,
    pub branch: String,
    pub modified: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
    pub last_commit_ts: Option<i64>,
    pub stale_branches: usize,
    pub branches: Vec<BranchInfo>,
    pub recent_commits: Vec<CommitInfo>,
    pub remotes: Vec<String>,
    pub diff_stat: DiffStat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub last_commit_ts: Option<i64>,
    pub last_commit_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStat {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl RepoInfo {
    pub fn is_clean(&self) -> bool {
        self.modified == 0 && self.untracked == 0
    }

    pub fn status_string(&self) -> String {
        if self.is_clean() {
            "clean".into()
        } else {
            let mut parts = Vec::new();
            if self.modified > 0 {
                parts.push(format!("{}M", self.modified));
            }
            if self.untracked > 0 {
                parts.push(format!("{}U", self.untracked));
            }
            parts.join(" ")
        }
    }

    pub fn last_commit_relative(&self) -> String {
        match self.last_commit_ts {
            Some(ts) => {
                let commit_time = DateTime::from_timestamp(ts, 0).unwrap_or_default();
                let now = Utc::now();
                let dur = now.signed_duration_since(commit_time);

                if dur.num_minutes() < 1 {
                    "just now".into()
                } else if dur.num_minutes() < 60 {
                    format!("{}m ago", dur.num_minutes())
                } else if dur.num_hours() < 24 {
                    format!("{}h ago", dur.num_hours())
                } else if dur.num_days() < 30 {
                    format!("{}d ago", dur.num_days())
                } else if dur.num_days() < 365 {
                    format!("{}mo ago", dur.num_days() / 30)
                } else {
                    format!("{}y ago", dur.num_days() / 365)
                }
            }
            None => "never".into(),
        }
    }

    pub fn ahead_behind_string(&self) -> String {
        format!("+{}/-{}", self.ahead, self.behind)
    }
}

pub fn read_all(paths: &[PathBuf], stale_days: u64) -> Vec<RepoInfo> {
    paths
        .par_iter()
        .filter_map(|p| read_repo(p, stale_days).ok())
        .collect()
}

fn read_repo(path: &PathBuf, stale_days: u64) -> Result<RepoInfo, git2::Error> {
    let repo = Repository::open(path)?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let branch = current_branch(&repo);
    let (modified, untracked) = count_status(&repo);
    let (ahead, behind) = ahead_behind(&repo);
    let last_commit_ts = last_commit_timestamp(&repo);
    let branches = list_branches(&repo);
    let stale_branches = count_stale(&branches, stale_days);
    let recent_commits = recent_commits(&repo, 20);
    let remotes = list_remotes(&repo);
    let diff_stat = get_diff_stat(&repo);

    Ok(RepoInfo {
        path: path.clone(),
        name,
        branch,
        modified,
        untracked,
        ahead,
        behind,
        last_commit_ts,
        stale_branches,
        branches,
        recent_commits,
        remotes,
        diff_stat,
    })
}

fn current_branch(repo: &Repository) -> String {
    if repo.head_detached().unwrap_or(false) {
        if let Ok(head) = repo.head() {
            let oid = head.target().unwrap_or_else(git2::Oid::zero);
            return format!("detached@{}", &oid.to_string()[..7]);
        }
        return "detached".into();
    }
    repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".into())
}

fn count_status(repo: &Repository) -> (usize, usize) {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };

    let mut modified = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let s = entry.status();
        if s.is_wt_new() || s.is_index_new() {
            untracked += 1;
        } else if s.is_wt_modified()
            || s.is_wt_deleted()
            || s.is_wt_renamed()
            || s.is_wt_typechange()
            || s.is_index_modified()
            || s.is_index_deleted()
            || s.is_index_renamed()
            || s.is_index_typechange()
        {
            modified += 1;
        }
    }

    (modified, untracked)
}

fn ahead_behind(repo: &Repository) -> (usize, usize) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return (0, 0),
    };

    let local_oid = match head.target() {
        Some(o) => o,
        None => return (0, 0),
    };

    let branch_name = match head.shorthand() {
        Some(n) => n.to_string(),
        None => return (0, 0),
    };

    let upstream_name = format!("refs/remotes/origin/{}", branch_name);
    let upstream_ref = match repo.find_reference(&upstream_name) {
        Ok(r) => r,
        Err(_) => return (0, 0),
    };

    let upstream_oid = match upstream_ref.target() {
        Some(o) => o,
        None => return (0, 0),
    };

    repo.graph_ahead_behind(local_oid, upstream_oid)
        .unwrap_or((0, 0))
}

fn last_commit_timestamp(repo: &Repository) -> Option<i64> {
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    Some(commit.time().seconds())
}

fn list_branches(repo: &Repository) -> Vec<BranchInfo> {
    let mut result = Vec::new();
    let branches = match repo.branches(Some(BranchType::Local)) {
        Ok(b) => b,
        Err(_) => return result,
    };

    for branch in branches.flatten() {
        let (b, _) = branch;
        let name = b.name().ok().flatten().unwrap_or("???").to_string();
        let is_head = b.is_head();
        let (last_commit_ts, last_commit_msg) = b
            .get()
            .peel_to_commit()
            .ok()
            .map(|c| {
                (
                    Some(c.time().seconds()),
                    c.message().map(|m| m.trim().to_string()),
                )
            })
            .unwrap_or((None, None));

        result.push(BranchInfo {
            name,
            is_head,
            last_commit_ts,
            last_commit_msg,
        });
    }

    result.sort_by(|a, b| b.last_commit_ts.cmp(&a.last_commit_ts));
    result
}

fn count_stale(branches: &[BranchInfo], stale_days: u64) -> usize {
    let cutoff = Utc::now().timestamp() - (stale_days as i64 * 86400);
    branches
        .iter()
        .filter(|b| {
            b.last_commit_ts
                .map(|ts| ts < cutoff)
                .unwrap_or(true)
        })
        .count()
}

fn recent_commits(repo: &Repository, count: usize) -> Vec<CommitInfo> {
    let mut result = Vec::new();
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return result,
    };

    if revwalk.push_head().is_err() {
        return result;
    }

    for oid in revwalk.flatten().take(count) {
        if let Ok(commit) = repo.find_commit(oid) {
            result.push(CommitInfo {
                hash: oid.to_string()[..7].to_string(),
                message: commit
                    .message()
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                author: commit.author().name().unwrap_or("unknown").to_string(),
                timestamp: commit.time().seconds(),
            });
        }
    }

    result
}

fn list_remotes(repo: &Repository) -> Vec<String> {
    repo.remotes()
        .ok()
        .map(|names| {
            names
                .iter()
                .flatten()
                .filter_map(|name| {
                    repo.find_remote(name)
                        .ok()
                        .and_then(|r| r.url().map(String::from))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn get_diff_stat(repo: &Repository) -> DiffStat {
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

    let diff = match repo.diff_index_to_workdir(None, None) {
        Ok(d) => d,
        Err(_) => return DiffStat::default(),
    };

    let stats = match diff.stats() {
        Ok(s) => s,
        Err(_) => return DiffStat::default(),
    };

    let mut ds = DiffStat {
        files_changed: stats.files_changed(),
        insertions: stats.insertions(),
        deletions: stats.deletions(),
    };

    // Also count staged changes
    if let Some(tree) = head_tree {
        if let Ok(staged_diff) = repo.diff_tree_to_index(Some(&tree), None, None) {
            if let Ok(staged_stats) = staged_diff.stats() {
                ds.files_changed += staged_stats.files_changed();
                ds.insertions += staged_stats.insertions();
                ds.deletions += staged_stats.deletions();
            }
        }
    }

    ds
}
