use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan the given directories for git repositories, returning paths to repo roots.
pub fn scan(paths: &[PathBuf], max_depth: usize, ignore: &[String]) -> Vec<PathBuf> {
    paths
        .par_iter()
        .flat_map(|root| scan_dir(root, max_depth, ignore))
        .collect()
}

fn scan_dir(root: &Path, max_depth: usize, ignore: &[String]) -> Vec<PathBuf> {
    let mut repos = Vec::new();

    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            // Don't descend into ignored directories
            if ignore.iter().any(|ig| name == *ig) {
                return false;
            }
            // Don't descend into hidden dirs (except .git which we're looking for)
            if name.starts_with('.') && name != ".git" {
                return false;
            }
            true
        });

    for entry in walker.flatten() {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_path_buf());
            }
        }
    }

    repos
}
