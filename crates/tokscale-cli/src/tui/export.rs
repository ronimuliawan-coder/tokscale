use anyhow::Result;
use serde_json::json;

use super::data::UsageData;

fn cache_hit_pct(read: u64, input: u64, write: u64) -> f64 {
    let total_prompt = input + read + write;
    if total_prompt > 0 {
        ((read as f64 / total_prompt as f64) * 1000.0).round() / 10.0
    } else {
        0.0
    }
}

/// Serializes `UsageData` into the pretty-printed JSON payload used by the
/// `e` export hotkey. Pure: callers are responsible for file I/O and any
/// user-facing status messages.
///
/// Sessions are always the flat per-session list (`UsageData::sessions`):
/// subagent sessions export as their own rows regardless of the Sessions
/// tab roll-up toggle, which only affects the on-screen view.
pub fn build_export_json(data: &UsageData) -> Result<String> {
    let (total_input, total_cache_read, total_cache_write) =
        data.models
            .iter()
            .fold((0u64, 0u64, 0u64), |(i, cr, cw), m| {
                (
                    i + m.tokens.input,
                    cr + m.tokens.cache_read,
                    cw + m.tokens.cache_write,
                )
            });

    let export_data = json!({
        "models": data.models.iter().map(|m| json!({
            "model": m.model,
            "provider": m.provider,
            "client": m.client,
            "tokens": {
                "input": m.tokens.input,
                "output": m.tokens.output,
                "cacheRead": m.tokens.cache_read,
                "cacheWrite": m.tokens.cache_write,
                "total": m.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(m.tokens.cache_read, m.tokens.input, m.tokens.cache_write),
            "cost": m.cost,
            "performance": m.performance,
            "sessionCount": m.session_count
        })).collect::<Vec<_>>(),
        "clients": data.clients.iter().map(|c| json!({
            "client": c.client,
            "tokens": {
                "input": c.tokens.input,
                "output": c.tokens.output,
                "cacheRead": c.tokens.cache_read,
                "cacheWrite": c.tokens.cache_write,
                "reasoning": c.tokens.reasoning,
                "total": c.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(c.tokens.cache_read, c.tokens.input, c.tokens.cache_write),
            "cost": c.cost,
            "messageCount": c.message_count,
            "sessionCount": c.session_count,
            "turnCount": c.turn_count,
            "firstActiveMs": c.first_active_ms,
            "lastActiveMs": c.last_active_ms
        })).collect::<Vec<_>>(),
        "agents": data.agents.iter().map(|a| json!({
            "agent": a.agent,
            "clients": a.clients,
            "tokens": {
                "input": a.tokens.input,
                "output": a.tokens.output,
                "cacheRead": a.tokens.cache_read,
                "cacheWrite": a.tokens.cache_write,
                "total": a.tokens.total()
            },
            "cost": a.cost,
            "messageCount": a.message_count
        })).collect::<Vec<_>>(),
        "daily": data.daily.iter().map(|d| json!({
            "date": d.date.to_string(),
            "tokens": {
                "input": d.tokens.input,
                "output": d.tokens.output,
                "cacheRead": d.tokens.cache_read,
                "cacheWrite": d.tokens.cache_write,
                "total": d.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(d.tokens.cache_read, d.tokens.input, d.tokens.cache_write),
            "messageCount": d.message_count,
            "turnCount": d.turn_count,
            "cost": d.cost
        })).collect::<Vec<_>>(),
        "hourly": data.hourly.iter().map(|h| json!({
            "hour": h.datetime.format("%Y-%m-%d %H:%M").to_string(),
            "tokens": {
                "input": h.tokens.input,
                "output": h.tokens.output,
                "cacheRead": h.tokens.cache_read,
                "cacheWrite": h.tokens.cache_write,
                "reasoning": h.tokens.reasoning,
                "total": h.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(h.tokens.cache_read, h.tokens.input, h.tokens.cache_write),
            "messageCount": h.message_count,
            "turnCount": h.turn_count,
            "cost": h.cost
        })).collect::<Vec<_>>(),
        "monthly": data.monthly.iter().map(|m| json!({
            "month": m.month,
            "tokens": {
                "input": m.tokens.input,
                "output": m.tokens.output,
                "cacheRead": m.tokens.cache_read,
                "cacheWrite": m.tokens.cache_write,
                "total": m.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(m.tokens.cache_read, m.tokens.input, m.tokens.cache_write),
            "messageCount": m.message_count,
            "turnCount": m.turn_count,
            "cost": m.cost
        })).collect::<Vec<_>>(),
        "sessions": data.sessions.iter().map(|s| json!({
            "sessionId": s.session_id,
            "client": s.client,
            "title": s.title,
            "tokens": {
                "input": s.tokens.input,
                "output": s.tokens.output,
                "cacheRead": s.tokens.cache_read,
                "cacheWrite": s.tokens.cache_write,
                "total": s.tokens.total()
            },
            "cacheHitPercent": cache_hit_pct(s.tokens.cache_read, s.tokens.input, s.tokens.cache_write),
            "messageCount": s.message_count,
            "turnCount": s.turn_count,
            "cost": s.cost,
            "firstActiveMs": s.first_active_ms,
            "lastActiveMs": s.last_active_ms
        })).collect::<Vec<_>>(),
        "totals": {
            "tokens": data.total_tokens,
            "cost": data.total_cost,
            "cacheHitPercent": cache_hit_pct(total_cache_read, total_input, total_cache_write)
        }
    });

    Ok(serde_json::to_string_pretty(&export_data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::data::{ClientUsage, HourlyUsage, TokenBreakdown};
    use chrono::NaiveDateTime;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn build_export_json_includes_clients_and_cache_percentages() {
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
            total_cost: 1.25,
            loading: false,
            error: None,
            current_streak: 0,
            longest_streak: 0,
        };

        data.clients.push(ClientUsage {
            client: "opencode".to_string(),
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 1.25,
            message_count: 10,
            session_count: 2,
            turn_count: 8,
            models: BTreeMap::new(),
            first_active_ms: 1000,
            last_active_ms: 2000,
        });

        let dt = NaiveDateTime::parse_from_str("2026-05-29 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        data.hourly.push(HourlyUsage {
            datetime: dt,
            tokens: TokenBreakdown {
                input: 100,
                output: 50,
                cache_read: 800,
                cache_write: 50,
                reasoning: 0,
            },
            cost: 1.25,
            clients: BTreeSet::from(["opencode".to_string()]),
            models: BTreeMap::new(),
            source_breakdown: BTreeMap::new(),
            message_count: 10,
            turn_count: 8,
        });

        let json_str = build_export_json(&data).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(parsed["clients"][0]["client"], "opencode");
        assert_eq!(parsed["clients"][0]["cacheHitPercent"], 84.2);
        assert_eq!(parsed["hourly"][0]["hour"], "2026-05-29 14:00");
        assert_eq!(parsed["hourly"][0]["cacheHitPercent"], 84.2);
    }
}
