use super::App;
use crate::git::RepoInfo;
use chrono::{DateTime, Utc};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let repo = match app.selected_repo() {
        Some(r) => r,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // repo header
            Constraint::Length(10), // branches
            Constraint::Min(0),    // commits
            Constraint::Length(1),  // footer
        ])
        .split(f.area());

    draw_repo_header(f, chunks[0], repo);
    draw_branches(f, chunks[1], repo, app);
    draw_commits(f, chunks[2], repo, app);
    draw_footer(f, chunks[3]);
}

fn draw_repo_header(f: &mut Frame, area: Rect, repo: &RepoInfo) {
    let remote_str = if repo.remotes.is_empty() {
        "none".to_string()
    } else {
        repo.remotes.join(", ")
    };

    let info = format!(
        "Path:    {}\n\
         Branch:  {} | Status: {} | {}\n\
         Remote:  {}\n\
         Diff:    {} files changed, +{} -{} ",
        repo.path.display(),
        repo.branch,
        repo.status_string(),
        repo.ahead_behind_string(),
        remote_str,
        repo.diff_stat.files_changed,
        repo.diff_stat.insertions,
        repo.diff_stat.deletions,
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            format!(" {} ", repo.name),
            Style::default().fg(Color::Cyan).bold(),
        ));

    let para = Paragraph::new(info)
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_branches(f: &mut Frame, area: Rect, repo: &RepoInfo, _app: &App) {
    let items: Vec<ListItem> = repo
        .branches
        .iter()
        .map(|b| {
            let age = match b.last_commit_ts {
                Some(ts) => relative_time(ts),
                None => "never".into(),
            };
            let head_marker = if b.is_head { "* " } else { "  " };
            let style = if b.is_head {
                Style::default().fg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(
                "{}{:<30} {}",
                head_marker, b.name, age
            ))
            .style(style)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            format!(" Branches ({}) ", repo.branches.len()),
            Style::default().fg(Color::Cyan).bold(),
        ));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_commits(f: &mut Frame, area: Rect, repo: &RepoInfo, app: &App) {
    let items: Vec<ListItem> = repo
        .recent_commits
        .iter()
        .skip(app.detail_scroll)
        .map(|c| {
            let age = relative_time(c.timestamp);
            ListItem::new(format!(
                " {} {:<8} {:<50} {}",
                c.hash,
                age,
                truncate(&c.message, 50),
                c.author
            ))
            .style(Style::default().fg(Color::White))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            format!(" Commits ({}) ", repo.recent_commits.len()),
            Style::default().fg(Color::Cyan).bold(),
        ));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let help = " Esc/Backspace: back | j/k: scroll commits | q: quit";
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}

fn relative_time(ts: i64) -> String {
    let time = DateTime::from_timestamp(ts, 0).unwrap_or_default();
    let now = Utc::now();
    let dur = now.signed_duration_since(time);

    if dur.num_minutes() < 1 {
        "now".into()
    } else if dur.num_minutes() < 60 {
        format!("{}m", dur.num_minutes())
    } else if dur.num_hours() < 24 {
        format!("{}h", dur.num_hours())
    } else if dur.num_days() < 30 {
        format!("{}d", dur.num_days())
    } else if dur.num_days() < 365 {
        format!("{}mo", dur.num_days() / 30)
    } else {
        format!("{}y", dur.num_days() / 365)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
