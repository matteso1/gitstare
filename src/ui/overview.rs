use super::{App, SortColumn, SortDirection};
use gitstare::theme;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Cell, Padding, Paragraph, Row, Table},
};

const LOGO: &str = r#"          _ __       __
   ____ _(_) /______/ /_____ _________
  / __ `/ / __/ ___/ __/ __ `/ ___/ _ \
 / /_/ / / /_(__  ) /_/ /_/ / /  /  __/
 \__, /_/\__/____/\__/\__,_/_/   \___/
/____/"#;

pub fn draw(f: &mut Frame, app: &App) {
    // Outer margin for breathing room
    let outer = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(f.area());

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // top margin
            Constraint::Length(8),  // logo + stats banner
            Constraint::Length(1),  // spacer
            Constraint::Min(0),    // table
            Constraint::Length(1),  // footer
        ])
        .split(outer[1]);

    // Fill background
    f.render_widget(Block::default().style(Style::default().bg(theme::BASE)), f.area());

    draw_banner(f, inner[1], app);
    draw_table(f, inner[3], app);
    draw_footer(f, inner[4], app);
}

fn draw_banner(f: &mut Frame, area: Rect, app: &App) {
    let total = app.repos.len();
    let clean = app.repos.iter().filter(|r| r.is_clean()).count();
    let dirty = total - clean;

    // Split banner horizontally: logo on left, stats on right
    let banner_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(44), // logo width
            Constraint::Min(0),    // stats
        ])
        .split(area);

    // Logo
    let logo_lines: Vec<Line> = LOGO
        .lines()
        .enumerate()
        .map(|(i, line)| {
            // Gradient from lavender to blue across lines
            let color = match i {
                0 => theme::MAUVE,
                1 => theme::LAVENDER,
                2 => theme::LAVENDER,
                3 => theme::BLUE,
                4 => theme::BLUE,
                _ => theme::BLUE,
            };
            Line::from(Span::styled(line, Style::default().fg(color).bold()))
        })
        .collect();

    let logo_block = Block::default()
        .padding(Padding::new(1, 0, 1, 0))
        .style(Style::default().bg(theme::BASE));

    let logo = Paragraph::new(logo_lines).block(logo_block);
    f.render_widget(logo, banner_chunks[0]);

    // Stats panel on the right
    let synced = app.repos.iter().filter(|r| r.ahead == 0 && r.behind == 0).count();
    let behind_count = app.repos.iter().filter(|r| r.behind > 0).count();
    let total_stale: usize = app.repos.iter().map(|r| r.stale_branches).sum();

    let stats_lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled("  Repos     ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{}", total), Style::default().fg(theme::TEXT).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Clean     ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{}", clean), Style::default().fg(theme::GREEN).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Dirty     ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                format!("{}", dirty),
                Style::default().fg(if dirty > 0 { theme::YELLOW } else { theme::GREEN }).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Behind    ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                format!("{}", behind_count),
                Style::default().fg(if behind_count > 0 { theme::RED } else { theme::GREEN }).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Synced    ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(format!("{}", synced), Style::default().fg(theme::GREEN).bold()),
        ]),
        Line::from(vec![
            Span::styled("  Stale     ", Style::default().fg(theme::SUBTEXT)),
            Span::styled(
                format!("{}", total_stale),
                Style::default().fg(if total_stale > 0 { theme::PEACH } else { theme::GREEN }).bold(),
            ),
        ]),
    ];

    // Filter mode overlay
    if app.filter_mode {
        let mut filter_lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("  / ", Style::default().fg(theme::MAUVE).bold()),
                Span::styled(&app.filter_text, Style::default().fg(theme::TEXT)),
                Span::styled("\u{2588}", Style::default().fg(theme::LAVENDER)),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::styled(
                    format!("  {} matches", app.filtered.len()),
                    Style::default().fg(theme::SUBTEXT),
                ),
            ]),
        ];
        filter_lines.resize(7, Line::raw(""));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::MAUVE))
            .title(Span::styled(" Search ", Style::default().fg(theme::MAUVE).bold()))
            .style(Style::default().bg(theme::BASE));

        let filter_para = Paragraph::new(filter_lines).block(block);
        f.render_widget(filter_para, banner_chunks[1]);
    } else if !app.filter_text.is_empty() {
        let mut filter_lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::styled("  filter: ", Style::default().fg(theme::SUBTEXT)),
                Span::styled(
                    format!("\"{}\"", app.filter_text),
                    Style::default().fg(theme::YELLOW),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    format!("  {} matches", app.filtered.len()),
                    Style::default().fg(theme::SUBTEXT),
                ),
            ]),
        ];
        filter_lines.resize(7, Line::raw(""));

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::SURFACE))
            .title(Span::styled(" Filter ", Style::default().fg(theme::YELLOW).bold()))
            .style(Style::default().bg(theme::BASE));

        let filter_para = Paragraph::new(filter_lines).block(block);
        f.render_widget(filter_para, banner_chunks[1]);
    } else {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::SURFACE))
            .title(Span::styled(" Status ", Style::default().fg(theme::LAVENDER).bold()))
            .style(Style::default().bg(theme::BASE));

        let stats_para = Paragraph::new(stats_lines).block(block);
        f.render_widget(stats_para, banner_chunks[1]);
    }
}

