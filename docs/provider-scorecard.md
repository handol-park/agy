# Provider Scorecard Template

Use this template to compare LLM providers for the `agy` async agent loop.

## CLI Export

Generate a table from benchmark JSON:

```bash
cargo run --bin scorecard -- benchmarks/provider-results.json
```

Write table output to a file:

```bash
cargo run --bin scorecard -- benchmarks/provider-results.json --output /tmp/scorecard-table.md
```

Update this document in place (between markers):

```bash
cargo run --bin scorecard -- benchmarks/provider-results.json --update-doc docs/provider-scorecard.md
```

Accepted JSON shapes:

```json
[{ "...": "..." }]
```

or

```json
{
  "entries": [{ "...": "..." }]
}
```

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
- same model tier (one cheap/small, one stronger)
- same prompt set, temperature, max tokens, and timeout
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

```text
score = 0.40*quality + 0.20*cost + 0.20*latency + 0.15*reliability + 0.05*availability
```

## Scorecard Table

<!-- SCORECARD_TABLE_START -->

| Provider  | Model                   | Qual % |    P50 |    P95 | Err % | TO % | Cost/1k $ | Uptime % | Score |
| --------- | ----------------------- | -----: | -----: | -----: | ----: | ---: | --------: | -------: | ----: |
| Fireworks | gpt-oss-120b            |  88.20 | 410.00 | 980.00 |  1.10 | 0.40 |     14.50 |    99.93 | 85.90 |
| Groq      | llama-3.3-70b-versatile |  84.70 | 210.00 | 620.00 |  0.80 | 0.20 |     19.80 |    99.93 | 83.40 |

<!-- SCORECARD_TABLE_END -->

### Column Legend

- `Qual %`: quality pass rate percentage
- `P50`: median latency in milliseconds
- `P95`: p95 latency in milliseconds
- `Err %`: request error rate percentage
- `TO %`: timeout rate percentage
- `Cost/1k $`: estimated USD cost per 1,000 benchmark tasks
- `Uptime %`: provider availability percentage for the measurement window
- `Score`: weighted composite score from the formula above

## Notes Log

Track qualitative findings:

- rate-limit behavior
- SDK ergonomics
- model selection depth
- incident communication quality
- billing clarity and surprises
