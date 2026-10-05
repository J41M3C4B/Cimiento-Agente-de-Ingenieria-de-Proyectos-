//! Real-time AI metrics read from `ai_usage`: what each model handled, how long it took,
//! how much of its allowance is used and what it would cost. Nothing here calls the provider.

use super::pipeline::Window;
use super::settings::{self, AiSettings, ProviderKind, RateLimit};
use super::ModelTier;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Attempts that never left the computer (no internet) do not count against any allowance.
const REACHED_PROVIDER: &str = "COALESCE(error_kind,'') != 'offline'";

/// What counts against the provider's allowance: also not the attempts the service itself failed
/// (5xx, "high demand"), which it does not charge to the key. They stay in the report.
const COUNTS_AGAINST_ALLOWANCE: &str = "COALESCE(error_kind,'') != 'offline' AND COALESCE(error_kind,'') NOT LIKE 'http_5%'";

/// Calls and text of `model` in the last `seconds`.
pub fn window(conn: &Connection, model: &str, seconds: u32) -> rusqlite::Result<Window> {
    conn.query_row(
        &format!(
            "SELECT COUNT(*),
                    COALESCE(SUM(input_tokens + output_tokens + cached_tokens), 0),
                    COALESCE(CAST(strftime('%s','now') AS INTEGER) - MIN(CAST(strftime('%s', at) AS INTEGER)), 0)
             FROM ai_usage
             WHERE model = ?1 AND {COUNTS_AGAINST_ALLOWANCE}
               AND at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-' || ?2 || ' seconds')"
        ),
        params![model, seconds],
        |r| {
            Ok(Window {
                calls: r.get::<_, i64>(0)? as u32,
                tokens: r.get::<_, i64>(1)? as u64,
                oldest_age_secs: r.get::<_, i64>(2)?.max(0) as u32,
            })
        },
    )
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModelUsage {
    pub provider: String,
    pub model: String,
    /// Which tier the active provider uses this model for, if any.
    pub tier: Option<ModelTier>,
    /// It is a backup of that tier, not its main model (ADR-008).
    pub is_fallback: bool,
    pub calls_last_minute: u32,
    pub calls_last_day: u32,
    pub tokens_last_minute: u64,
    pub limit: Option<RateLimit>,
    pub calls_total: u32,
    pub failed_total: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub thought_tokens: u64,
    pub cached_tokens: u64,
    pub avg_latency_ms: Option<u64>,
    pub p95_latency_ms: Option<u64>,
    pub cost_mxn: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TaskUsage {
    pub task: String,
    pub calls: u32,
    pub avg_latency_ms: Option<u64>,
    pub avg_input_tokens: u64,
    pub avg_output_tokens: u64,
    pub avg_thought_tokens: u64,
    pub cost_mxn: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecentCall {
    pub at: String,
    pub task: String,
    pub model: String,
    pub latency_ms: Option<u64>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub thought_tokens: u64,
    pub ok: bool,
    pub error_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UsageReport {
    pub provider: ProviderKind,
    pub month_spend_mxn: f64,
    pub cap_mxn: f64,
    pub models: Vec<ModelUsage>,
    pub tasks: Vec<TaskUsage>,
    pub recent: Vec<RecentCall>,
}

fn percentile(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((sorted.len() as f64) * p).ceil() as usize;
    Some(sorted[rank.clamp(1, sorted.len()) - 1])
}

fn model_row(conn: &Connection, s: &AiSettings, provider: &str, model: &str) -> rusqlite::Result<ModelUsage> {
    let (calls, failed, input, output, thought, cached, cost) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(success = 0), 0), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0),
                    COALESCE(SUM(thought_tokens), 0), COALESCE(SUM(cached_tokens), 0), COALESCE(SUM(estimated_cost_mxn), 0)
             FROM ai_usage WHERE provider = ?1 AND model = ?2 AND {REACHED_PROVIDER}"
        ),
        params![provider, model],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?, r.get::<_, i64>(3)?, r.get::<_, i64>(4)?, r.get::<_, i64>(5)?, r.get::<_, f64>(6)?)),
    )?;
    let mut stmt = conn.prepare(
        "SELECT latency_ms FROM ai_usage WHERE provider = ?1 AND model = ?2 AND success = 1 AND latency_ms IS NOT NULL ORDER BY latency_ms",
    )?;
    let latencies: Vec<u64> = stmt.query_map(params![provider, model], |r| r.get::<_, i64>(0).map(|v| v as u64))?.collect::<Result<_, _>>()?;
    let minute = window(conn, model, 60)?;
    let day = window(conn, model, 86_400)?;
    let active = ProviderKind::parse(provider).filter(|p| *p == s.provider);
    // the tier whose chain holds this model, and whether it is there as a backup
    let (tier, is_fallback) = active
        .and_then(|p| {
            [ModelTier::Light, ModelTier::Strong].into_iter().find_map(|t| {
                s.chain_of(p, t).iter().position(|m| m == model).map(|i| (t, i > 0))
            })
        })
        .map_or((None, false), |(t, backup)| (Some(t), backup));
    Ok(ModelUsage {
        provider: provider.to_string(),
        model: model.to_string(),
        tier,
        is_fallback,
        calls_last_minute: minute.calls,
        calls_last_day: day.calls,
        tokens_last_minute: minute.tokens,
        limit: s.rate_limits.get(model).copied(),
        calls_total: calls as u32,
        failed_total: failed as u32,
        input_tokens: input as u64,
        output_tokens: output as u64,
        thought_tokens: thought as u64,
        cached_tokens: cached as u64,
        avg_latency_ms: (!latencies.is_empty()).then(|| latencies.iter().sum::<u64>() / latencies.len() as u64),
        p95_latency_ms: percentile(&latencies, 0.95),
        cost_mxn: cost,
    })
}

