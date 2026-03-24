# Release Checklist

Use this checklist before cutting an MVP release.

---

## 1. Build and Test

- Run `cargo fmt --check`.
- Run `cargo test`.
- Run `cargo run -- --help`.
- Run `cargo run -- tree --help`.
- Run `cargo run -- get --help`.
- Run `cargo run -- find --help`.

## 2. Smoke Test the User Flow

- Run `cargo run -- tree README.md`.
- Run `cargo run -- get README.md --id s1`.
- Run `cargo run -- find README.md mdq`.
- If installing from source for validation, run `cargo install --path .` and then `mdq --help`.

## 3. Docs and Contract Checks

- Confirm `README.md` quickstart commands still work.
- Confirm `docs/cli-reference.md` matches current CLI help output.
- Confirm `docs/json-output-v1.md` still matches emitted JSON fields.
- Confirm `docs/format-annotated-md-v1.md` still matches `tree --format annotated-md`.
- Confirm open MVP issues are either resolved or explicitly deferred for the release.

## 4. Release Readiness

- Verify the working tree is clean before tagging or publishing.
- Ensure the release branch or PR references the shipped issue set.
- Summarize any known MVP limitations in the release notes.