fn sort_indicator(app: &App, col: SortColumn) -> Span<'static> {
    if app.sort_col == col {
        match app.sort_dir {
            SortDirection::Asc => Span::styled(" \u{25B2}", Style::default().fg(theme::MAUVE)),
            SortDirection::Desc => Span::styled(" \u{25BC}", Style::default().fg(theme::MAUVE)),
        }
    } else {
        Span::raw("")
    }
}

fn draw_table(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = vec![
        Cell::from(Line::from(vec![
            Span::raw("  "),
            Span::styled("Repo", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::Name),
        ])),
        Cell::from(Line::from(vec![
            Span::styled("Branch", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::Branch),
        ])),
        Cell::from(Line::from(vec![
            Span::styled("Status", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::Status),
        ])),
        Cell::from(Line::from(vec![
            Span::styled("+/-", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::AheadBehind),
        ])),
        Cell::from(Line::from(vec![
            Span::styled("Last Commit", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::LastCommit),
        ])),
        Cell::from(Line::from(vec![
            Span::styled("Stale", Style::default().fg(theme::SUBTEXT).bold()),
            sort_indicator(app, SortColumn::StaleBranches),
        ])),
    ];

    let header = Row::new(header_cells)
        .height(1)
        .style(Style::default().bg(theme::BASE))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let repo = &app.repos[idx];
            let selected = i == app.selected;
            let status_color = status_color(repo);

            let accent = if selected {
                Span::styled("\u{258C} ", Style::default().fg(theme::BLUE))
            } else {
                Span::styled("  ", Style::default())
            };

            let name_style = if selected {
                Style::default().fg(theme::TEXT).bold()
            } else {
                Style::default().fg(theme::TEXT)
            };

            let cells = vec![
                Cell::from(Line::from(vec![
                    accent,
                    Span::styled(repo.name.as_str(), name_style),
                ])),
                Cell::from(Span::styled(
                    repo.branch.as_str(),
                    Style::default().fg(theme::BLUE),
                )),
                Cell::from(Span::styled(
                    repo.status_string(),
                    Style::default().fg(status_color),
                )),
                Cell::from(Span::styled(
                    repo.ahead_behind_string(),
                    Style::default().fg(ahead_behind_color(repo)),
                )),
                Cell::from(Span::styled(
                    repo.last_commit_relative(),
                    Style::default().fg(theme::SUBTEXT),
                )),
                Cell::from(Span::styled(
                    format!("{}", repo.stale_branches),
                    Style::default().fg(if repo.stale_branches > 0 {
                        theme::PEACH
                    } else {
                        theme::SUBTEXT
                    }),
                )),
            ];

            let bg = if selected { theme::SURFACE } else { theme::BASE };
            Row::new(cells).style(Style::default().bg(bg))
        })
        .collect();

    let widths = [
        Constraint::Percentage(22),
        Constraint::Percentage(20),
        Constraint::Percentage(14),
        Constraint::Percentage(12),
        Constraint::Percentage(16),
        Constraint::Percentage(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::SURFACE))
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(theme::BASE)),
        );

    f.render_widget(table, area);
}

fn status_color(repo: &gitstare::git::RepoInfo) -> Color {
    if repo.behind > 0 {
        theme::RED
    } else if !repo.is_clean() {
        theme::YELLOW
    } else if repo.ahead > 0 {
        theme::YELLOW
    } else {
        theme::GREEN
    }
}

fn ahead_behind_color(repo: &gitstare::git::RepoInfo) -> Color {
    if repo.behind > 0 {
        theme::RED
    } else if repo.ahead > 0 {
        theme::YELLOW
    } else {
        theme::GREEN
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let keys: Vec<(&str, &str)> = if app.filter_mode {
        vec![
            ("Esc", "cancel"),
            ("Enter", "confirm"),
        ]
    } else {
        vec![
            ("j/k", "navigate"),
            ("Enter", "detail"),
            ("/", "filter"),
            ("1-6", "sort"),
            ("g/G", "top/bottom"),
            ("q", "quit"),
        ]
    };

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
