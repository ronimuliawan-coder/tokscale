use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use super::widgets::{
    format_cost, format_cost_per_million, format_tokens, get_client_display_name,
    total_tokens_cell, truncate_text, viewport_scrollbar_state,
};
use crate::tui::app::{App, SortDirection, SortField};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.is_client_detail_active() {
        render_client_detail(frame, app, area);
    } else {
        render_client_list(frame, app, area);
    }
}

fn render_client_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(app.glyphs.border)
        .border_style(Style::default().fg(app.theme.border))
        .title(Span::styled(
            " Clients ",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(app.theme.background));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height.saturating_sub(1) as usize;
    app.set_max_visible_items(visible_height);

    let is_narrow = app.is_narrow();
    let is_very_narrow = app.is_very_narrow();
    let sort_field = app.sort_field;
    let sort_direction = app.sort_direction;
    let scroll_offset = app.scroll_offset;
    let selected_index = app.selected_index;
    let theme_accent = app.theme.accent;
    let theme_muted = app.theme.muted;
    let theme_selection = app.theme.selection;
    let striped_row_style = app.theme.striped_row_style();

    let clients = app.get_sorted_clients();
    if clients.is_empty() {
        let empty_msg =
            Paragraph::new("No client usage data found. Press 'r' to refresh, 's' for sources.")
                .style(Style::default().fg(theme_muted))
                .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let cache_hdr = app.cache_hit_header();

    let header_cells = if is_very_narrow {
        vec!["Client", "Cost"]
    } else if is_narrow {
        vec!["Client", "Cost", "Tokens", cache_hdr, "Msgs"]
    } else {
        vec![
            "#", "Client", "Share", "Cost", "Tokens", "Input", "Output", "Cache R", "Cache W",
            cache_hdr, "Cost/1M", "Msgs", "Sessions",
        ]
    };

    let sort_indicator = |field: SortField| -> &'static str {
        if sort_field == field {
            match sort_direction {
                SortDirection::Ascending => " ▴",
                SortDirection::Descending => " ▾",
            }
        } else {
            ""
        }
    };

    let header = Row::new(
        header_cells
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let indicator = if is_very_narrow {
                    match i {
                        1 => sort_indicator(SortField::Cost),
                        _ => "",
                    }
                } else if is_narrow {
                    match i {
                        1 => sort_indicator(SortField::Cost),
                        2 => sort_indicator(SortField::Tokens),
                        _ => "",
                    }
                } else {
                    match i {
                        3 => sort_indicator(SortField::Cost),
                        4 => sort_indicator(SortField::Tokens),
                        _ => "",
                    }
                };
                Cell::from(format!("{}{}", h, indicator))
            })
            .collect::<Vec<_>>(),
    )
    .style(
        Style::default()
            .fg(theme_accent)
            .add_modifier(Modifier::BOLD),
    )
    .height(1);

    let clients_len = clients.len();
    let start = scroll_offset.min(clients_len.saturating_sub(1));
    let end = (start + visible_height).min(clients_len);

    if start >= clients_len {
        return;
    }

    let total_cost = app.data.total_cost;

    let rows: Vec<Row> = clients[start..end]
        .iter()
        .enumerate()
        .map(|(i, client)| {
            let idx = i + start;
            let is_selected = idx == selected_index;
            let is_striped = idx % 2 == 1;

            let display_name = get_client_display_name(&client.client);
            let total_tokens = client.tokens.total();
            let share_pct = if total_cost > 0.0 {
                format!("{:.1}%", (client.cost / total_cost) * 100.0)
            } else {
                app.glyphs.missing.to_string()
            };
            let cache_str = app.format_cache(
                client.tokens.cache_read,
                client.tokens.input,
                client.tokens.cache_write,
            );

            let cells: Vec<Cell> = if is_very_narrow {
                vec![
                    Cell::from(truncate_text(&display_name, 18))
                        .style(Style::default().fg(app.theme.foreground)),
                    Cell::from(format_cost(client.cost)).style(Style::default().fg(Color::Green)),
                ]
            } else if is_narrow {
                vec![
                    Cell::from(truncate_text(&display_name, 18)).style(
                        Style::default()
                            .fg(app.theme.foreground)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(format_cost(client.cost)).style(Style::default().fg(Color::Green)),
                    total_tokens_cell(total_tokens, &app.theme),
                    Cell::from(cache_str).style(Style::default().fg(theme_muted)),
                    Cell::from(client.message_count.to_string())
                        .style(Style::default().fg(theme_muted)),
                ]
            } else {
                vec![
                    Cell::from(format!("{}", idx + 1)).style(Style::default().fg(theme_muted)),
                    Cell::from(truncate_text(&display_name, 20)).style(
                        Style::default()
                            .fg(app.theme.foreground)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(share_pct).style(Style::default().fg(app.theme.accent)),
                    Cell::from(format_cost(client.cost)).style(Style::default().fg(Color::Green)),
                    total_tokens_cell(total_tokens, &app.theme),
                    Cell::from(format_tokens(client.tokens.input))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(client.tokens.output))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(client.tokens.cache_read))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(client.tokens.cache_write))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(cache_str).style(Style::default().fg(app.theme.accent)),
                    Cell::from(format_cost_per_million(
                        client.cost,
                        total_tokens,
                        app.glyphs,
                    ))
                    .style(Style::default().fg(theme_muted)),
                    Cell::from(client.message_count.to_string())
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(client.session_count.to_string())
                        .style(Style::default().fg(theme_muted)),
                ]
            };

            let row_style = if is_selected {
                Style::default().bg(theme_selection)
            } else if is_striped {
                striped_row_style
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style).height(1)
        })
        .collect();

    let widths: Vec<Constraint> = if is_very_narrow {
        vec![Constraint::Percentage(60), Constraint::Percentage(40)]
    } else if is_narrow {
        vec![
            Constraint::Min(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(6),
        ]
    } else if inner.width < 127 {
        // Mid tier: the wide layout needs ~127 columns (99 fixed + name +
        // gaps + borders) and clips at 80-126; drop the $/M, messages and
        // sessions columns and shrink the rest so rows fit.
        vec![
            Constraint::Length(2),
            Constraint::Min(12),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(7),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(14),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(7),
            Constraint::Length(8),
        ]
    };

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);

    if clients_len > visible_height {
        let mut scrollbar_state =
            viewport_scrollbar_state(clients_len, scroll_offset, visible_height);
        let scrollbar = app.glyphs.scrollbar();
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn render_client_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let client_name = app.client_detail_name().unwrap_or("Unknown").to_string();
    let display_name = get_client_display_name(&client_name);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(app.glyphs.border)
        .border_style(Style::default().fg(app.theme.border))
        .title(Span::styled(
            format!(" Client: {} (Press Esc to return) ", display_name),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(app.theme.background));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_height = inner.height.saturating_sub(1) as usize;
    app.set_max_visible_items(visible_height);

    let rows_data = app.get_sorted_client_detail_rows();
    if rows_data.is_empty() {
        let empty_msg = Paragraph::new("No model usage found for this client.")
            .style(Style::default().fg(app.theme.muted))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let is_narrow = app.is_narrow();
    let is_very_narrow = app.is_very_narrow();
    let scroll_offset = app.scroll_offset;
    let selected_index = app.selected_index;
    let theme_accent = app.theme.accent;
    let theme_muted = app.theme.muted;
    let theme_selection = app.theme.selection;
    let striped_row_style = app.theme.striped_row_style();

    let cache_hdr = app.cache_hit_header();

    let header_cells = if is_very_narrow {
        vec!["Model", "Cost"]
    } else if is_narrow {
        vec!["Model", "Provider", "Tokens", "Cost"]
    } else {
        vec![
            "#", "Model", "Provider", "Tokens", "Input", "Output", "Cache R", "Cache W", cache_hdr,
            "Cost", "Cost/1M", "Msgs",
        ]
    };

    let header = Row::new(
        header_cells
            .iter()
            .map(|h| Cell::from(*h))
            .collect::<Vec<_>>(),
    )
    .style(
        Style::default()
            .fg(theme_accent)
            .add_modifier(Modifier::BOLD),
    )
    .height(1);

    let rows_len = rows_data.len();
    let start = scroll_offset.min(rows_len.saturating_sub(1));
    let end = (start + visible_height).min(rows_len);

    if start >= rows_len {
        return;
    }

    let rows: Vec<Row> = rows_data[start..end]
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let idx = i + start;
            let is_selected = idx == selected_index;
            let is_striped = idx % 2 == 1;

            let total_tokens = row.tokens.total();
            let model_color = app.model_color_for(row.provider, row.color_key);
            let cache_str = app.format_cache(
                row.tokens.cache_read,
                row.tokens.input,
                row.tokens.cache_write,
            );

            let cells: Vec<Cell> = if is_very_narrow {
                vec![
                    Cell::from(truncate_text(row.model, 20))
                        .style(Style::default().fg(model_color)),
                    Cell::from(format_cost(row.cost)).style(Style::default().fg(Color::Green)),
                ]
            } else if is_narrow {
                vec![
                    Cell::from(truncate_text(row.model, 22)).style(
                        Style::default()
                            .fg(model_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(row.provider).style(Style::default().fg(theme_muted)),
                    total_tokens_cell(total_tokens, &app.theme),
                    Cell::from(format_cost(row.cost)).style(Style::default().fg(Color::Green)),
                ]
            } else {
                vec![
                    Cell::from(format!("{}", idx + 1)).style(Style::default().fg(theme_muted)),
                    Cell::from(truncate_text(row.model, 28)).style(
                        Style::default()
                            .fg(model_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Cell::from(row.provider).style(Style::default().fg(theme_muted)),
                    total_tokens_cell(total_tokens, &app.theme),
                    Cell::from(format_tokens(row.tokens.input))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(row.tokens.output))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(row.tokens.cache_read))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(format_tokens(row.tokens.cache_write))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(cache_str).style(Style::default().fg(app.theme.accent)),
                    Cell::from(format_cost(row.cost)).style(Style::default().fg(Color::Green)),
                    Cell::from(format_cost_per_million(row.cost, total_tokens, app.glyphs))
                        .style(Style::default().fg(theme_muted)),
                    Cell::from(row.messages.to_string()).style(Style::default().fg(theme_muted)),
                ]
            };

            let row_style = if is_selected {
                Style::default().bg(theme_selection)
            } else if is_striped {
                striped_row_style
            } else {
                Style::default()
            };

            Row::new(cells).style(row_style).height(1)
        })
        .collect();

    let widths: Vec<Constraint> = if is_very_narrow {
        vec![Constraint::Percentage(60), Constraint::Percentage(40)]
    } else if is_narrow {
        vec![
            Constraint::Min(16),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(7),
        ]
    };

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, inner);

    if rows_len > visible_height {
        let mut scrollbar_state = viewport_scrollbar_state(rows_len, scroll_offset, visible_height);
        let scrollbar = app.glyphs.scrollbar();
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
