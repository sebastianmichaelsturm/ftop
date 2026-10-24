use crate::app::App;
use crate::model::{format_size, ChangeType, SortMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header / Metrics
            Constraint::Min(5),    // Table of changing files
            Constraint::Length(2), // Help / Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_table(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let elapsed = app.start_time.elapsed().as_secs();
    let uptime_str = format!("{:02}:{:02}", elapsed / 60, elapsed % 60);

    let mut title_spans = vec![
        Span::styled("ftop", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ];

    if app.is_paused {
        title_spans.push(Span::raw(" "));
        title_spans.push(Span::styled(
            "[PAUSED]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    title_spans.push(Span::raw(" — Watching: "));
    title_spans.push(Span::styled(
        app.root_path.to_string_lossy(),
        Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED),
    ));

    let mut metrics_spans = vec![
        Span::raw("Files: "),
        Span::styled(format!("{}", app.records.len()), Style::default().fg(Color::Green)),
        Span::raw("  |  Events: "),
        Span::styled(format!("{}", app.total_events), Style::default().fg(Color::Yellow)),
        Span::raw("  |  Sort: "),
        Span::styled(app.sort_mode.as_str(), Style::default().fg(Color::Magenta)),
    ];

    if !app.filter_query.is_empty() {
        metrics_spans.push(Span::raw("  |  Filter: "));
        metrics_spans.push(Span::styled(
            format!("\"{}\"", app.filter_query),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }

    metrics_spans.push(Span::raw("  |  Uptime: "));
    metrics_spans.push(Span::styled(uptime_str, Style::default().fg(Color::DarkGray)));

    let header_text = vec![Line::from(title_spans), Line::from(metrics_spans)];

    let header_widget = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(header_widget, area);
}

fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let default_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

    let path_label = if app.sort_mode == SortMode::Path { "Path ▲" } else { "Path" };
    let path_style = if app.sort_mode == SortMode::Path { active_style } else { default_style };

    let changes_label = if app.sort_mode == SortMode::ChangeCount { "Changes ▼" } else { "Changes" };
    let changes_style = if app.sort_mode == SortMode::ChangeCount { active_style } else { default_style };

    let size_label = if app.sort_mode == SortMode::Size { "Size ▼" } else { "Size" };
    let size_style = if app.sort_mode == SortMode::Size { active_style } else { default_style };

    let time_label = if app.sort_mode == SortMode::Recent { "Time ▼" } else { "Time" };
    let time_style = if app.sort_mode == SortMode::Recent { active_style } else { default_style };

    let header_cells = vec![
        Cell::from(path_label).style(path_style),
        Cell::from("Last Event").style(default_style),
        Cell::from(changes_label).style(changes_style),
        Cell::from(size_label).style(size_style),
        Cell::from(time_label).style(time_style),
    ];
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let sorted = app.sorted_records();

    let rows: Vec<Row> = sorted
        .iter()
        .map(|item| {
            let (event_color, event_text) = match item.event_type {
                ChangeType::Created => (Color::Green, "CREATED"),
                ChangeType::Modified => (Color::Yellow, "MODIFIED"),
                ChangeType::Deleted => (Color::Red, "DELETED"),
                ChangeType::Renamed => (Color::Cyan, "RENAMED"),
                ChangeType::Other => (Color::Gray, "OTHER"),
            };

            let cells = vec![
                Cell::from(item.display_path.clone()),
                Cell::from(event_text).style(Style::default().fg(event_color)),
                Cell::from(item.change_count.to_string()).style(Style::default().fg(Color::White)),
                Cell::from(format_size(item.size_bytes)),
                Cell::from(item.last_updated.format("%H:%M:%S%.3f").to_string())
                    .style(Style::default().fg(Color::DarkGray)),
            ];
            Row::new(cells).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::NONE))
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let line = if app.is_filtering {
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&app.filter_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
            Span::styled("  (Enter: apply, Esc: clear/exit)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit  "),
            Span::styled("[Space]", Style::default().fg(Color::Yellow)),
            Span::raw(if app.is_paused { " Resume  " } else { " Pause  " }),
            Span::styled("[/]", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter  "),
            Span::styled("[s]", Style::default().fg(Color::Yellow)),
            Span::raw(" Sort  "),
            Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" Open  "),
            Span::styled("[c]", Style::default().fg(Color::Yellow)),
            Span::raw(" Clear  "),
            Span::styled("[↑/↓, j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate"),
        ])
    };

    let footer = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}
