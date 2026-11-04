use anyhow::{bail, Result};
use std::io::Write;
use std::path::PathBuf;

use crate::tui::data::{prompt_total_u128, UsageData};
use crate::tui::export::build_export_json;
use crate::tui::ui::widgets::get_client_display_name;

fn escape_csv(val: &str) -> String {
    // Neutralize spreadsheet formula injection: text beginning with =, +, -, @,
    // tab or CR evaluates as a formula when the CSV is opened in a spreadsheet.
    // Prefix those values with an apostrophe; leave everything else untouched.
    let formula_prefix = val.starts_with(['=', '+', '-', '@', '\t', '\r']);
    let escaped = if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r')
    {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    };
    if formula_prefix {
        format!("'{}", escaped)
    } else {
        escaped
    }
}

fn cache_hit_pct_str(read: u64, input: u64, write: u64) -> String {
    let total_prompt = prompt_total_u128(input, read, write);
    if total_prompt > 0 {
        format!("{:.1}%", (read as f64 / total_prompt as f64) * 100.0)
    } else {
        "0.0%".to_string()
    }
}

pub fn export_csv(data: &UsageData, granularity: &str, by_source: bool) -> Result<String> {
    let mut out = String::new();

    match granularity.to_lowercase().as_str() {
        "daily" => {
            if by_source {
                out.push_str("Date,Source,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n");
                let mut sorted_daily = data.daily.clone();
                sorted_daily.sort_by_key(|d| d.date);
                for day in sorted_daily {
                    let date_str = day.date.to_string();
                    if day.source_breakdown.is_empty() {
                        let hit_pct = cache_hit_pct_str(
                            day.tokens.cache_read,
                            day.tokens.input,
                            day.tokens.cache_write,
                        );
                        out.push_str(&format!(
                            "{},{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                            escape_csv(&date_str),
                            escape_csv("total"),
                            day.tokens.input,
                            day.tokens.output,
                            day.tokens.cache_read,
                            day.tokens.cache_write,
                            day.tokens.reasoning,
                            day.tokens.total(),
                            hit_pct,
                            day.cost,
                            day.message_count,
                            day.turn_count,
                        ));
                    } else {
                        for (source, info) in day.source_breakdown {
                            let hit_pct = cache_hit_pct_str(
                                info.tokens.cache_read,
                                info.tokens.input,
                                info.tokens.cache_write,
                            );
                            out.push_str(&format!(
                                "{},{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                                escape_csv(&date_str),
                                escape_csv(&source),
                                info.tokens.input,
                                info.tokens.output,
                                info.tokens.cache_read,
                                info.tokens.cache_write,
                                info.tokens.reasoning,
                                info.tokens.total(),
                                hit_pct,
                                info.cost,
                                info.message_count,
                                info.turn_count,
                            ));
                        }
                    }
                }
            } else {
                out.push_str("Date,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n");
                let mut sorted_daily = data.daily.clone();
                sorted_daily.sort_by_key(|d| d.date);
                for day in sorted_daily {
                    let hit_pct = cache_hit_pct_str(
                        day.tokens.cache_read,
                        day.tokens.input,
                        day.tokens.cache_write,
                    );
                    out.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                        escape_csv(&day.date.to_string()),
                        day.tokens.input,
                        day.tokens.output,
                        day.tokens.cache_read,
                        day.tokens.cache_write,
                        day.tokens.reasoning,
                        day.tokens.total(),
                        hit_pct,
                        day.cost,
                        day.message_count,
                        day.turn_count,
                    ));
                }
            }
        }
        "hourly" => {
            if by_source {
                out.push_str("Timestamp,Source,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n");
                let mut sorted_hourly = data.hourly.clone();
                sorted_hourly.sort_by_key(|h| h.datetime);
                for h in sorted_hourly {
                    let time_str = h.datetime.format("%Y-%m-%d %H:%M").to_string();
                    if h.source_breakdown.is_empty() {
                        let hit_pct = cache_hit_pct_str(
                            h.tokens.cache_read,
                            h.tokens.input,
                            h.tokens.cache_write,
                        );
                        out.push_str(&format!(
                            "{},{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                            escape_csv(&time_str),
                            escape_csv("total"),
                            h.tokens.input,
                            h.tokens.output,
                            h.tokens.cache_read,
                            h.tokens.cache_write,
                            h.tokens.reasoning,
                            h.tokens.total(),
                            hit_pct,
                            h.cost,
                            h.message_count,
                            h.turn_count,
                        ));
                    } else {
                        for (source, info) in h.source_breakdown {
                            let hit_pct = cache_hit_pct_str(
                                info.tokens.cache_read,
                                info.tokens.input,
                                info.tokens.cache_write,
                            );
                            out.push_str(&format!(
                                "{},{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                                escape_csv(&time_str),
                                escape_csv(&source),
                                info.tokens.input,
                                info.tokens.output,
                                info.tokens.cache_read,
                                info.tokens.cache_write,
                                info.tokens.reasoning,
                                info.tokens.total(),
                                hit_pct,
                                info.cost,
                                info.message_count,
                                info.turn_count,
                            ));
                        }
                    }
                }
            } else {
                out.push_str("Timestamp,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n");
                let mut sorted_hourly = data.hourly.clone();
                sorted_hourly.sort_by_key(|h| h.datetime);
                for h in sorted_hourly {
                    let time_str = h.datetime.format("%Y-%m-%d %H:%M").to_string();
                    let hit_pct = cache_hit_pct_str(
                        h.tokens.cache_read,
                        h.tokens.input,
                        h.tokens.cache_write,
                    );
                    out.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                        escape_csv(&time_str),
                        h.tokens.input,
                        h.tokens.output,
                        h.tokens.cache_read,
                        h.tokens.cache_write,
                        h.tokens.reasoning,
                        h.tokens.total(),
                        hit_pct,
                        h.cost,
                        h.message_count,
                        h.turn_count,
                    ));
                }
            }
        }
        "monthly" => {
            out.push_str("Month,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n");
            let mut sorted_monthly = data.monthly.clone();
            sorted_monthly.sort_by(|a, b| a.month.cmp(&b.month));
            for m in sorted_monthly {
                let hit_pct =
                    cache_hit_pct_str(m.tokens.cache_read, m.tokens.input, m.tokens.cache_write);
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{:.4},{},{}\n",
                    escape_csv(&m.month),
                    m.tokens.input,
                    m.tokens.output,
                    m.tokens.cache_read,
                    m.tokens.cache_write,
                    m.tokens.reasoning,
                    m.tokens.total(),
                    hit_pct,
                    m.cost,
                    m.message_count,
                    m.turn_count,
                ));
            }
        }
        "models" => {
            out.push_str("Model,Provider,Client,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,SessionCount\n");
            let mut sorted_models = data.models.clone();
            sorted_models.sort_by(|a, b| b.cost.total_cmp(&a.cost));
            for m in sorted_models {
                let hit_pct =
                    cache_hit_pct_str(m.tokens.cache_read, m.tokens.input, m.tokens.cache_write);
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{:.4},{}\n",
                    escape_csv(&m.model),
                    escape_csv(&m.provider),
                    escape_csv(&m.client),
                    m.tokens.input,
                    m.tokens.output,
                    m.tokens.cache_read,
                    m.tokens.cache_write,
                    m.tokens.reasoning,
                    m.tokens.total(),
                    hit_pct,
                    m.cost,
                    m.session_count,
                ));
            }
        }
        "clients" => {
            out.push_str("Client,DisplayName,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,SessionCount,TurnCount\n");
            let mut sorted_clients = data.clients.clone();
            sorted_clients.sort_by(|a, b| b.cost.total_cmp(&a.cost));
            for c in sorted_clients {
                let display_name = get_client_display_name(&c.client);
                let hit_pct =
                    cache_hit_pct_str(c.tokens.cache_read, c.tokens.input, c.tokens.cache_write);
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{:.4},{},{},{}\n",
                    escape_csv(&c.client),
                    escape_csv(&display_name),
                    c.tokens.input,
                    c.tokens.output,
                    c.tokens.cache_read,
                    c.tokens.cache_write,
                    c.tokens.reasoning,
                    c.tokens.total(),
                    hit_pct,
                    c.cost,
                    c.message_count,
                    c.session_count,
                    c.turn_count,
                ));
            }
        }
        "sessions" => {
            out.push_str("SessionId,Client,Title,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount,FirstActive,LastActive\n");
            let mut sorted_sessions = data.sessions.clone();
            sorted_sessions.sort_by(|a, b| b.cost.total_cmp(&a.cost));
            for s in sorted_sessions {
                let title = s.title.as_deref().unwrap_or("");
                let hit_pct =
                    cache_hit_pct_str(s.tokens.cache_read, s.tokens.input, s.tokens.cache_write);
                let first_active = chrono::DateTime::from_timestamp_millis(s.first_active_ms)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default();
                let last_active = chrono::DateTime::from_timestamp_millis(s.last_active_ms)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default();
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{:.4},{},{},{},{}\n",
                    escape_csv(&s.session_id),
                    escape_csv(&s.client),
                    escape_csv(title),
                    s.tokens.input,
                    s.tokens.output,
                    s.tokens.cache_read,
                    s.tokens.cache_write,
                    s.tokens.reasoning,
                    s.tokens.total(),
                    hit_pct,
                    s.cost,
                    s.message_count,
                    s.turn_count,
                    escape_csv(&first_active),
                    escape_csv(&last_active),
                ));
            }
        }
        other => {
            bail!("Unsupported granularity '{}'. Allowed: hourly, daily, monthly, models, clients, sessions", other);
        }
    }

    Ok(out)
}

