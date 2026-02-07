# Issue 539 Minimal Fix Notes

## Goal

Fix `coder`/`code` failing with:

- `model_not_found` for `gpt-5.3-codex`

while `codex` succeeds on the same machine/account.

## What We Observed

- `code` (fork) was sending OpenAI requests to the correct ChatGPT backend (`https://chatgpt.com/backend-api/codex/responses`) with ChatGPT auth.
- The request still failed with `model_not_found` for `gpt-5.3-codex`.
- `codex` (upstream binary) in the same environment succeeded.
- The key request difference was the `version` HTTP header:
  - `code`: older local value (`0.0.0`/`0.6.59` depending build path)
  - `codex`: newer upstream-compatible value (`0.98.0`)

This made the issue look like backend capability/version gating, not auth routing.

## Minimal Patch Strategy

To keep this PR narrow and durable:

1. Add a tiny compatibility helper (`code-rs/core/src/version_compat.rs`) that computes a backend-compatible version value.
2. Source the compatibility floor from existing repo metadata (`announcement_tip.toml`) instead of duplicating a version in multiple places.
3. Apply the helper only where required for this issue:
   - OpenAI provider request header `version` in `code-rs/core/src/model_provider_info.rs`.

No broader auth/session/model-selection behavior was changed.

## Why This Is Minimal

- Touches only the request-version compatibility path used by failing calls.
- Avoids unrelated refactors or additional runtime behaviors.
- Keeps version-floor maintenance centralized in one already-maintained repo location.

## Validation

- Required check passed: `./build-fast.sh`
- Runtime verification with newly built binary:
  - `gpt-5.3-codex` now returns `pong`
  - `gpt-5.2-codex` still returns `pong`
- Trace confirms outbound OpenAI provider header now advertises a backend-compatible version (`0.98.0`) on the failing path.
