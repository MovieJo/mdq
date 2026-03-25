# MVP Release Checklist

Use this checklist before tagging or publishing the MVP release.

---

## 1. Build and test

- Run `cargo fmt --check`.
- Run `cargo test`.
- Confirm golden and snapshot fixtures still match the intended CLI contract.

## 2. Command help and docs

- Check `cargo run -- --help`.
- Check `cargo run -- tree --help`.
- Check `cargo run -- get --help`.
- Check `cargo run -- find --help`.
- Verify `README.md` examples still match current command behavior.
- Verify `docs/cli-reference.md` and `docs/json-output-v1.md` match the implementation.

## 3. Installation and execution flow

- Run `cargo install --path . --root <temp-dir>` from a clean build environment.
- Confirm the installed binary can run `tree`, `get`, and `find` against fixture files.
- Confirm local development instructions using `cargo run -- ...` still work from a fresh clone.

## 4. MVP scope checks

- `tree` emits stable ids, correct line ranges, and optional summaries.
- `get` returns exact slices, supports `--max-lines`, and preserves original line numbers with `--with-line-numbers`.
- `find` supports substring and regex search, case-sensitive mode, and `--max-matches`.
- JSON outputs for `tree`, `get`, and `find` match `docs/json-output-v1.md`.
- Error exit codes still follow the documented `0/1/2/3/4` mapping.

## 5. Release hygiene

- Update or close the remaining project issues tied to the MVP release.
- Ensure the release notes mention installation, command surface, and JSON output availability.
- Tag and publish only after the checklist items above are complete.
