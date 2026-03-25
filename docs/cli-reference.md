# CLI Reference

This document defines the command-line contract for the MVP commands.

---

## 0. Top-level command

```text
mdq <command>
```

Top-level help summary:

- `tree`: print an annotated outline for a Markdown file
- `get`: extract the exact Markdown slice for a section
- `find`: search matching lines and map them to section ids

During local development, the same commands can be run as `cargo run -- <command> ...`.

---

## 1. `mdq tree`

### Usage

```text
mdq tree <file> [--format annotated-md|json] [--max-depth N] [--no-summary]
```

### Text Output

Default text output uses the annotated Markdown format defined in `docs/format-annotated-md-v1.md`.

Notes:
- `--max-depth` filters printed sections only; it does not change computed ids or line ranges.
- `--no-summary` suppresses summary lines in text and JSON output alike.

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

---

## 2. `mdq get`

### Usage

```text
mdq get <file> --id <section_id> [--format text|json] [--max-lines N] [--with-line-numbers]
```

### Text Output

- Default output is the extracted raw section slice.
- If `--with-line-numbers` is set, each emitted line is prefixed with `L<line>: ` using original file line numbers.
- If `--max-lines N` is set, only the first `N` extracted lines are emitted.
- `--with-line-numbers` is valid only with `--format text`.
- `--max-lines` must be greater than `0`.
- Truncation happens after exact section extraction and does not add an ellipsis marker.

Example:

```text
L19: ## Data Model
L20: | id | title |
```

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

Notes:
- `start_line` and `end_line` always describe the full section range.
- `truncated` reports whether additional extracted lines were omitted by `--max-lines`.

---

## 3. `mdq find`

### Usage

```text
mdq find <file> <query> [--format text|json] [--regex] [--case-sensitive] [--max-matches N]
```

### Text Output

Each matching source line is emitted as:

```text
L<line> [<section-id-or-dash>] <line-text>
```

Rules:
- `<section-id-or-dash>` is the containing section id.
- For content before the first heading, emit `-`.
- At most one output line is emitted per matching source line.
- Output order is ascending source line order.
- If there are zero matches, text output is empty and the exit code is still `0`.
- Matching is case-insensitive by default.
- `--max-matches` defaults to `200` and must be greater than `0`.

Example:

```text
L13 [s1-1] ### Install
L14 [s1-1] echo "hello"
```

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

Notes:
- Matches before the first heading use `root` as the section id in JSON.
- Invalid regular expressions are reported as CLI usage errors.

---

## 4. Exit Codes

- `0`: success
- `1`: internal error
- `2`: CLI usage error
- `3`: file read or decode error
- `4`: section id not found (`get` only)
