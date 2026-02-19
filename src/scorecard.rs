use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct ScorecardEntry {
    pub provider: String,
    pub model: String,
    pub quality_pct: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub error_pct: f64,
    pub timeout_pct: f64,
    pub cost_per_1k_tasks_usd: f64,
    pub availability_pct: f64,
    pub score: f64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InputShape {
    List(Vec<ScorecardEntry>),
    Wrapped { entries: Vec<ScorecardEntry> },
}

pub fn load_entries_from_file(path: &str) -> Result<Vec<ScorecardEntry>, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read '{path}': {e}"))?;
    let input: InputShape = serde_json::from_str(&raw)
        .map_err(|e| format!("failed to parse JSON from '{path}': {e}"))?;

    let entries = match input {
        InputShape::List(entries) => entries,
        InputShape::Wrapped { entries } => entries,
    };

    if entries.is_empty() {
        return Err("no entries found in scorecard input".to_string());
    }

    Ok(entries)
}

pub fn render_markdown_table(entries: &[ScorecardEntry]) -> String {
    let mut out = String::new();
    out.push_str(
        "| Provider | Model | Qual % | P50 | P95 | Err % | TO % | Cost/1k $ | Uptime % | Score |\n",
    );
    out.push_str("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|\n");

    for e in entries {
        out.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |\n",
            escape_cell(&e.provider),
            escape_cell(&e.model),
            e.quality_pct,
            e.p50_ms,
            e.p95_ms,
            e.error_pct,
            e.timeout_pct,
            e.cost_per_1k_tasks_usd,
            e.availability_pct,
            e.score
        ));
    }

    out
}

pub fn update_doc_table(doc: &str, table: &str) -> Result<String, String> {
    let start_marker = "<!-- SCORECARD_TABLE_START -->";
    let end_marker = "<!-- SCORECARD_TABLE_END -->";

    let start = doc
        .find(start_marker)
        .ok_or_else(|| format!("missing marker: {start_marker}"))?;
    let end = doc
        .find(end_marker)
        .ok_or_else(|| format!("missing marker: {end_marker}"))?;

    if end <= start {
        return Err("invalid marker order in documentation file".to_string());
    }

    let prefix = &doc[..start + start_marker.len()];
    let suffix = &doc[end..];

    let mut out = String::new();
    out.push_str(prefix);
    out.push_str("\n");
    out.push_str(table.trim_end());
    out.push_str("\n");
    out.push_str(suffix);

    Ok(out)
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown_table() {
        let entries = vec![ScorecardEntry {
            provider: "Fireworks".to_string(),
            model: "gpt-oss-120b".to_string(),
            quality_pct: 89.4,
            p50_ms: 320.0,
            p95_ms: 900.0,
            error_pct: 1.2,
            timeout_pct: 0.3,
            cost_per_1k_tasks_usd: 14.2,
            availability_pct: 99.95,
            score: 86.7,
        }];

        let table = render_markdown_table(&entries);
        assert!(table.contains("| Fireworks | gpt-oss-120b | 89.40"));
        assert!(table.contains("|---|---|---:"));
    }

    #[test]
    fn updates_table_between_markers() {
        let doc = "a\n<!-- SCORECARD_TABLE_START -->\nold\n<!-- SCORECARD_TABLE_END -->\nz";
        let updated = update_doc_table(doc, "| h |\n|---|\n| v |").expect("update should work");
        assert!(updated.contains("| h |"));
        assert!(!updated.contains("\nold\n"));
    }
}
