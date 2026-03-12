# mdq

`mdq` is a small CLI for agent-friendly Markdown navigation.

It helps you avoid dumping entire Markdown files into context by providing:
- a compact annotated outline (`mdq tree`)
- section extraction by stable section id (`mdq get`)
- keyword search mapped to section ids (`mdq find`)

The default `tree` output is **valid Markdown** with minimal **ASCII-only format markers**.
Titles and summary payload text are printed as original UTF-8.

---

## Why

Markdown docs often contain token-heavy noise:
- huge tables
- long code blocks
- embedded images (including `data:image/...;base64,...`)

Agents usually don’t need *everything* at once. They need:
1) a map of the structure
2) a way to jump to the relevant section(s)
3) a way to fetch only what’s necessary

`mdq` is designed for that workflow.

---

## Quickstart

### 1) Print an annotated outline

```sh
mdq tree README.md
````

Example output (annotated Markdown):

```md
# [s1 L1-L30] Main Header
P: High-level overview.

## [s1-1 L4-L18] Install
C: sh, 3 lines

## [s1-2 L19-L24] Data Model
T: id | title | range (3 cols x 5 rows)
```

* `s1`, `s1-1`, ... are stable section ids (not based on line numbers).
* `Lx-Ly` is the section range in the original file (inclusive).
* The summary line is deterministic, first-block-only, and plaintext.
* Empty sections do not emit a summary line.

### 2) Extract a section by id

```sh
mdq get README.md --id s1-2
```

This returns the exact raw slice of the original Markdown for that section range.

### 3) Find text and jump to a section

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

* `mdq tree <file>`: annotated outline + first-block summaries
* `mdq get <file> --id <section_id>`: section extraction
* `mdq find <file> <query>`: search mapped to section ids

See: `docs/cli-reference.md`

---

## Output formats

### Annotated Markdown (default for `tree`)

`mdq tree` produces "annotated Markdown":

* headings are preserved (`#`..`######`)
* metadata markers are ASCII: `[<id> L<start>-L<end>]`
* summary markers are ASCII tags: `P:`, `T:`, etc.
* titles and summary payload text remain original UTF-8
* summary is first-block-only; empty sections emit no summary line

Format spec:

* `docs/format-annotated-md-v1.md`

### JSON

Each command can produce JSON for tool integration.

JSON spec:

* `docs/json-output-v1.md`

---

## Section IDs

### IDs do NOT contain line numbers

Line numbers change often during editing; ids must remain stable under edits inside a section body.

### IDs are hierarchical by sibling index

* root children: `s1`, `s2`, ...
* children: `s1-1`, `s1-2`, ...
* deeper: `s1-2-1`, ...

IDs may change if the heading structure changes (inserting/removing/reordering headings). This is expected.

---

## Summary behavior (first-block-only)

For each section, mdq summarizes at most one content block:

* P: paragraph
* Q: blockquote
* L: list (up to 3 items)
* C: fenced code (lang + line count only)
* T: table (header columns + shape only)
* I: image (alt + src kind only; never prints base64)

Important:

* Summary scanning for a section considers only content **before its first nested heading**.

  * If a section contains only subsections and no body content, it emits no summary line.

---

## Installation

### From source (Rust)

```sh
cargo install --path .
```

(Once published:)

```sh
cargo install mdq
```

---

## Development

* Spec: `docs/spec-mvp.md`
* Format: `docs/format-annotated-md-v1.md`
* JSON: `docs/json-output-v1.md`
* Architecture: `docs/architecture.md`
* Testing: `docs/testing.md`

---

## License

Apache-2.0
