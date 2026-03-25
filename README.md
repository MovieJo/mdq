# mdq

`mdq` is a small CLI for agent-friendly Markdown navigation.

It helps you avoid dumping entire Markdown files into context by providing:
- a compact annotated outline (`mdq tree`)
- section extraction by stable section id (`mdq get`)
- keyword search mapped to section ids (`mdq find`)

The default `tree` output is valid Markdown with minimal ASCII-only format markers.
Titles and summary payload text are printed as original UTF-8.

---

## Why

Markdown docs often contain token-heavy noise:
- huge tables
- long code blocks
- embedded images, including `data:image/...;base64,...`

Agents usually do not need everything at once. They need:
1. a map of the structure
2. a way to jump to the relevant section
3. a way to fetch only what is necessary

`mdq` is designed for that workflow.

---

## Quickstart

Examples below assume `mdq` is already on your `PATH`.
When working from a fresh clone, replace `mdq` with `cargo run --`.

### 1. Print an annotated outline

```sh
mdq tree README.md
```

Example output:

```md
# [s1 L1-L30] Main Header
P: High-level overview.

## [s1-1 L4-L18] Install
C: sh, 3 lines

## [s1-2 L19-L24] Data Model
T: id | title | range (3 cols x 5 rows)
```

- `s1`, `s1-1`, ... are stable section ids and do not contain line numbers.
- `Lx-Ly` is the inclusive section range in the original file.
- Summary lines are deterministic, first-block-only, and plaintext.
- Empty sections do not emit a summary line.

### 2. Extract a section by id

```sh
mdq get README.md --id s1-2
```

This returns the exact raw slice of the original Markdown for that section range.

To inspect only the first few lines with original line numbers:

```sh
mdq get README.md --id s1-2 --max-lines 3 --with-line-numbers
```

### 3. Find text and jump to a section

```sh
mdq find README.md install
```

Example text output:

```text
L13 [s1-1] ### Install
L14 [s1-1] echo "hello"
```

---

## Commands

- `mdq tree <file>`: annotated outline and first-block summaries
- `mdq get <file> --id <section_id>`: exact section extraction, optional truncation, optional line-numbered text output
- `mdq find <file> <query>`: line-based search mapped to section ids, with regex and case-sensitive options

See `docs/cli-reference.md` for the full command contract.

---

## Output Formats

### Annotated Markdown

`mdq tree` produces annotated Markdown:

- headings are preserved (`#`..`######`)
- metadata markers are ASCII: `[<id> L<start>-L<end>]`
- summary markers are ASCII tags: `P:`, `T:`, etc.
- titles and summary payload text remain original UTF-8
- summary is first-block-only; empty sections emit no summary line

Format spec:
- `docs/format-annotated-md-v1.md`

### JSON

Each command can produce JSON for tool integration:

- `mdq tree <file> --format json`
- `mdq get <file> --id <section_id> --format json`
- `mdq find <file> <query> --format json`

JSON spec:
- `docs/json-output-v1.md`

---

## Section IDs

### IDs do not contain line numbers

Line numbers change often during editing; ids remain stable under edits inside a section body.

### IDs are hierarchical by sibling index

- root children: `s1`, `s2`, ...
- children: `s1-1`, `s1-2`, ...
- deeper: `s1-2-1`, ...

IDs may change if the heading structure changes. This is expected.

---

## Summary Behavior

For each section, `mdq` summarizes at most one content block:

- `P`: paragraph
- `Q`: blockquote
- `L`: list (up to 3 items)
- `C`: fenced code (language + line count only)
- `T`: table (header columns + shape only)
- `I`: image (alt + source kind only, never base64)

Important:
- Summary scanning for a section considers only content before its first nested heading.
- If a section contains only subsections and no body content, it emits no summary line.

---

## Installation

### From source (Rust)

```sh
cargo install --path .
```

Or run without installing:

```sh
cargo run -- tree README.md
```

Once published:

```sh
cargo install mdq
```

---

## Development

- Spec: `docs/spec-mvp.md`
- Format: `docs/format-annotated-md-v1.md`
- JSON: `docs/json-output-v1.md`
- CLI: `docs/cli-reference.md`
- Release: `docs/release-checklist.md`
- Testing: `docs/testing.md`

---

## License

Apache-2.0
