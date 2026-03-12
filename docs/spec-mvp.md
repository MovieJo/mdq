# mdq MVP Specification

## 1. Overview

`mdq` is a CLI tool that helps humans and LLM agents navigate Markdown efficiently by:
- producing an annotated outline (`mdq tree`) with stable section ids and source line ranges,
- emitting a first-block-only, rule-based plaintext summary per non-empty section,
- extracting a section’s raw Markdown by section id (`mdq get`),
- finding query matches and mapping them to section ids and line numbers (`mdq find`).

`mdq` supports two output styles:
- **annotated-md** (default for `tree`): a Markdown-compatible outline document with minimal ASCII format markers
- **json**: structured output for tool integration

This document defines MVP behavior and contracts for `mdq tree`, `mdq get`, and `mdq find`.

---

## 2. Non-goals (MVP)

Out of scope for MVP:
- Multi-file or directory recursion
- Full CommonMark/GFM rendering compliance (mdq is not a renderer)
- AI summarization (summary is deterministic and rule-based)
- Persistent indexes/caches/daemon services
- TUI/interactive viewer
- Inline-level AST extraction (beyond what’s required for first-block classification)

---

## 3. Inputs and Basic Rules

### 3.1 Input file
- One Markdown file per invocation.
- File must be UTF-8 decodable. If decoding fails, the command errors.

### 3.2 Line numbering
- 1-based line numbers.
- Line breaks are treated as `\n`.
- If CRLF exists, treat `\r\n` as one line break.
- Keep behavior deterministic across platforms.

### 3.3 Determinism
For the same file bytes and the same flags:
- output must be identical across runs (no timestamps/randomness)
- ordering is always source order

---

## 4. Section Model

### 4.1 Headings considered
A “section” is defined by Markdown headings:
- ATX headings (`#`..`######`)
- Setext headings (`===`, `---`) MAY be supported if the chosen parser supports them; otherwise treat them as plain text.

Recommended: use a parser that can identify headings and provide source offsets (e.g., `pulldown-cmark`).

### 4.2 Virtual root
A virtual root node exists conceptually:
- id: `root`
- depth: 0
Root is not printed in annotated-md output, but is present in JSON for parent relations.

### 4.3 Parent selection for irregular heading structure
Markdown can be irregular:
- start with h2 without h1
- skip heading levels
- introduce h1 late

mdq must build a consistent tree without editing the input.

**Algorithm (stack-based):**
When a heading H with level L (1..6) is encountered in document order:
1) Find nearest prior open node on stack with level < L.
2) If found, that node is the parent; else parent is `root`.
3) Pop stack until top has level < L, then push H.

This ensures:
- top-level h2 becomes a root child
- h1 -> h3 becomes child of h1
- late h1 becomes a root child

### 4.4 Section line range
Each heading defines a section range that includes the heading line.

For a section S defined by heading at line START with level L:
- START = heading line
- END = the line immediately before the next heading whose level <= L
- If no such heading exists, END = EOF line

Always: START <= END.

### 4.5 Summary scanning window (important)
To avoid parent sections summarizing their child headings:
- Summary scanning MUST consider only the section body **before the first nested heading** (a heading with level > current level) inside that section.
- If there is no non-empty content before the first nested heading, the section is treated as empty for summary purposes (no summary line).

This matches common documentation style where parents are containers for subsections.

---

## 5. Stable Section IDs

### 5.1 Requirements
- IDs MUST NOT include line numbers.
- IDs should remain stable under edits inside a section body (adding/removing lines within the section).
- IDs may change when heading structure changes (insert/remove/reorder headings, change heading levels).

### 5.2 Format
IDs are hierarchical and based on sibling indices in the section tree:
- root’s Nth child: `sN`
- any node X’s Mth child: `X-M`

Examples:
- `s1`, `s2`
- `s1-1`, `s1-2`
- `s2-3-1`

### 5.3 Assignment rule
- Sibling ordering is source order.
- Indices start at 1.
- IDs are assigned deterministically after the tree is built.

---

## 6. Rule-based First-block Summary (plaintext)

