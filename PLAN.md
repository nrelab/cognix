# Rebrand Plan: Cognix -> Cognix

## Goal
Mechanically replace every occurrence of the product/brand identifier "cognix" with "cognix" across the repository, using a Python script at `replacer.py`, then verify the build still compiles.

## What gets replaced
Case-aware token replacement (applied in this order to avoid clobbering):
1. `COGNIX` -> `COGNIX`
2. `Cognix` -> `Cognix`
3. `cognix` -> `cognix`

This covers:
- Rust source: imports (`use cognix_core::...`), crate names (`cognix_core` -> `cognix_core`), module names (`cognix_managed_paths_watcher`), string literals (`"cognix_drive"`, `"cognixUserHandoff"`), comments.
- `Cargo.toml` / `Cargo.lock`: package names (`name = "cognix"`), dependency aliases, workspace members paths.
- Docs/markdown: `Cognix`, `cognix.dev`, `nrelab`.
- `about.toml`: `cognix.dev`, `nrelab`.
- Filenames and directory names (e.g. `crates/cognix_core` -> `crates/cognix_core`, `.cognix` -> `.cognix`).

## What is deliberately NOT touched
- `.git/` directory (history preserved).
- `target/` build artifacts.
- The `replacer.py` script itself.
- Binary / non-text files (images, compiled assets).
- `vendor/` or cargo registry caches (not present in workspace).
- External git remotes (workflow/proto crates hosted on `github.com/nrelab`) — these are referenced by URL/rev only; mechanical replacement of URLs is a separate decision.

## Risks / manual follow-up
- **Crate renames require matching `Cargo.toml` `[package] name` and dependency references.** The text replace handles both sides, but run `cargo check` to confirm.
- **String literals that are external URLs/API endpoints** containing "cognix" (e.g. `cognix.dev`) may need to remain or be redirected — review after replacement.
- **Rust `macro_rules!` / `include_str!` paths** that embed files by path: file renames handled by path rename step.
- **Serialization / persisted user prefs** keyed on `"cognix_*"` strings: changing them may break backward compatibility with existing user data. Review `"cognix_drive"`-style keys.
- **`Cargo.lock`** must be regenerated (`cargo update`) after crate renames.

## Execution order
1. Run `python replacer.py --dry-run` to preview changes.
2. Review the preview, especially the "would rename" and "would edit" lists.
3. Run `python replacer.py --apply` to perform replacements and renames.
4. Run `cargo check` / `cargo build` to verify compilation.
5. Review string literals / URLs manually; fix any that should not have changed.
6. Regenerate `Cargo.lock` (`cargo update`) and run tests.

## Safety
- The script prints every edit and rename before applying.
- No changes are made unless `--apply` is passed.
- Always commit only after verifying `git diff` looks sane.
