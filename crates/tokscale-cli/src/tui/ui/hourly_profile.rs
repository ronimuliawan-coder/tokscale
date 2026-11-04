use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::widgets::{
    format_cache_hit_percent, format_cost, format_tokens, get_client_color,
    get_client_display_name, viewport_scrollbar_state,
};
use crate::tui::app::App;
use crate::tui::data::{aggregate_by_period, aggregate_by_weekday, find_peak_hour};
use crate::tui::i18n::{tr, MessageKey};
use crate::ui_glyphs::ChartFill;
use chrono::Timelike;

fn profile_bar_spans(app: &App, filled: usize, empty: usize) -> Vec<Span<'static>> {
    match app.glyphs.chart_fill {
        ChartFill::Foreground => vec![
            Span::styled(
                app.glyphs.bar_full.repeat(filled),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                app.glyphs.bar_empty.repeat(empty),
                Style::default().fg(app.theme.muted),
            ),
        ],
        ChartFill::Background => vec![
            Span::styled(" ".repeat(filled), Style::default().bg(Color::Green)),
            Span::styled(
                app.glyphs.bar_empty.repeat(empty),
                Style::default().fg(app.theme.muted),
            ),
        ],
    }
}

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let lang = app.settings.tui_language;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(app.glyphs.border)
        .border_style(Style::default().fg(app.theme.border))
        .title(Span::styled(
            tr(lang, MessageKey::TitleHourlyProfile),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(app.theme.background));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.data.hourly.is_empty() {
        let empty_msg = Paragraph::new(tr(lang, MessageKey::EmptyNoHourlyData))
            .style(Style::default().fg(app.theme.muted))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let hourly = &app.data.hourly;
    let total_tokens = app.data.total_tokens;
    let total_cost = app.data.total_cost;

    // Calculate dynamic bar width to use most of the available width
    // Period line: label (10) + hour_range (12) + spaces (4) + percentage (8) = 34 chars overhead
    // Weekday line: day name (10) + spaces (3) + percentage (8) = 21 chars overhead
    // Use period overhead since it's larger, then subtract a margin for safety
    let overhead = 36; // 34 + small margin
    let bar_width = (inner.width as usize)
        .saturating_sub(overhead)
        .clamp(20, 80);

    // Get date range
    let min_date = hourly.iter().map(|h| h.datetime.date()).min();
    let max_date = hourly.iter().map(|h| h.datetime.date()).max();
    let date_range = match (min_date, max_date) {
        (Some(mn), Some(mx)) if mn == mx => mn.format("%Y-%m-%d").to_string(),
        (Some(mn), Some(mx)) => format!("{} to {}", mn.format("%Y-%m-%d"), mx.format("%Y-%m-%d")),
        _ => tr(lang, MessageKey::EmptyNoDataAvailable).to_string(),
    };

    // Aggregate data
    let periods = aggregate_by_period(hourly);
    let weekdays = aggregate_by_weekday(hourly);
    let peak_hour = find_peak_hour(hourly);

    // Build content
    let mut lines: Vec<Line> = Vec::new();

    // Title line
    lines.push(Line::from(vec![
        Span::styled(
            tr(lang, MessageKey::TitleHourlyProfile).trim(),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(&date_range, Style::default().fg(app.theme.muted)),
    ]));
    lines.push(Line::from(""));

    // Summary line
    let (total_input, total_cache_read, total_cache_write) =
        hourly.iter().fold((0u64, 0u64, 0u64), |(i, cr, cw), h| {
            (
                i.saturating_add(h.tokens.input),
                cr.saturating_add(h.tokens.cache_read),
                cw.saturating_add(h.tokens.cache_write),
            )
        });
    let overall_cache_hit =
        format_cache_hit_percent(total_cache_read, total_input, total_cache_write, app.glyphs);

    let summary_spans = vec![
        Span::styled(
            format!("{} {}", hourly.len(), tr(lang, MessageKey::CountHours)),
            app.theme.count_style(),
        ),
        Span::styled(
            format!("  {}  ", app.glyphs.axis_vertical),
            Style::default().fg(app.theme.muted),
        ),
        Span::styled(
            format!(
                "{} {}",
                format_tokens(total_tokens),
                tr(lang, MessageKey::ProfileTotalTokens)
            ),
            app.theme.count_style(),
        ),
        Span::styled(
            format!("  {}  ", app.glyphs.axis_vertical),
            Style::default().fg(app.theme.muted),
        ),
        Span::styled(
            format!(
                "{} {}",
                format_cost(total_cost),
                tr(lang, MessageKey::ProfileTotalCost)
            ),
            Style::default().fg(Color::Green),
        ),
        Span::styled("  |  ", Style::default().fg(app.theme.muted)),
        Span::styled(
            format!("{} cache hit", overall_cache_hit),
            Style::default().fg(Color::Cyan),
        ),
    ];
    lines.push(Line::from(summary_spans));
    lines.push(Line::from(""));

    // Time-of-day breakdown
    lines.push(Line::from(vec![Span::styled(
        tr(lang, MessageKey::ProfileWhenYouWorkMost),
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    let max_period_tokens = periods.iter().map(|p| p.total_tokens).max().unwrap_or(1);

    for period in &periods {
        let percentage = if total_tokens > 0 {
            period.total_tokens as f64 / total_tokens as f64 * 100.0
        } else {
            0.0
        };
        let bar_filled = if max_period_tokens > 0 {
            (period.total_tokens as f64 / max_period_tokens as f64 * bar_width as f64).round()
                as usize
        } else {
            0
        };
        let bar_filled = bar_filled.min(bar_width);
        let bar_empty = bar_width - bar_filled;

        let p_label = match period.label {
            "Morning" => tr(lang, MessageKey::PeriodMorning),
            "Daytime" => tr(lang, MessageKey::PeriodDaytime),
            "Evening" => tr(lang, MessageKey::PeriodEvening),
            "Night" => tr(lang, MessageKey::PeriodNight),
            _ => period.label,
        };

        let p_width = unicode_width::UnicodeWidthStr::width(p_label);
        let p_pad = 10usize.saturating_sub(p_width);
        let p_display = format!("  {}{}", p_label, " ".repeat(p_pad));

        let mut row = vec![
            Span::styled(p_display, Style::default().fg(app.theme.foreground)),
            Span::styled(
                format!("{:>12}", period.hour_range),
                Style::default().fg(app.theme.muted),
            ),
            Span::styled("  ", Style::default()),
        ];
        row.extend(profile_bar_spans(app, bar_filled, bar_empty));
        row.extend([
            Span::styled("  ", Style::default()),
            Span::styled(format!("{:>5.1}%", percentage), app.theme.hint_key_style()),
        ]);
        lines.push(Line::from(row));
    }
    lines.push(Line::from(""));

    // Client Activity & Efficiency
    if !app.data.clients.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Client Activity & Efficiency",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        let mut sorted_clients = app.data.clients.clone();
        sorted_clients.sort_by_key(|a| std::cmp::Reverse(a.tokens.total()));

        for client in sorted_clients.iter().take(6) {
            let client_tokens = client.tokens.total();
            let pct = if total_tokens > 0 {
                client_tokens as f64 / total_tokens as f64 * 100.0
            } else {
                0.0
            };
            let cache_pct = format_cache_hit_percent(
                client.tokens.cache_read,
                client.tokens.input,
                client.tokens.cache_write,
                app.glyphs,
            );

            // Find peak period for this client
            let mut period_tokens = [0u64; 4];
            for entry in hourly {
                let hour = entry.datetime.hour() as usize;
                let p_idx = if (5..=11).contains(&hour) {
                    0
                } else if (12..=16).contains(&hour) {
                    1
                } else if (17..=21).contains(&hour) {
                    2
                } else {
                    3
                };
                if let Some(tokens) = entry.client_breakdown.get(&client.client) {
                    period_tokens[p_idx] = period_tokens[p_idx].saturating_add(tokens.total());
                }
            }
            let period_names = ["Morning", "Daytime", "Evening", "Night"];
            let (max_p_idx, max_tokens) = period_tokens
                .iter()
                .enumerate()
                .max_by_key(|(_, t)| *t)
                .unwrap_or((0, &0));
            let peak_p = if *max_tokens > 0 {
                period_names[max_p_idx]
            } else {
                app.glyphs.missing
            };

            let client_color = get_client_color(&client.client);

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<14}", get_client_display_name(&client.client)),
                    Style::default()
                        .fg(client_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:>6.1}% tokens", pct), app.theme.count_style()),
                Span::styled("  |  Peak: ", Style::default().fg(app.theme.muted)),
                Span::styled(format!("{:<8}", peak_p), app.theme.hint_key_style()),
                Span::styled("  |  Cache Hit: ", Style::default().fg(app.theme.muted)),
                Span::styled(
                    format!("{:>6}", cache_pct),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Weekday breakdown
    lines.push(Line::from(vec![Span::styled(
        tr(lang, MessageKey::ProfileMostProductiveDay),
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    let max_weekday_tokens = weekdays.iter().map(|w| w.total_tokens).max().unwrap_or(1);

    // Find best weekday
    let best_weekday = weekdays
        .iter()
        .max_by_key(|w| w.total_tokens)
        .map(|w| w.day)
        .unwrap_or("Monday");

    for weekday in &weekdays {
        let percentage = if total_tokens > 0 {
            weekday.total_tokens as f64 / total_tokens as f64 * 100.0
        } else {
            0.0
        };
        let bar_filled = if max_weekday_tokens > 0 {
            (weekday.total_tokens as f64 / max_weekday_tokens as f64 * bar_width as f64).round()
                as usize
        } else {
            0
        };
        let bar_filled = bar_filled.min(bar_width);
        let bar_empty = bar_width - bar_filled;

        let is_best = weekday.day == best_weekday;

        let w_label = match weekday.day {
            "Monday" => tr(lang, MessageKey::WeekdayMonday),
            "Tuesday" => tr(lang, MessageKey::WeekdayTuesday),
            "Wednesday" => tr(lang, MessageKey::WeekdayWednesday),
            "Thursday" => tr(lang, MessageKey::WeekdayThursday),
            "Friday" => tr(lang, MessageKey::WeekdayFriday),
            "Saturday" => tr(lang, MessageKey::WeekdaySaturday),
            "Sunday" => tr(lang, MessageKey::WeekdaySunday),
            _ => weekday.day,
        };

        let w_width = unicode_width::UnicodeWidthStr::width(w_label);
        let w_pad = 10usize.saturating_sub(w_width);
        let w_display = format!("  {}{}", w_label, " ".repeat(w_pad));

        let mut row = vec![Span::styled(
            w_display,
            Style::default().fg(if is_best {
                app.theme.hint_key_color()
            } else {
                app.theme.foreground
            }),
        )];
        row.extend(profile_bar_spans(app, bar_filled, bar_empty));
        row.extend([
            Span::styled("  ", Style::default()),
            Span::styled(format!("{:>5.1}%", percentage), app.theme.hint_key_style()),
        ]);
        lines.push(Line::from(row));
    }
    lines.push(Line::from(""));

    // Peak hour insight
    if let Some((hour, tokens, cost)) = peak_hour {
        lines.push(Line::from(vec![
            Span::styled(
                tr(lang, MessageKey::ProfilePeakHour),
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:02}:00-{:02}:59", hour, hour),
                app.theme.hint_key_style(),
            ),
            Span::styled("  (", Style::default().fg(app.theme.muted)),
            Span::styled(format_tokens(tokens), app.theme.count_style()),
            Span::styled(
                format!(" {}, ", tr(lang, MessageKey::ColTokens)),
                Style::default().fg(app.theme.muted),
            ),
            Span::styled(format_cost(cost), Style::default().fg(Color::Green)),
            Span::styled(")", Style::default().fg(app.theme.muted)),
        ]));
    }

    // Legend
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            tr(lang, MessageKey::ProfileLegend),
            Style::default().fg(app.theme.muted),
        ),
        Span::styled(app.glyphs.bar_empty, Style::default().fg(app.theme.muted)),
        Span::styled(
            format!(" {}  ", tr(lang, MessageKey::ProfileLow)),
            Style::default().fg(app.theme.muted),
        ),
        Span::styled(app.glyphs.bar_full, Style::default().fg(Color::Green)),
        Span::styled(
            format!(" {}", tr(lang, MessageKey::ProfileHigh)),
            Style::default().fg(app.theme.muted),
        ),
    ]));

    // Hint
    lines.push(Line::from(""));
    let (prefix, suffix) = match lang {
        crate::tui::i18n::TuiLanguage::Ko => ("", " 키를 눌러 테이블 보기로 전환"),
        crate::tui::i18n::TuiLanguage::Ja => ("", " を押してテーブル表示に切替"),
        crate::tui::i18n::TuiLanguage::ZhCn => ("按 ", " 切换到表格视图"),
        crate::tui::i18n::TuiLanguage::Fr => ("Appuyez sur ", " pour passer en vue tableau"),
        crate::tui::i18n::TuiLanguage::En => ("Press ", " to switch to table view"),
    };
    lines.push(Line::from(vec![
        Span::styled(prefix, Style::default().fg(app.theme.muted)),
        Span::styled("[v]", app.theme.hint_key_style()),
        Span::styled(suffix, Style::default().fg(app.theme.muted)),
    ]));

    let visible_height = inner.height as usize;
    app.hourly_profile_total_lines = lines.len();
    let scroll_offset = app.scroll_offset.min(
        app.hourly_profile_total_lines
            .saturating_sub(visible_height),
    );

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(paragraph, inner);

    if app.hourly_profile_total_lines > visible_height {
        let mut scrollbar_state = viewport_scrollbar_state(
            app.hourly_profile_total_lines,
            scroll_offset,
            visible_height,
        );
        frame.render_stateful_widget(app.glyphs.scrollbar(), area, &mut scrollbar_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::TuiConfig;
    use crate::tui::data::{ClientUsage, HourlySourceInfo, HourlyUsage, TokenBreakdown};
    use chrono::NaiveDateTime;
    use ratatui::{backend::TestBackend, Terminal};
    use std::collections::{BTreeMap, BTreeSet};

    fn make_app() -> App {
        let config = TuiConfig {
            theme: "blue".to_string(),
            refresh: 0,
            sessions_path: None,
            clients: None,
            since: None,
            until: None,
            year: None,
            initial_tab: None,
            ..Default::default()
        };
        App::new_with_cached_data(config, None).unwrap()
    }

    #[test]
    fn hourly_profile_renders_empty_state_cleanly() {
        let mut app = make_app();
        app.data.hourly.clear();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &mut app, Rect::new(0, 0, 80, 24)))
            .unwrap();

        let body = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>();
        assert!(body.contains("No hourly usage data found"));
    }

    #[test]
    fn hourly_profile_renders_client_activity_and_cache_hit() {
        let mut app = make_app();
        let dt = NaiveDateTime::parse_from_str("2026-05-29 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let mut sources = BTreeMap::new();
        sources.insert(
            "opencode".to_string(),
            HourlySourceInfo {
                tokens: TokenBreakdown {
                    input: 100,
                    output: 50,
                    cache_read: 800,
                    cache_write: 50,
                    reasoning: 0,
                },
                cost: 0.05,
                message_count: 5,
                turn_count: 5,
                models: BTreeMap::new(),
            },
        );

        app.data.hourly = vec![HourlyUsage {
            datetime: dt,
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.05,
            clients: BTreeSet::from(["opencode".to_string()]),
            models: BTreeMap::new(),
            source_breakdown: sources,
            client_breakdown: BTreeMap::from([(
                "opencode".to_string(),
                TokenBreakdown {
                    input: 100,
                    output: 50,
                    cache_read: 800,
                    cache_write: 50,
                    reasoning: 0,
                },
            )]),
            message_count: 5,
            turn_count: 5,
        }];
        app.data.total_tokens = 1000;
        app.data.total_cost = 0.05;
        app.data.clients = vec![ClientUsage {
            client: "opencode".to_string(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.05,
            message_count: 5,
            session_count: 1,
            turn_count: 5,
            models: BTreeMap::new(),
            first_active_ms: 0,
            last_active_ms: 0,
        }];

        let backend = TestBackend::new(100, 35);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &mut app, Rect::new(0, 0, 100, 35)))
            .unwrap();

        let body = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect::<String>();

        assert!(body.contains("Client Activity & Efficiency"));
        assert!(body.contains("OpenCode"));
        assert!(body.contains("84.2%")); // 800 / (100 + 800 + 50) = 800 / 950 = 84.2%
        assert!(body.contains("Peak:"));
        assert!(app.hourly_profile_total_lines > 0);
    }
}
