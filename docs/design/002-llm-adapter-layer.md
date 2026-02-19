# LLM Adapter Layer

## Goal
Enable early real-model demos without coupling agent core logic to a specific provider SDK or HTTP shape.

## Current Design
The adapter boundary lives in `src/lib.rs`:
- `LanguageModel` trait with `synthesize(goal, constraint) -> Result<String, String>`
- agent loop depends on this trait, not on HTTP clients

Provider-specific implementation currently lives in `src/main.rs`:
- `OpenAiCompatModel` uses blocking `reqwest`
- calls `POST {base_url}/chat/completions`
- parses first choice content and returns text

Fallback implementation:
- `TemplateModel` provides deterministic local synthesis when no API key is set
- used by default path in CLI for offline/demo safety

## Why This Split
- Keeps `src/lib.rs` testable and deterministic.
- Allows fast provider swaps by changing only adapter code.
- Preserves one stable interface (`LanguageModel`) for future planners.

## Runtime Configuration
CLI selects model mode by environment variables:
- `OPENAI_API_KEY`: if set, use `OpenAiCompatModel`
- `OPENAI_BASE_URL`: optional, default `https://api.openai.com/v1`
- `OPENAI_MODEL`: optional, default `gpt-4o-mini`

If `OPENAI_API_KEY` is missing, CLI uses `TemplateModel`.

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

## Next Improvements
- move provider adapter into `src/model/` module
- replace string errors with typed error enums
- support streaming and token/cost telemetry
- add retry and timeout policy at adapter boundary
