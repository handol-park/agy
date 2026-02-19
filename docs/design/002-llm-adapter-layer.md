# LLM Adapter Layer

## Goal
Enable early real-model demos without coupling agent core logic to a specific provider SDK or HTTP shape.

## Agent Definition at Stage 002
At this stage, an agent is:
- the Stage `001` bounded loop
- plus an interchangeable language-model interface used for synthesis

The key change is not "having AI," but separating agent control flow from model provider implementation.

## Current Design
The adapter boundary lives in `src/lib.rs`:
- async `LanguageModel` trait with `synthesize(goal, constraint) -> Result<String, String>`
- agent loop depends on this trait, not on HTTP clients

Provider-specific implementation currently lives in `src/main.rs`:
- `OpenAiCompatModel` uses async `reqwest::Client`
- calls `POST {base_url}/chat/completions`
- parses first choice content and returns text

Fallback implementation:
- `TemplateModel` provides deterministic local synthesis when no API key is set
- used by default path in CLI for offline/demo safety

## Why This Split
- Keeps `src/lib.rs` testable and deterministic.
- Allows fast provider swaps by changing only adapter code.
- Preserves one stable interface (`LanguageModel`) for future planners.
- Aligns with async-first runtime so model/tool I/O can scale without blocking the loop.

## Runtime Configuration
CLI selects model mode by environment variables:
- `LLM_PROVIDER`: optional, `openai` (default) or `glm5`
- `LLM_API_KEY`: preferred API key variable
- `LLM_BASE_URL`: optional override
- `LLM_MODEL`: optional override

Back-compat aliases are also supported:
- `OPENAI_API_KEY`, `OPENAI_BASE_URL`, `OPENAI_MODEL`
- `GLM_API_KEY`, `GLM_BASE_URL`, `GLM_MODEL`

Provider defaults:
- `openai` -> base URL `https://api.openai.com/v1`, model `gpt-4o-mini`
- `glm5` -> base URL `https://api.z.ai/api/paas/v4`, model `glm-5`

If no API key is found, CLI uses `TemplateModel`.

## Error Handling
Adapter wraps and returns string errors for:
- request failures
- non-2xx HTTP responses
- response decode failures
- empty/no-choice responses

Agent core handles adapter errors with graceful fallback finalization message, so loop still completes.

## Test Strategy
Core tests use fake model implementations:
- success case (`FakeModelOk`)
- error case (`FakeModelErr`)

No network is required for default test runs.
Core async tests run with `#[tokio::test]`.

## Next Improvements
- move provider adapter into `src/model/` module
- replace string errors with typed error enums
- support streaming and token/cost telemetry
- add retry and timeout policy at adapter boundary
