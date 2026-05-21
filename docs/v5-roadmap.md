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
llm_format run myagent.llm --key $ANTHROPIC_API_KEY
Validates, transpiles to native format, sends HTTP
request, validates response against output: schema,
exits with code 0/1/2.

## Track L — lint Command (Prompt Quality)
llm_format lint myprompt.llm
Static analysis: redundant instructions, missing
output schema, contradictory constraints, prompt
length warnings, provider-specific recommendations.

## Track Sequencing
J → K → L

Track J first: it is the honest foundation. Without
native JSON output, the run command has nothing real
to send.

## v5 Success Criteria
- [ ] anthropic-messages and openai-chat targets exist
      and produce valid API payloads
- [ ] run command executes a .llm file end-to-end
- [ ] response validated against output: schema
- [ ] lint command catches at least 5 real prompt issues
- [ ] README describes what the tool actually does today
- [ ] zero stale claims in any public-facing document
