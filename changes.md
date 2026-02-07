# Issue 539 Fix Notes

## Motivation

Issue 539 exposed a consistency problem in source builds: several code paths surfaced a `0.0.0` client/version value when `CODE_VERSION` was not injected by release tooling. That weakened version attribution for remote model requests and made local behavior diverge from packaged builds.

While tracing that path, we also found adjacent consistency gaps in auth mode selection and API-key model gating that could produce surprising behavior for Codex model access.

## Discovery

I reproduced the version mismatch by following the source-build path where `CODE_VERSION` is absent. In that flow, `code-version` fell back to `0.0.0`.

From there, I traced all version consumers and found two user-visible/system-visible call sites:

- remote model manager request metadata (`client_version`)
- exec human output banner

I then reviewed auth/model-selection logic around the same flow and identified two additional mismatches:

- exec auth manager did not consistently prefer ChatGPT auth mode when ChatGPT auth was configured
- TUI API-key restrictions gated `gpt-5.3-codex` but not `gpt-5.2-codex`

## Approach

1. Fix version fallback at the source.
- Updated `code-rs/code-version/build.rs` to keep honoring `CODE_VERSION` when present, but otherwise read the version from `codex-cli/package.json`.
- This preserves release behavior and gives source builds a real version.

2. Normalize remote client version generation.
- Updated `code-rs/core/src/remote_models/mod.rs` to derive version from `code_version::version()` and parse the semver triplet for API payload compatibility.
- Added/updated tests in `code-rs/core/src/remote_models/mod.rs` and `code-rs/core/tests/remote_models_manager.rs`, including an assertion that `client_version` is not `0.0.0`.

3. Align adjacent runtime behavior.
- Updated exec human summary version text to use `code_version::version()` in `code-rs/exec/src/event_processor_with_human_output.rs`.
- Updated exec auth manager mode preference in `code-rs/exec/src/lib.rs` so ChatGPT-configured auth uses ChatGPT mode.
- Updated TUI API-key gating in `code-rs/tui/src/lib.rs` to block both `gpt-5.2-codex` and `gpt-5.3-codex` consistently.
- Added `code-version` as a dependency for `code-exec` (`code-rs/exec/Cargo.toml`, lockfile updated).

4. Validate with required repo check.
- Ran `./build-fast.sh` (repo-required completion check) and confirmed it passes cleanly with no remaining warnings/errors.

## Why this structure

This fix keeps one practical source of truth for versioning in source builds, avoids hardcoded or duplicated version strings, and adds targeted regression tests where the breakage surfaced. The goal was to make local, CI, and packaged behavior converge while keeping changes narrow and auditable.
