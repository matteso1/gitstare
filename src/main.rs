mod cache;
mod config;
mod git;
mod scanner;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gitstare", version, about = "A git-aware multi-repo dashboard for your terminal")]
struct Cli {
    /// Directories to scan for git repos (overrides config)
    #[arg(short, long)]
    paths: Vec<PathBuf>,

    /// Maximum directory depth to scan
    #[arg(short = 'd', long)]
    max_depth: Option<usize>,

    /// Skip the cache and do a fresh scan
    #[arg(long)]
    fresh: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = config::Config::load()?;

    let scan_paths = if cli.paths.is_empty() {
        cfg.scan_paths.clone()
    } else {
        cli.paths.clone()
    };
    let max_depth = cli.max_depth.unwrap_or(cfg.max_depth);

    let db = cache::Cache::open()?;

    let repos = if !cli.fresh {
        if let Some(cached) = db.load()? {
            cached
        } else {
            let discovered = scanner::scan(&scan_paths, max_depth, &cfg.ignore);
            let repos = git::read_all(&discovered, cfg.stale_threshold);
            db.save(&repos)?;
            repos
        }
    } else {
        let discovered = scanner::scan(&scan_paths, max_depth, &cfg.ignore);
        let repos = git::read_all(&discovered, cfg.stale_threshold);
        db.save(&repos)?;
        repos
    };

    ui::run(repos, cfg)?;

    Ok(())
}
