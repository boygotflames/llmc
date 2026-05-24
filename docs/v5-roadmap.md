# v5 Roadmap — Make It Real

## Why v5 Exists

v1 through v4 built a technically solid toolchain pointed
at the wrong problem. The shadow encoding (@a="value") is
an invented machine format with no evidence of model
performance improvement. The 8.5% token savings are too
small to matter on real production prompts. Nobody will
adopt a new DSL to save 2-4 tokens.

v5 pivots to solving a pain people actually have:

> Prompts are written as raw strings scattered across
> codebases, never validated, impossible to test or reuse,
> completely opaque until you pay to run them.

.llm becomes a provider-agnostic authoring layer that
transpiles to the native JSON format each API actually
expects — and can execute those API calls directly.

## What Changes in v5

### Killed
- Shadow encoding removed from primary CLI interface
  and README examples. Kept as --target shadow for
  backwards compatibility but no longer presented as
  a feature.
- "Token savings" as the value proposition. Replaced
  by "write once, run anywhere" and "validate before
  you spend."
- "Open standard" claim. Replaced by "open tool."

### Built
- --target anthropic-messages: emits the exact JSON
  format Anthropic's API expects
- --target openai-chat: emits the exact JSON format
  OpenAI's chat completion API expects
- run command: validates + sends to the provider API
  using the native format, validates the response
  against the output: schema
- lint command: analyzes a prompt for structural
  problems before execution

## Track J — Native Provider JSON Targets
Emit real API-ready JSON instead of custom @-marker
encoding. The .llm file becomes a clean authoring
layer; the output is what you paste into curl.

## Track K — run Command (Live Execution)

### Track K Part 1 [COMPLETE]
- run command: parse → compose → validate → transpile
  to native JSON → inject model → POST → print response
- Provider support: openai (Chat Completions),
  anthropic (Messages API)
- API key: --key flag or OPENAI_API_KEY/ANTHROPIC_API_KEY
  env var fallback
- Empty key detection: set-but-empty env var rejected
  with actionable error message
- Validation wall: E101 fires before any network call

### Track K Part 2 [COMPLETE]
- Response schema validation against output: field;
  warns on missing JSON fields (not an error)
- --dry-run flag: prints API payload without sending;
  no API key required in dry-run mode

### Track K Part 3 [NEXT]
- Timeout configuration (--timeout flag)
- Retry logic for transient network failures

## Track L — Lint Command [COMPLETE]

Static analysis for prompt quality. Six rules.

- L001: Missing output: schema
- L002: Missing user: turn
- L003: System prompt very short (< 20 chars)
- L004: System prompt very long (> 4000 chars)
- L005: Contradictory constraints
- L006: Duplicate constraints

Deferred to Part 2 (post-v5):
- Provider-specific lint rules
- Prompt injection detection
- Semantic similarity for near-duplicate constraints

## Track Sequencing
J → K → L

Track J first: it is the honest foundation. Without
native JSON output, the run command has nothing real
to send.

## v5 Success Criteria
- [x] anthropic-messages and openai-chat targets exist
      and produce valid API payloads
- [x] run command executes a .llm file end-to-end
- [x] response validated against output: schema
- [x] lint command: 6 rules implemented (L001–L006)
- [x] README describes what the tool actually does today
- [x] zero stale claims in any public-facing document