pub fn usage_report(conn: &Connection, s: &AiSettings) -> rusqlite::Result<UsageReport> {
    // Every model that has been used, newest first, plus the two the active provider would use now
    // (so the allowance bars show up before the first call).
    let mut pairs: Vec<(String, String)> = {
        let mut stmt = conn.prepare(&format!(
            "SELECT provider, model FROM ai_usage WHERE {REACHED_PROVIDER} GROUP BY provider, model ORDER BY MAX(rowid) DESC"
        ))?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.collect::<Result<_, _>>()?;
        rows
    };
    for tier in [ModelTier::Light, ModelTier::Strong] {
        let pair = (s.provider.as_str().to_string(), s.model_for(tier).to_string());
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }
    let models = pairs.iter().map(|(p, m)| model_row(conn, s, p, m)).collect::<rusqlite::Result<Vec<_>>>()?;

    let tasks = {
        let mut stmt = conn.prepare(&format!(
            "SELECT task, COUNT(*), CAST(AVG(CASE WHEN success = 1 THEN latency_ms END) AS INTEGER),
                    CAST(AVG(input_tokens) AS INTEGER), CAST(AVG(output_tokens) AS INTEGER), CAST(AVG(thought_tokens) AS INTEGER),
                    COALESCE(SUM(estimated_cost_mxn), 0)
             FROM ai_usage WHERE {REACHED_PROVIDER} AND success = 1 GROUP BY task ORDER BY task"
        ))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(TaskUsage {
                    task: r.get(0)?,
                    calls: r.get::<_, i64>(1)? as u32,
                    avg_latency_ms: r.get::<_, Option<i64>>(2)?.map(|v| v as u64),
                    avg_input_tokens: r.get::<_, i64>(3)? as u64,
                    avg_output_tokens: r.get::<_, i64>(4)? as u64,
                    avg_thought_tokens: r.get::<_, i64>(5)? as u64,
                    cost_mxn: r.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    let recent = {
        let mut stmt = conn.prepare(
            "SELECT at, task, model, latency_ms, input_tokens, output_tokens, thought_tokens, success, error_kind
             FROM ai_usage ORDER BY rowid DESC LIMIT 20",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RecentCall {
                    at: r.get(0)?,
                    task: r.get(1)?,
                    model: r.get(2)?,
                    latency_ms: r.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                    input_tokens: r.get::<_, i64>(4)? as u64,
                    output_tokens: r.get::<_, i64>(5)? as u64,
                    thought_tokens: r.get::<_, i64>(6)? as u64,
                    ok: r.get::<_, i64>(7)? == 1,
                    error_kind: r.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    Ok(UsageReport {
        provider: s.provider,
        month_spend_mxn: settings::month_spend_mxn(conn)?,
        cap_mxn: s.monthly_cap_mxn,
        models,
        tasks,
        recent,
    })
}

/// A plain-text table for the terminal (golden-case runs print it).
pub fn format_report(r: &UsageReport) -> String {
    let ms = |v: Option<u64>| v.map_or("-".to_string(), |v| format!("{v} ms"));
    let mut out = String::new();
    out.push_str(&format!("Gasto del mes: ${:.4} de ${:.2} MXN (proveedor activo: {:?})\n\n", r.month_spend_mxn, r.cap_mxn, r.provider));
    out.push_str("Por modelo\n");
    for m in &r.models {
        let limit = m.limit.map_or("sin límite conocido".to_string(), |l| {
            format!("{}/{} por minuto, {}/{} por día", m.calls_last_minute, l.per_minute, m.calls_last_day, l.per_day)
        });
        out.push_str(&format!(
            "  {} ({}): {} llamadas ({} fallidas) | entrada {} · salida {} (de ellos razonamiento {}) · caché {} | tiempo medio {} · p95 {} | ${:.4} MXN | {limit}\n",
            m.model, m.provider, m.calls_total, m.failed_total, m.input_tokens, m.output_tokens, m.thought_tokens, m.cached_tokens,
            ms(m.avg_latency_ms), ms(m.p95_latency_ms), m.cost_mxn
        ));
    }
    out.push_str("\nPor tarea (llamadas válidas)\n");
    for t in &r.tasks {
        out.push_str(&format!(
            "  {}: {} llamadas | tiempo medio {} | por llamada: entrada {} · salida {} (razonamiento {}) | ${:.4} MXN\n",
            t.task, t.calls, ms(t.avg_latency_ms), t.avg_input_tokens, t.avg_output_tokens, t.avg_thought_tokens, t.cost_mxn
        ));
    }
    // Why each attempt ended the way it did: the only place a failed call is explained.
    out.push_str("\nÚltimos intentos (el más reciente primero)\n");
    for c in r.recent.iter().take(12) {
        let result = if c.ok { "bien".to_string() } else { c.error_kind.clone().unwrap_or_else(|| "falló".into()) };
        out.push_str(&format!("  {} | {} | {} | {} | {result}\n", c.at, c.task, c.model, ms(c.latency_ms)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(include_str!("../../migrations/0001_initial.sql")).unwrap();
        c.execute_batch(include_str!("../../migrations/0003_ai_metrics.sql")).unwrap();
        c
    }

    #[allow(clippy::too_many_arguments)]
    fn add(c: &Connection, at: &str, task: &str, provider: &str, model: &str, i: i64, o: i64, th: i64, ms: Option<i64>, ok: bool, kind: Option<&str>, cost: f64) {
        c.execute(
            "INSERT INTO ai_usage (id,at,task,provider,model,input_tokens,output_tokens,thought_tokens,latency_ms,success,error_kind,estimated_cost_mxn)
             VALUES (lower(hex(randomblob(8))),?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![at, task, provider, model, i, o, th, ms, ok as i64, kind, cost],
        )
        .unwrap();
    }

    const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%SZ','now')";

    fn now(c: &Connection) -> String {
        c.query_row(&format!("SELECT {NOW}"), [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn window_counts_recent_calls_and_ignores_old_offline_and_overloaded_ones() {
        let c = conn();
        let n = now(&c);
        add(&c, &n, "t", "gemini", "m", 100, 20, 0, Some(10), true, None, 0.0);
        add(&c, &n, "t", "gemini", "m", 100, 20, 0, Some(10), false, Some("rate_limited"), 0.0);
        add(&c, &n, "t", "gemini", "m", 0, 0, 0, None, false, Some("offline"), 0.0);
        add(&c, &n, "t", "gemini", "m", 0, 0, 0, Some(30_000), false, Some("http_503"), 0.0);
        add(&c, "2020-01-01T00:00:00Z", "t", "gemini", "m", 100, 20, 0, Some(10), true, None, 0.0);
        add(&c, &n, "t", "gemini", "otro", 100, 20, 0, Some(10), true, None, 0.0);
        let w = window(&c, "m", 60).unwrap();
        assert_eq!((w.calls, w.tokens), (2, 240));
        assert_eq!(window(&c, "m", 86_400).unwrap().calls, 2);
        assert_eq!(window(&c, "nunca-usado", 60).unwrap(), Window::default());
    }

    #[test]
    fn the_report_adds_up_per_model_and_per_task() {
        let c = conn();
        let n = now(&c);
        let mut s = AiSettings::default(); // gemini active: flash-lite (light), 3.5 flash (strong) with 3 flash preview behind it
        s.disabled_models.clear();
        add(&c, &n, "diagnosis.next_question", "gemini", "gemini-3.5-flash-lite", 2000, 80, 20, Some(400), true, None, 0.01);
        add(&c, &n, "diagnosis.next_question", "gemini", "gemini-3.5-flash-lite", 2200, 90, 30, Some(600), true, None, 0.02);
        add(&c, &n, "diagnosis.next_question", "gemini", "gemini-3.5-flash-lite", 0, 0, 0, Some(50), false, Some("rate_limited"), 0.0);
        add(&c, &n, "diagnosis.summary", "gemini", "gemini-3-flash-preview", 6000, 3000, 2500, Some(9000), true, None, 0.1);
        add(&c, &n, "diagnosis.summary", "anthropic", "claude-sonnet-5-5", 6000, 900, 0, Some(5000), true, None, 0.5);
        let r = usage_report(&c, &s).unwrap();

        let lite = r.models.iter().find(|m| m.model == "gemini-3.5-flash-lite").unwrap();
        assert_eq!((lite.calls_total, lite.failed_total), (3, 1));
        assert_eq!((lite.input_tokens, lite.output_tokens, lite.thought_tokens), (4200, 170, 50));
        assert_eq!(lite.avg_latency_ms, Some(500), "only answered calls are timed");
        assert_eq!(lite.p95_latency_ms, Some(600));
        assert_eq!(lite.tier, Some(ModelTier::Light));
        assert_eq!(lite.limit.unwrap().per_day, 500);
        assert_eq!(lite.calls_last_day, 3);
        assert!((lite.cost_mxn - 0.03).abs() < 1e-9);

        // a model of the other provider is listed but is not a tier of the active one
        let claude = r.models.iter().find(|m| m.model == "claude-sonnet-5-5").unwrap();
        assert_eq!(claude.tier, None);
        assert!(!lite.is_fallback);
        // the preview model is a backup of the strong tier; the main strong model is 3.5 Flash
        let newer = r.models.iter().find(|m| m.model == "gemini-3-flash-preview").unwrap();
        assert_eq!((newer.tier, newer.is_fallback), (Some(ModelTier::Strong), true));

        let q = r.tasks.iter().find(|t| t.task == "diagnosis.next_question").unwrap();
        assert_eq!(q.calls, 2, "tasks count only valid answers");
        assert_eq!(q.avg_input_tokens, 2100);
        assert_eq!(q.avg_latency_ms, Some(500));
        assert_eq!(r.recent.len(), 5);
        assert!((r.month_spend_mxn - 0.63).abs() < 1e-9);
    }

    #[test]
    fn a_model_switched_off_is_not_the_one_in_front_so_the_next_is_not_a_backup() {
        let c = conn();
        let n = now(&c);
        add(&c, &n, "diagnosis.summary", "gemini", "gemini-3-flash-preview", 6000, 3000, 2500, Some(9000), true, None, 0.1);
        let r = usage_report(&c, &AiSettings::default()).unwrap();
        let first = r.models.iter().find(|m| m.model == "gemini-3-flash-preview").unwrap();
        assert_eq!((first.tier, first.is_fallback), (Some(ModelTier::Strong), false));
    }

    #[test]
    fn a_backup_model_that_answered_is_listed_as_a_backup_of_its_tier() {
        let c = conn();
        let n = now(&c);
        add(&c, &n, "diagnosis.summary", "gemini", "gemini-3-flash-preview", 6000, 3000, 2500, Some(9000), true, None, 0.1);
        let mut on = AiSettings::default();
        on.disabled_models.clear();
        let r = usage_report(&c, &on).unwrap();
        let backup = r.models.iter().find(|m| m.model == "gemini-3-flash-preview").unwrap();
        assert_eq!((backup.tier, backup.is_fallback), (Some(ModelTier::Strong), true));
        assert_eq!(backup.limit.unwrap().per_day, 20, "it has its own allowance");
    }

    #[test]
    fn the_models_about_to_be_used_show_up_before_the_first_call() {
        let c = conn();
        let r = usage_report(&c, &AiSettings::default()).unwrap();
        let names: Vec<_> = r.models.iter().map(|m| m.model.as_str()).collect();
        assert_eq!(names, vec!["gemini-3.5-flash-lite", "gemini-3-flash-preview"]);
        let strong = &r.models[1];
        assert_eq!((strong.calls_total, strong.calls_last_day), (0, 0));
        assert_eq!(strong.limit.unwrap().per_day, 20);
        assert_eq!(strong.avg_latency_ms, None);
    }

    #[test]
    fn percentile_picks_the_nearest_rank() {
        assert_eq!(percentile(&[], 0.95), None);
        assert_eq!(percentile(&[7], 0.95), Some(7));
        let v: Vec<u64> = (1..=20).collect();
        assert_eq!(percentile(&v, 0.95), Some(19));
    }

    #[test]
    fn the_text_report_names_every_model_and_task() {
        let c = conn();
        let n = now(&c);
        add(&c, &n, "diagnosis.summary", "gemini", "gemini-3.8-flash", 6000, 3000, 2500, Some(9000), true, None, 0.1);
        let text = format_report(&usage_report(&c, &AiSettings::default()).unwrap());
        assert!(text.contains("gemini-3.8-flash") && text.contains("diagnosis.summary") && text.contains("9000 ms"), "{text}");
        assert!(text.contains("0/5 por minuto") || text.contains("1/5 por minuto"), "{text}");
    }
}
