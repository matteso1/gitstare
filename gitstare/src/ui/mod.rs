mod detail;
mod overview;

use crate::config::Config;
use crate::git::RepoInfo;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Branch,
    Status,
    AheadBehind,
    LastCommit,
    StaleBranches,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum View {
    Overview,
    Detail,
}

pub struct App {
    pub repos: Vec<RepoInfo>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub sort_col: SortColumn,
    pub sort_dir: SortDirection,
    pub filter_text: String,
    pub filter_mode: bool,
    pub view: View,
    pub detail_scroll: usize,
    #[allow(dead_code)]
    pub config: Config,
    pub should_quit: bool,
}

impl App {
    fn new(repos: Vec<RepoInfo>, config: Config) -> Self {
        let filtered: Vec<usize> = (0..repos.len()).collect();
        let mut app = App {
            repos,
            filtered,
            selected: 0,
            sort_col: SortColumn::Name,
            sort_dir: SortDirection::Asc,
            filter_text: String::new(),
            filter_mode: false,
            view: View::Overview,
            detail_scroll: 0,
            config,
            should_quit: false,
        };
        app.apply_sort();
        app
    }

    fn apply_filter(&mut self) {
        let query = self.filter_text.to_lowercase();
        self.filtered = (0..self.repos.len())
            .filter(|&i| {
                if query.is_empty() {
                    return true;
                }
                let r = &self.repos[i];
                r.name.to_lowercase().contains(&query)
                    || r.branch.to_lowercase().contains(&query)
                    || r.path.to_string_lossy().to_lowercase().contains(&query)
            })
            .collect();
        self.apply_sort();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    fn apply_sort(&mut self) {
        let repos = &self.repos;
        let col = self.sort_col;
        let dir = self.sort_dir;

        self.filtered.sort_by(|&a, &b| {
            let ra = &repos[a];
            let rb = &repos[b];
            let ord = match col {
                SortColumn::Name => ra.name.to_lowercase().cmp(&rb.name.to_lowercase()),
                SortColumn::Branch => ra.branch.to_lowercase().cmp(&rb.branch.to_lowercase()),
                SortColumn::Status => {
                    let sa = ra.modified + ra.untracked;
                    let sb = rb.modified + rb.untracked;
                    sa.cmp(&sb)
                }
                SortColumn::AheadBehind => {
                    let sa = ra.ahead + ra.behind;
                    let sb = rb.ahead + rb.behind;
                    sa.cmp(&sb)
                }
                SortColumn::LastCommit => ra.last_commit_ts.cmp(&rb.last_commit_ts),
                SortColumn::StaleBranches => ra.stale_branches.cmp(&rb.stale_branches),
            };
            match dir {
                SortDirection::Asc => ord,
                SortDirection::Desc => ord.reverse(),
            }
        });
    }

    fn set_sort(&mut self, col: SortColumn) {
        if self.sort_col == col {
            self.sort_dir = self.sort_dir.toggle();
        } else {
            self.sort_col = col;
            self.sort_dir = SortDirection::Asc;
        }
        self.apply_sort();
    }

    fn selected_repo(&self) -> Option<&RepoInfo> {
        self.filtered
            .get(self.selected)
            .map(|&idx| &self.repos[idx])
    }
}

pub fn run(repos: Vec<RepoInfo>, config: Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(repos, config);

    loop {
        terminal.draw(|f| {
            match app.view {
                View::Overview => overview::draw(f, &app),
                View::Detail => detail::draw(f, &app),
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Ctrl+C always quits
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.should_quit = true;
            }

            if app.filter_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.filter_mode = false;
                        app.filter_text.clear();
                        app.apply_filter();
                    }
                    KeyCode::Enter => {
                        app.filter_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.filter_text.pop();
                        app.apply_filter();
                    }
                    KeyCode::Char(c) => {
                        app.filter_text.push(c);
                        app.apply_filter();
                    }
                    _ => {}
                }
            } else {
                match app.view {
                    View::Overview => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('j') | KeyCode::Down => {
                            if app.selected < app.filtered.len().saturating_sub(1) {
                                app.selected += 1;
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.selected = app.selected.saturating_sub(1);
                        }
                        KeyCode::Char('g') => app.selected = 0,
                        KeyCode::Char('G') => {
                            app.selected = app.filtered.len().saturating_sub(1);
                        }
                        KeyCode::Enter => {
                            if app.selected_repo().is_some() {
                                app.detail_scroll = 0;
                                app.view = View::Detail;
                            }
                        }
                        KeyCode::Char('/') => {
                            app.filter_mode = true;
                            app.filter_text.clear();
                        }
                        // Sort keys: 1-6 for each column
                        KeyCode::Char('1') => app.set_sort(SortColumn::Name),
                        KeyCode::Char('2') => app.set_sort(SortColumn::Branch),
                        KeyCode::Char('3') => app.set_sort(SortColumn::Status),
                        KeyCode::Char('4') => app.set_sort(SortColumn::AheadBehind),
                        KeyCode::Char('5') => app.set_sort(SortColumn::LastCommit),
                        KeyCode::Char('6') => app.set_sort(SortColumn::StaleBranches),
                        _ => {}
                    },
                    View::Detail => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Esc | KeyCode::Backspace => {
                            app.view = View::Overview;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.detail_scroll += 1;
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.detail_scroll = app.detail_scroll.saturating_sub(1);
                        }
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
