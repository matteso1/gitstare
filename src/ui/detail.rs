use super::App;
use gitstare::git::RepoInfo;
use gitstare::theme;
use chrono::{DateTime, Utc};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, List, ListItem, Padding, Paragraph, Wrap},
};

pub fn draw(f: &mut Frame, app: &App) {
    let repo = match app.selected_repo() {
        Some(r) => r,
        None => return,
    };

    // Fill background
    f.render_widget(Block::default().style(Style::default().bg(theme::BASE)), f.area());

    // Outer margin
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // top margin
            Constraint::Length(6),  // repo header
            Constraint::Length(1),  // spacer
            Constraint::Percentage(35), // branches
            Constraint::Length(1),  // spacer
            Constraint::Min(0),    // commits
            Constraint::Length(1),  // footer
        ])
        .split(outer[1]);

    draw_repo_header(f, chunks[1], repo);
    draw_branches(f, chunks[3], repo);
    draw_commits(f, chunks[5], repo, app);
    draw_footer(f, chunks[6]);
}

fn draw_repo_header(f: &mut Frame, area: Rect, repo: &RepoInfo) {
    let remote_str = if repo.remotes.is_empty() {
        "none".to_string()
    } else {
        repo.remotes.join(", ")
    };

    let status_color = if repo.is_clean() { theme::GREEN } else { theme::YELLOW };
    let ab_color = if repo.behind > 0 {
        theme::RED
    } else if repo.ahead > 0 {
        theme::YELLOW
    } else {
        theme::GREEN
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Path     ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                repo.path.to_string_lossy().to_string(),
                Style::default().fg(theme::TEXT),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Branch   ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(&repo.branch, Style::default().fg(theme::BLUE).bold()),
            Span::styled("  \u{2022}  ", Style::default().fg(theme::OVERLAY)),
            Span::styled(repo.status_string(), Style::default().fg(status_color)),
            Span::styled("  \u{2022}  ", Style::default().fg(theme::OVERLAY)),
            Span::styled(repo.ahead_behind_string(), Style::default().fg(ab_color)),
        ]),
        Line::from(vec![
            Span::styled("  Remote   ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(remote_str, Style::default().fg(theme::SUBTEXT)),
        ]),
        Line::from(vec![
            Span::styled("  Changes  ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                format!("{} files", repo.diff_stat.files_changed),
                Style::default().fg(theme::TEXT),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("+{}", repo.diff_stat.insertions),
                Style::default().fg(theme::GREEN),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("-{}", repo.diff_stat.deletions),
                Style::default().fg(theme::RED),
            ),
        ]),
    ];

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::SURFACE))
        .title(Span::styled(
            format!(" {} ", repo.name),
            Style::default().fg(theme::LAVENDER).bold(),
        ))
        .padding(Padding::vertical(0))
        .style(Style::default().bg(theme::BASE));

    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn draw_branches(f: &mut Frame, area: Rect, repo: &RepoInfo) {
    let items: Vec<ListItem> = repo
        .branches
        .iter()
        .map(|b| {
            let age = match b.last_commit_ts {
                Some(ts) => relative_time(ts),
                None => "never".into(),
            };

            let line = if b.is_head {
                Line::from(vec![
                    Span::styled("  \u{25CF} ", Style::default().fg(theme::BLUE)),
                    Span::styled(
                        format!("{:<30}", b.name),
                        Style::default().fg(theme::BLUE).bold(),
                    ),
                    Span::styled(age, Style::default().fg(theme::SUBTEXT)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(
                        format!("{:<30}", b.name),
                        Style::default().fg(theme::TEXT),
                    ),
                    Span::styled(age, Style::default().fg(theme::SUBTEXT)),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::SURFACE))
        .title(Span::styled(
            format!(" Branches ({}) ", repo.branches.len()),
            Style::default().fg(theme::LAVENDER).bold(),
        ))
        .style(Style::default().bg(theme::BASE));

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
            let line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&c.hash, Style::default().fg(theme::MAUVE)),
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{:<6}", age),
                    Style::default().fg(theme::SUBTEXT),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    truncate(&c.message, 50),
                    Style::default().fg(theme::TEXT),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(&c.author, Style::default().fg(theme::SUBTEXT)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::SURFACE))
        .title(Span::styled(
            format!(" Commits ({}) ", repo.recent_commits.len()),
            Style::default().fg(theme::LAVENDER).bold(),
        ))
        .style(Style::default().bg(theme::BASE));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let keys: Vec<(&str, &str)> = vec![
        ("Esc", "back"),
        ("j/k", "scroll"),
        ("q", "quit"),
    ];

    let mut spans = vec![Span::raw(" ")];
    for (i, (key, desc)) in keys.iter().enumerate() {
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default().fg(theme::BASE).bg(theme::SUBTEXT).bold(),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(theme::SUBTEXT),
        ));
        if i < keys.len() - 1 {
            spans.push(Span::styled("  ", Style::default()));
        }
    }

    let footer = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(theme::BASE));
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