### 6.1 Goal
Each non-empty section emits at most one summary line describing the first meaningful content block (deterministic, not AI).

### 6.2 “First block” scanning
For a section range START..END with heading level L:
- scan begins at line START+1
- scan ends at the earlier of:
  - END, or
  - the line before the first nested heading (level > L) that occurs within START..END
- skip blank lines to find first non-empty line
- classify block using priority (below)

If no block found, section is empty for summary; do not print summary line.

### 6.3 Block classification priority
First non-empty block is classified by the first matching rule:

1) Fenced code block
   - line begins with ``` or ~~~ (3+ fence chars), allowing up to 3 leading spaces
2) Blockquote
   - line begins with `>`
3) GFM pipe table
   - a header row containing `|`
   - immediately followed by a separator row like `---|---` or `|:---:|`
4) List
   - unordered: `- `, `* `, `+ `
   - ordered: `1. `, `2. `, etc.
5) Image
   - Markdown image syntax: `![alt](src)`
6) Paragraph
   - any other non-empty text until a blank line

### 6.4 Summary tags (ASCII markers)
Summary lines use a 1-letter tag:
- `P:` paragraph
- `Q:` blockquote
- `L:` list
- `C:` codeblock
- `T:` table
- `I:` image

Empty sections emit no summary line.

### 6.5 Payload rules (single line)
Payload must be a single line; join multiple lines with a single space.

- **P:** first paragraph text, whitespace normalized
- **Q:** first blockquote paragraph, strip `>` and join
- **L:** up to 3 items, stripped of list markers, joined with `; `
- **C:** `C: <lang>, <N> lines` (no code content)
- **T:** `T: col1 | col2 | ... (<cols> cols x <rows> rows)`
- **I:** `I: alt="<alt>", src=<data-uri|url|path>` (never include base64)

### 6.6 Format markers vs content text (clarification)
- The metadata markers (`[...]`, `Lx-Ly`, tags like `P:`) MUST use ASCII characters.
- Section titles and summary payload text are emitted as original UTF-8 text (no escaping/encoding transformations).

---

## 7. Commands (MVP)

### 7.1 `mdq tree <file>`
Produces an annotated outline:
- annotated heading lines including `[id Lstart-Lend]`
- optional first-block summary line for non-empty sections

Options:
- `--format annotated-md|json` (default: annotated-md)
- `--max-depth N` (optional): omit headings deeper than N from output
- `--no-summary` (optional): suppress all summary lines

Exit codes:
- 0 success
- 2 CLI usage error
- 3 file read / decode error
- 1 internal error

### 7.2 `mdq get <file> --id <section_id>`
Extracts raw Markdown slice for the section range (START..END), including the heading line by default.

Options:
- `--format text|json` (default: text)
- `--max-lines N` (optional): truncate output after N lines (JSON reports truncation)
- `--with-line-numbers` (text only): prefix `L{line}: `

Exit codes:
- 0 success
- 2 CLI usage error
- 3 file read / decode error
- 4 section id not found
- 1 internal error

### 7.3 `mdq find <file> <query>`
Searches for query matches and maps each match to a section id:
- plain substring search by default
- regex if `--regex` is set
- case-insensitive by default

Options:
- `--format text|json` (default: text)
- `--regex`
- `--case-sensitive`
- `--max-matches N` (default 200)

Exit codes:
- 0 success (even if 0 matches)
- 2 CLI usage error
- 3 file read / decode error
- 1 internal error

---

## 8. Acceptance Criteria

MVP is complete when:
1) `mdq tree` prints annotated headings with stable hierarchical ids and correct line ranges.
2) IDs do not include line numbers and are derived from sibling indices in the built tree.
3) Section END lines follow the “next heading with level <= current” rule.
4) Summary lines:
   - are first-block-only within the defined summary scanning window (before first nested heading)
   - are deterministic and token-efficient plaintext
   - never include base64 content from data-uri images
   - are omitted for empty sections
5) `mdq get` returns the exact original slice for the section range.
6) `mdq find` maps each match to a section id and line number.
