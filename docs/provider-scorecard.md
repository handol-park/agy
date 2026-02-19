# Provider Scorecard Template

Use this template to compare LLM providers for the `agy` async agent loop.

## CLI Export
Generate the markdown table from benchmark JSON:

`cargo run --bin scorecard -- benchmarks/provider-results.json`

Write table to a file:

`cargo run --bin scorecard -- benchmarks/provider-results.json --output /tmp/scorecard-table.md`

Update this doc in place (between markers):

`cargo run --bin scorecard -- benchmarks/provider-results.json --update-doc docs/provider-scorecard.md`

Accepted JSON shapes:
- `[{...}, {...}]`
- `{\"entries\": [{...}, {...}]}`

Entry fields:
- `provider`, `model`
- `quality_pct`, `p50_ms`, `p95_ms`
- `error_pct`, `timeout_pct`
- `cost_per_1k_tasks_usd`, `availability_pct`
- `score`

## Providers in Scope
- Fireworks
- Together
- Groq
- OpenRouter

## Test Protocol
Run the same workload for each provider:
- 100 requests per scenario
- same model tier (small/cheap instruct model, and one stronger model)
- same prompt set, temperature, max tokens, timeout
- run at least 3 times, then average

Scenarios:
- simple planning prompt
- tool-use planning prompt
- long-context summarization prompt

## Metrics
- `quality_pass_rate`: percent of outputs that satisfy expected checks
- `latency_p50_ms`, `latency_p95_ms`
- `error_rate_pct`: non-2xx + timeout + malformed response
- `timeout_rate_pct`
- `cost_per_1k_tasks_usd`: estimated from measured token usage and provider pricing
- `availability_pct`: from provider status history over the same window

## Weighted Score (0-100)
Use normalized subscores:
- quality: 40%
- cost: 20%
- p95 latency: 20%
- reliability (error + timeout): 15%
- availability: 5%

Formula:
`score = 0.40*quality + 0.20*cost + 0.20*latency + 0.15*reliability + 0.05*availability`

## Scorecard Table
<!-- SCORECARD_TABLE_START -->
| Provider | Model | Quality % | P50 ms | P95 ms | Error % | Timeout % | Cost / 1k tasks ($) | Availability % | Score |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Fireworks | | | | | | | | | |
| Together | | | | | | | | | |
| Groq | | | | | | | | | |
| OpenRouter | | | | | | | | | |
<!-- SCORECARD_TABLE_END -->

## Notes Log
Track qualitative findings:
- rate-limit behavior
- SDK ergonomics
- model selection depth
- incident communication quality
- billing clarity and surprises
