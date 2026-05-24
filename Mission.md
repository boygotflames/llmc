# Mission: promptus-dsl

## What This Is

promptus-dsl is a prompt compiler for LLM workflows.

You write structured prompts in .llm files. The compiler
validates them, formats them, and transpiles them into
the exact JSON format each provider API expects. You can
also execute them directly from the CLI.

## The Problem It Solves

Prompts are written as raw strings scattered across
codebases: hardcoded in Python, copy-pasted in notebooks,
inconsistently formatted between providers. There is no
validation before you spend tokens. There is no reuse.
There is no separation between what you write and what
the API receives.

promptus-dsl solves this with three things:

1. A structured authoring format (.llm) that is version-
   control friendly, composable via includes, and
   validates before execution
2. A compiler that emits provider-native JSON — the exact
   payload Anthropic or OpenAI expects, ready to send
3. A runtime that executes the prompt, validates the
   response against your declared output schema, and
   exits with a useful code

## What It Is Not

- Not a new "open standard" — it is an open tool
- Not a token-saving format — token count is not the
  bottleneck; prompt quality is
- Not a replacement for provider SDKs — it compiles
  to what those SDKs already accept

## Engineering Tenets

1. Solve real problems — every feature must address a
   pain that exists today, not a hypothetical future
2. Honest claims only — no benchmarks that cherry-pick
   baselines, no performance claims without evidence
3. Deterministic output — same input always produces
   same output; this is the core contract
4. Provider-native — output targets are real API formats,
   not invented encodings
5. Fail loudly — validation errors before execution save
   money and debugging time