pub fn export_json(data: &UsageData, granularity: &str) -> Result<String> {
    match granularity.to_lowercase().as_str() {
        "all" | "full" => build_export_json(data),
        "clients" => {
            let clients: Vec<serde_json::Value> = data
                .clients
                .iter()
                .map(|c| {
                    let total_prompt = prompt_total_u128(c.tokens.input, c.tokens.cache_read, c.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((c.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "client": c.client,
                        "displayName": get_client_display_name(&c.client),
                        "tokens": {
                            "input": c.tokens.input,
                            "output": c.tokens.output,
                            "cacheRead": c.tokens.cache_read,
                            "cacheWrite": c.tokens.cache_write,
                            "reasoning": c.tokens.reasoning,
                            "total": c.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "cost": c.cost,
                        "messageCount": c.message_count,
                        "sessionCount": c.session_count,
                        "turnCount": c.turn_count,
                        "firstActiveMs": c.first_active_ms,
                        "lastActiveMs": c.last_active_ms
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&clients)?)
        }
        "models" => {
            let models: Vec<serde_json::Value> = data
                .models
                .iter()
                .map(|m| {
                    let total_prompt = prompt_total_u128(m.tokens.input, m.tokens.cache_read, m.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((m.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "model": m.model,
                        "provider": m.provider,
                        "client": m.client,
                        "tokens": {
                            "input": m.tokens.input,
                            "output": m.tokens.output,
                            "cacheRead": m.tokens.cache_read,
                            "cacheWrite": m.tokens.cache_write,
                            "reasoning": m.tokens.reasoning,
                            "total": m.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "cost": m.cost,
                        "performance": m.performance,
                        "sessionCount": m.session_count
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&models)?)
        }
        "daily" => {
            let daily: Vec<serde_json::Value> = data
                .daily
                .iter()
                .map(|d| {
                    let total_prompt = prompt_total_u128(d.tokens.input, d.tokens.cache_read, d.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((d.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "date": d.date.to_string(),
                        "tokens": {
                            "input": d.tokens.input,
                            "output": d.tokens.output,
                            "cacheRead": d.tokens.cache_read,
                            "cacheWrite": d.tokens.cache_write,
                            "reasoning": d.tokens.reasoning,
                            "total": d.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "messageCount": d.message_count,
                        "turnCount": d.turn_count,
                        "cost": d.cost,
                        "sourceBreakdown": d.source_breakdown.iter().map(|(src, info)| {
                            (src.clone(), serde_json::json!({
                                "tokens": {
                                    "input": info.tokens.input,
                                    "output": info.tokens.output,
                                    "cacheRead": info.tokens.cache_read,
                                    "cacheWrite": info.tokens.cache_write,
                                    "reasoning": info.tokens.reasoning,
                                    "total": info.tokens.total()
                                },
                                "cost": info.cost
                            }))
                        }).collect::<serde_json::Map<String, serde_json::Value>>()
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&daily)?)
        }
        "hourly" => {
            let hourly: Vec<serde_json::Value> = data
                .hourly
                .iter()
                .map(|h| {
                    let total_prompt = prompt_total_u128(h.tokens.input, h.tokens.cache_read, h.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((h.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "hour": h.datetime.format("%Y-%m-%d %H:%M").to_string(),
                        "tokens": {
                            "input": h.tokens.input,
                            "output": h.tokens.output,
                            "cacheRead": h.tokens.cache_read,
                            "cacheWrite": h.tokens.cache_write,
                            "reasoning": h.tokens.reasoning,
                            "total": h.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "messageCount": h.message_count,
                        "turnCount": h.turn_count,
                        "cost": h.cost,
                        "sourceBreakdown": h.source_breakdown.iter().map(|(src, info)| {
                            (src.clone(), serde_json::json!({
                                "tokens": {
                                    "input": info.tokens.input,
                                    "output": info.tokens.output,
                                    "cacheRead": info.tokens.cache_read,
                                    "cacheWrite": info.tokens.cache_write,
                                    "reasoning": info.tokens.reasoning,
                                    "total": info.tokens.total()
                                },
                                "cost": info.cost,
                                "messageCount": info.message_count,
                                "turnCount": info.turn_count
                            }))
                        }).collect::<serde_json::Map<String, serde_json::Value>>()
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&hourly)?)
        }
        "monthly" => {
            let monthly: Vec<serde_json::Value> = data
                .monthly
                .iter()
                .map(|m| {
                    let total_prompt = prompt_total_u128(m.tokens.input, m.tokens.cache_read, m.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((m.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "month": m.month,
                        "tokens": {
                            "input": m.tokens.input,
                            "output": m.tokens.output,
                            "cacheRead": m.tokens.cache_read,
                            "cacheWrite": m.tokens.cache_write,
                            "reasoning": m.tokens.reasoning,
                            "total": m.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "messageCount": m.message_count,
                        "turnCount": m.turn_count,
                        "cost": m.cost
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&monthly)?)
        }
        "sessions" => {
            let sessions: Vec<serde_json::Value> = data
                .sessions
                .iter()
                .map(|s| {
                    let total_prompt = prompt_total_u128(s.tokens.input, s.tokens.cache_read, s.tokens.cache_write);
                    let cache_hit_pct = if total_prompt > 0 {
                        ((s.tokens.cache_read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
                    } else {
                        0.0
                    };
                    serde_json::json!({
                        "sessionId": s.session_id,
                        "client": s.client,
                        "title": s.title,
                        "tokens": {
                            "input": s.tokens.input,
                            "output": s.tokens.output,
                            "cacheRead": s.tokens.cache_read,
                            "cacheWrite": s.tokens.cache_write,
                            "reasoning": s.tokens.reasoning,
                            "total": s.tokens.total()
                        },
                        "cacheHitPercent": cache_hit_pct,
                        "messageCount": s.message_count,
                        "turnCount": s.turn_count,
                        "cost": s.cost,
                        "firstActiveMs": s.first_active_ms,
                        "lastActiveMs": s.last_active_ms
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&sessions)?)
        }
        other => {
            bail!("Unsupported granularity '{}'. Allowed: hourly, daily, monthly, models, clients, sessions, all", other);
        }
    }
}

pub fn export_data(
    data: &UsageData,
    format: &str,
    granularity: &str,
    by_source: bool,
    output: Option<PathBuf>,
) -> Result<()> {
    let content = match format.to_lowercase().as_str() {
        "csv" => export_csv(data, granularity, by_source)?,
        "json" => export_json(data, granularity)?,
        other => bail!("Unsupported export format '{}'. Allowed: csv, json", other),
    };

    if let Some(path) = output {
        std::fs::write(&path, &content)?;
        println!("Exported {} data to {}", granularity, path.display());
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(content.as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::data::{ClientUsage, DailySourceInfo, DailyUsage, TokenBreakdown};
    use chrono::NaiveDate;
    use std::collections::BTreeMap;

    fn test_usage_data() -> UsageData {
        let mut data = UsageData {
            models: Vec::new(),
            agents: Vec::new(),
            daily: Vec::new(),
            hourly: Vec::new(),
            minutely: Vec::new(),
            monthly: Vec::new(),
            sessions: Vec::new(),
            sessions_rolled: Vec::new(),
            projects: Vec::new(),
            clients: Vec::new(),
            graph: None,
            total_tokens: 1000,
            undated_tokens: 0,
            total_cost: 0.50,
            loading: false,
            error: None,
            current_streak: 0,
            longest_streak: 0,
        };

        let mut sources = BTreeMap::new();
        sources.insert(
            "opencode".to_string(),
            DailySourceInfo {
                tokens: TokenBreakdown {
                    input: 100,
                    output: 50,
                    cache_read: 800,
                    cache_write: 50,
                    reasoning: 0,
                },
                cost: 0.50,
                models: BTreeMap::new(),
                message_count: 5,
                turn_count: 5,
            },
        );

        data.daily.push(DailyUsage {
            date: NaiveDate::from_ymd_opt(2026, 5, 29).unwrap(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            source_breakdown: sources,
            message_count: 5,
            turn_count: 5,
        });

        data.clients.push(ClientUsage {
            client: "opencode".to_string(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            message_count: 5,
            session_count: 1,
            turn_count: 5,
            models: BTreeMap::new(),
            first_active_ms: 0,
            last_active_ms: 0,
        });

        data
    }

    #[test]
    fn export_csv_daily_aggregated() {
        let data = test_usage_data();
        let csv = export_csv(&data, "daily", false).unwrap();
        assert!(csv.starts_with("Date,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n"));
        assert!(csv.contains("2026-05-29,100,50,800,50,0,1000,84.2%,0.5000,5,5"));
    }

    #[test]
    fn export_csv_daily_by_source() {
        let data = test_usage_data();
        let csv = export_csv(&data, "daily", true).unwrap();
        assert!(csv.starts_with("Date,Source,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n"));
        assert!(csv.contains("2026-05-29,opencode,100,50,800,50,0,1000,84.2%,0.5000,5,5"));
    }

    #[test]
    fn export_csv_daily_by_source_counts_are_per_source() {
        let mut data = test_usage_data();
        data.daily[0].source_breakdown.insert(
            "claude".to_string(),
            DailySourceInfo {
                tokens: TokenBreakdown::default(),
                cost: 0.0,
                models: BTreeMap::new(),
                message_count: 2,
                turn_count: 2,
            },
        );
        data.daily[0].source_breakdown.get_mut("opencode").unwrap().message_count = 3;
        data.daily[0].source_breakdown.get_mut("opencode").unwrap().turn_count = 3;

        let csv = export_csv(&data, "daily", true).unwrap();
        let rows: Vec<&str> = csv.lines().skip(1).collect();
        assert_eq!(rows.len(), 2, "one row per source");
        let opencode = rows.iter().find(|r| r.contains(",opencode,")).unwrap();
        let claude = rows.iter().find(|r| r.contains(",claude,")).unwrap();
        // Each source row carries its own counts, not the day's 5/5 totals.
        assert!(opencode.ends_with(",3,3"), "{opencode}");
        assert!(claude.ends_with(",2,2"), "{claude}");
    }

    #[test]
    fn export_csv_clients() {
        let data = test_usage_data();
        let csv = export_csv(&data, "clients", false).unwrap();
        assert!(csv.starts_with("Client,DisplayName,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,SessionCount,TurnCount\n"));
        assert!(csv.contains("opencode,OpenCode,100,50,800,50,0,1000,84.2%,0.5000,5,1,5"));
    }

    #[test]
    fn export_json_clients() {
        let data = test_usage_data();
        let json_str = export_json(&data, "clients").unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed[0]["client"], "opencode");
        assert_eq!(parsed[0]["cacheHitPercent"], 84.2);
    }

    #[test]
    fn export_csv_hourly_aggregated_and_by_source() {
        use crate::tui::data::{HourlySourceInfo, HourlyUsage};
        use chrono::NaiveDateTime;
        let mut data = test_usage_data();
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
                cost: 0.50,
                message_count: 5,
                turn_count: 5,
                models: BTreeMap::new(),
            },
        );
        data.hourly.push(HourlyUsage {
            datetime: dt,
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            clients: Default::default(),
            models: BTreeMap::new(),
            source_breakdown: sources,
            message_count: 5,
            turn_count: 5,
        });

        let csv_agg = export_csv(&data, "hourly", false).unwrap();
        assert!(csv_agg.starts_with("Timestamp,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n"));
        assert!(csv_agg.contains("2026-05-29 14:00,100,50,800,50,0,1000,84.2%,0.5000,5,5"));

        let csv_src = export_csv(&data, "hourly", true).unwrap();
        assert!(csv_src.starts_with("Timestamp,Source,InputTokens,OutputTokens,CacheReadTokens,CacheWriteTokens,ReasoningTokens,TotalTokens,CacheHitPercent,Cost,MessageCount,TurnCount\n"));
        assert!(csv_src.contains("2026-05-29 14:00,opencode,100,50,800,50,0,1000,84.2%,0.5000,5,5"));
    }

    #[test]
    fn export_csv_monthly_models_sessions() {
        use crate::tui::data::{ModelUsage, MonthlyUsage, SessionUsage};
        use tokscale_core::ModelPerformance;
        let mut data = test_usage_data();
        data.monthly.push(MonthlyUsage {
            month: "2026-05".to_string(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            message_count: 5,
            turn_count: 5,
        });
        data.models.push(ModelUsage {
            model: "claude-3-5-sonnet".to_string(),
            color_key: "claude-3-5-sonnet".to_string(),
            provider: "anthropic".to_string(),
            client: "opencode".to_string(),
            workspace_key: None,
            workspace_label: None,
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            performance: ModelPerformance::default(),
            session_count: 1,
            machine: None,
            group_key: String::new(),
            daily: Vec::new(),
        });
        data.sessions.push(SessionUsage {
            session_id: "sess-1".to_string(),
            client: "opencode".to_string(),
            title: Some("Test Session".to_string()),
            workspace_key: None,
            workspace_label: None,
            agents: Vec::new(),
            models: Vec::new(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 0.50,
            message_count: 5,
            turn_count: 5,
            first_active_ms: 1000,
            last_active_ms: 2000,
            subagent_count: 0,
        });

        let csv_monthly = export_csv(&data, "monthly", false).unwrap();
        assert!(csv_monthly.contains("2026-05,100,50,800,50,0,1000,84.2%,0.5000,5,5"));

        let csv_models = export_csv(&data, "models", false).unwrap();
        assert!(csv_models
            .contains("claude-3-5-sonnet,anthropic,opencode,100,50,800,50,0,1000,84.2%,0.5000,1"));

        let csv_sessions = export_csv(&data, "sessions", false).unwrap();
        assert!(csv_sessions
            .contains("sess-1,opencode,Test Session,100,50,800,50,0,1000,84.2%,0.5000,5,5"));
    }

    #[test]
    fn escape_csv_neutralizes_formula_prefixes() {
        assert_eq!(escape_csv("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(escape_csv("+1+1"), "'+1+1");
        assert_eq!(escape_csv("@import"), "'@import");
        assert_eq!(escape_csv("-cmd"), "'-cmd");
        // Formula character plus a delimiter: prefix outside the quoting.
        assert_eq!(escape_csv("=cmd,1"), "'\"=cmd,1\"");
        // Ordinary data is untouched (dates and timestamps never lead with a flag).
        assert_eq!(escape_csv("2026-05-29"), "2026-05-29");
        assert_eq!(escape_csv("plain title"), "plain title");
        assert_eq!(escape_csv("a,\"b\""), "\"a,\"\"b\"\"\"");
    }

    #[test]
    fn export_csv_sessions_hostile_title() {
        let mut data = test_usage_data();
        data.sessions.push(crate::tui::data::SessionUsage {
            session_id: "sess-evil".to_string(),
            client: "opencode".to_string(),
            title: Some("=HYPERLINK(\"http://evil\")".to_string()),
            workspace_key: None,
            workspace_label: None,
            agents: Vec::new(),
            models: Vec::new(),
            tokens: Default::default(),
            cost: 0.0,
            message_count: 1,
            turn_count: 1,
            first_active_ms: 0,
            last_active_ms: 0,
            subagent_count: 0,
        });
        let csv = export_csv(&data, "sessions", false).unwrap();
        assert!(csv.contains("'\"=HYPERLINK(\"\"http://evil\"\")\""));
    }

    #[test]
    fn export_data_writes_to_file() {
        let data = test_usage_data();
        let temp_dir = tempfile::tempdir().unwrap();
        let target_file = temp_dir.path().join("export_test.csv");
        export_data(&data, "csv", "daily", false, Some(target_file.clone())).unwrap();
        assert!(target_file.exists());
        let content = std::fs::read_to_string(&target_file).unwrap();
        assert!(content.contains("2026-05-29,100,50,800,50,0,1000,84.2%"));
    }
}
