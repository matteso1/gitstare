use super::{App, SortColumn, SortDirection};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),   // table
            Constraint::Length(1), // footer
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_table(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let total = app.repos.len();
    let clean = app.repos.iter().filter(|r| r.is_clean()).count();
    let dirty = total - clean;

    let title = format!(
        " gitstare -- {} repos ({} clean, {} dirty)",
        total, clean, dirty
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(title, Style::default().fg(Color::Cyan).bold()));

    if app.filter_mode {
        let input = Paragraph::new(format!(" / {}", app.filter_text))
            .style(Style::default().fg(Color::Yellow))
            .block(block);
        f.render_widget(input, area);
    } else if !app.filter_text.is_empty() {
        let filtered = Paragraph::new(format!(
            " filter: \"{}\" ({} matches)",
            app.filter_text,
            app.filtered.len()
        ))
        .style(Style::default().fg(Color::Yellow))
        .block(block);
        f.render_widget(filtered, area);
    } else {
        let empty = Paragraph::new("").block(block);
        f.render_widget(empty, area);
    }
}

fn sort_indicator(app: &App, col: SortColumn) -> &'static str {
    if app.sort_col == col {
        match app.sort_dir {
            SortDirection::Asc => " ^",
            SortDirection::Desc => " v",
        }
    } else {
        ""
    }
}

fn draw_table(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = [
        format!("Repo{}", sort_indicator(app, SortColumn::Name)),
        format!("Branch{}", sort_indicator(app, SortColumn::Branch)),
        format!("Status{}", sort_indicator(app, SortColumn::Status)),
        format!(
            "Ahead/Behind{}",
            sort_indicator(app, SortColumn::AheadBehind)
        ),
        format!("Last Commit{}", sort_indicator(app, SortColumn::LastCommit)),
        format!(
            "Stale{}",
            sort_indicator(app, SortColumn::StaleBranches)
        ),
    ];

    let header = Row::new(
        header_cells
            .iter()
            .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::Cyan).bold())),
    )
    .height(1);

    let rows: Vec<Row> = app
        .filtered
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let repo = &app.repos[idx];
            let row_style = row_color(repo);
            let selected = i == app.selected;

            let cells = vec![
                Cell::from(repo.name.as_str()),
                Cell::from(repo.branch.as_str()),
                Cell::from(repo.status_string()),
                Cell::from(repo.ahead_behind_string()),
                Cell::from(repo.last_commit_relative()),
                Cell::from(format!("{}", repo.stale_branches)),
            ];

            let row = Row::new(cells).style(row_style);
            if selected {
                row.style(
                    row_style
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::DarkGray),
                )
            } else {
                row
            }
        })
        .collect();

    let widths = [
        Constraint::Percentage(22),
        Constraint::Percentage(20),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(16),
        Constraint::Percentage(14),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .row_highlight_style(Style::default());

    f.render_widget(table, area);
}

fn row_color(repo: &crate::git::RepoInfo) -> Style {
    if repo.behind > 0 && !repo.is_clean() {
        // Behind and dirty = red
        Style::default().fg(Color::Red)
    } else if repo.behind > 0 {
        // Behind but clean
        Style::default().fg(Color::Red)
    } else if !repo.is_clean() {
        // Dirty but not behind
        Style::default().fg(Color::Yellow)
    } else if repo.ahead > 0 {
        // Ahead but clean
        Style::default().fg(Color::Yellow)
    } else {
        // Clean and synced
        Style::default().fg(Color::Green)
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help = if app.filter_mode {
        " Type to filter | Enter to confirm | Esc to cancel"
    } else {
        " j/k: navigate | Enter: detail | /: filter | 1-6: sort | q: quit"
    };
    let footer = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, area);
}
