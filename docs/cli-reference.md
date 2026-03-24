# CLI Reference

This document defines the command-line contract for the MVP commands.

---

## 1. `mdq tree`

### Help Usage

```text
mdq tree [OPTIONS] <FILE>
```

### Text Output

Default text output uses the annotated Markdown format defined in `docs/format-annotated-md-v1.md`.

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

### Options

- `--format <FORMAT>`: output format (`annotated-md` or `json`)
- `--max-depth <N>`: maximum heading depth to print
- `--no-summary`: suppress summary lines

---

## 2. `mdq get`

### Help Usage

```text
mdq get [OPTIONS] --id <SECTION_ID> <FILE>
```

### Text Output

- Default output is the extracted raw section slice.
- If `--with-line-numbers` is set, each emitted line is prefixed with `L<line>: ` using original file line numbers.
- If `--max-lines N` is set, only the first `N` extracted lines are emitted.

Example:

```text
L19: ## Data Model
L20: | id | title |
```

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

### Options

- `--id <SECTION_ID>`: section id to extract
- `--format <FORMAT>`: output format (`text` or `json`)
- `--max-lines <N>`: limit emitted lines after extraction
- `--with-line-numbers`: prefix text output with original file line numbers
- `--with-line-numbers` is valid only with `--format text`

---

## 3. `mdq find`

### Help Usage

```text
mdq find [OPTIONS] <FILE> <QUERY>
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

Example:

```text
L13 [s1-1] ### Install
L14 [s1-1] echo "hello"
```

### JSON Output

JSON output uses the schema defined in `docs/json-output-v1.md`.

### Options

- `--format <FORMAT>`: output format (`text` or `json`)
- `--regex`: interpret the query as a regular expression
- `--case-sensitive`: match with ASCII case sensitivity
- `--max-matches <N>`: stop after emitting `N` matching lines (default: `200`)

---

## 4. Exit Codes

- `0`: success
- `1`: internal error
- `2`: CLI usage error
- `3`: file read or decode error
- `4`: section id not found (`get` only)

---

## 5. Installation and Verification

Install from the repository root:

```sh
cargo install --path .
```

Run without installing:

```sh
cargo run -- --help
```

For release validation steps, see `docs/release-checklist.md`.
