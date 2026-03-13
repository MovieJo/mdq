# mdq MVP Specification

## 1. Overview

`mdq` is a CLI tool that helps humans and LLM agents navigate Markdown efficiently by:
- producing an annotated outline (`mdq tree`) with stable section ids and source line ranges,
- emitting a first-block-only, rule-based plaintext summary per non-empty section,
- extracting a section's raw Markdown by section id (`mdq get`),
- finding query matches and mapping them to section ids and line numbers (`mdq find`).

`mdq` supports two output styles:
- **annotated-md** (default for `tree`): a Markdown-compatible outline document with minimal ASCII format markers
- **json**: structured output for tool integration

This document defines MVP behavior and contracts for `mdq tree`, `mdq get`, and `mdq find`.

---

## 2. Non-goals (MVP)

Out of scope for MVP:
- Multi-file or directory recursion
- Full CommonMark/GFM rendering compliance (`mdq` is not a renderer)
- AI summarization (summary is deterministic and rule-based)
- Persistent indexes, caches, or daemon services
- TUI or interactive viewer
- Inline-level AST extraction beyond what is required for first-block classification

---

## 3. Inputs and Basic Rules

### 3.1 Input file
- One Markdown file per invocation.
- File must be UTF-8 decodable. If decoding fails, the command errors.
- A UTF-8 BOM, if present, is ignored for parsing and is not included in output payloads.

### 3.2 Line numbering
- 1-based line numbers.
- Line breaks are treated as `\n`.
- If CRLF exists, treat `\r\n` as one line break.
- Keep behavior deterministic across platforms.

### 3.3 Determinism
For the same file bytes and the same flags:
- output must be identical across runs (no timestamps or randomness)
- ordering is always source order

---

## 4. Section Model

### 4.1 Headings considered
A section is defined by Markdown headings:
- ATX headings (`#`..`######`)
- Setext headings (`===`, `---`) MUST be recognized as headings

Recommended: use a parser that can identify headings and provide source offsets.

Implementation guidance:
- A CommonMark-compatible parser is recommended.
- For MVP, fixture and golden-test results are the normative behavior contract.
- If parser behavior and fixture expectations differ, fixture expectations win.

### 4.2 Virtual root
A virtual root node exists conceptually:
- id: `root`
- depth: `0`

`root` is not printed in annotated-md output, but is present in JSON for parent relations and preamble search matches.

### 4.3 Parent selection for irregular heading structure
Markdown can be irregular:
- start with h2 without h1
- skip heading levels
- introduce h1 late

`mdq` must build a consistent tree without editing the input.

Algorithm (stack-based):
1. When a heading `H` with level `L` (1..6) is encountered in document order, find the nearest prior open node on the stack with level `< L`.
2. If found, that node is the parent. Otherwise parent is `root`.
3. Pop the stack until the top has level `< L`, then push `H`.

This ensures:
- top-level h2 becomes a root child
- h1 -> h3 becomes a child of h1
- late h1 becomes a root child

### 4.4 Preamble content before first heading
Content that appears before the first heading is preamble content.

- Preamble content does not become a section.
- `mdq tree` does not print preamble content.
- `mdq get` cannot address preamble content by id.
- `mdq find` maps matches in preamble content to section id `root` in JSON, and to `-` in text output.

### 4.5 Section line range
Each heading defines a section range that includes the heading line.

For a section `S` defined by heading at line `START` with level `L`:
- `START` = heading line
- `END` = the line immediately before the next heading whose level `<= L`
- if no such heading exists, `END` = EOF line

Always: `START <= END`.

### 4.6 Summary scanning window
To avoid parent sections summarizing their child headings:
- summary scanning MUST consider only the section body before the first nested heading (a heading with level `> current level`) inside that section
- if there is no non-empty content before the first nested heading, the section is treated as empty for summary purposes and emits no summary line

This matches common documentation style where parents are containers for subsections.

---

## 5. Stable Section IDs

### 5.1 Requirements
- IDs MUST NOT include line numbers.
- IDs should remain stable under edits inside a section body.
- IDs may change when heading structure changes (insert, remove, reorder, or re-level headings).

### 5.2 Format
IDs are hierarchical and based on sibling indices in the section tree:
- root's Nth child: `sN`
- any node `X`'s Mth child: `X-M`

Examples:
- `s1`, `s2`
- `s1-1`, `s1-2`
- `s2-3-1`

### 5.3 Assignment rule
- Sibling ordering is source order.
- Indices start at `1`.
- IDs are assigned deterministically after the tree is built.

---

## 6. Rule-based First-block Summary

### 6.1 Goal
Each non-empty section emits at most one summary line describing the first meaningful content block.

### 6.2 First block scanning
For a section range `START..END` with heading level `L`:
- scan begins at line `START + 1`
- scan ends at the earlier of:
  - `END`, or
  - the line before the first nested heading with level `> L` inside `START..END`
- skip blank lines to find the first non-empty line
- if a thematic break (`---`, `***`, `___`) is encountered, continue scanning
- classify the first matching block using the priority below

If no block is found, the section is empty for summary purposes.

### 6.3 Block classification priority
First non-empty block is classified by the first matching rule:

1. Fenced code block
   - line begins with ``` or `~~~` (3+ fence chars), allowing up to 3 leading spaces
2. Blockquote
   - line begins with `>`
3. GFM pipe table
   - a header row containing `|`
   - immediately followed by a separator row like `---|---` or `|:---:|`
4. List
   - unordered: `- `, `* `, `+ `
   - ordered: `1. `, `2. `, etc.
5. Image
   - a paragraph whose first non-whitespace inline content is Markdown image syntax: `![alt](src)`
6. Paragraph
   - any other non-empty text until a blank line

Notes:
- HTML blocks and inline HTML are treated as paragraph content in MVP.
- Indented code blocks are treated as paragraph content in MVP.

### 6.4 Summary tags
Summary lines use a 1-letter tag:
- `P:` paragraph
- `Q:` blockquote
- `L:` list
- `C:` code block
- `T:` table
- `I:` image

### 6.5 Payload rules
Payload must be a single line. Join multiple lines with a single space.

- `P:` first paragraph text, with whitespace normalized
- `Q:` first blockquote paragraph, stripping `>` markers and joining lines
- `L:` up to 3 top-level items, stripped of list markers and joined with `; `
- `C:` `C: <lang>, <N> lines`
- `T:` `T: col1 | col2 | ... (<cols> cols x <rows> rows)`
- `I:` `I: alt="<alt>", src=<data-uri|url|path>`

Additional rules:
- For `P:`, inline Markdown markers may remain as source text.
- For `P:`, `Q:`, and `L:`, normalize line breaks and repeated ASCII whitespace to a single space.
- For `Q:`, only the first contiguous blockquote block is summarized.
- For `L:`, nested list items are ignored.
- For `C:`, `<lang>` is the first info-string token before any whitespace; if absent, use `text`.
- For `C:`, `<N>` counts content lines inside the fence and excludes fence lines.
- For `T:`, `<rows>` counts body rows only, excluding the header row and separator row.
- For `I:`, `src` kinds are normalized to exactly one of `data-uri`, `url`, or `path`.
- Base64 payload text from data URIs MUST NOT be emitted.

### 6.6 Format markers vs content text
- Metadata markers (`[...]`, `Lx-Ly`, tags like `P:`) MUST use ASCII characters.
- Section titles and summary payload text are emitted as original UTF-8 text.

---

## 7. Commands (MVP)

### 7.1 `mdq tree <file>`
Produces an annotated outline:
- annotated heading lines including `[id Lstart-Lend]`
- optional first-block summary line for non-empty sections

Options:
- `--format annotated-md|json` (default: `annotated-md`)
- `--max-depth N` (optional): omit headings deeper than `N` from output
- `--no-summary` (optional): suppress all summary lines

Behavior notes:
- `--max-depth` filters output only.
- Tree construction, id assignment, line ranges, and summaries are computed from the full document.
- If a parent is printed and its children are omitted by `--max-depth`, the parent keeps its original `Lstart-Lend` range.

Exit codes:
- `0` success
- `2` CLI usage error
- `3` file read or decode error
- `1` internal error

### 7.2 `mdq get <file> --id <section_id>`
Extracts the raw Markdown slice for the section range `START..END`, including the heading line.

Options:
- `--format text|json` (default: `text`)
- `--max-lines N` (optional): truncate output after `N` lines
- `--with-line-numbers` (text only): prefix `L{line}: `

Behavior notes:
- Without `--max-lines`, the returned content MUST be the exact original slice for the section range.
- With `--max-lines`, truncation is line-based after extracting the exact slice.
- `--with-line-numbers` uses original file line numbers, not section-relative numbers.
- Text output with `--max-lines` does not add an ellipsis marker.
- JSON output reports truncation explicitly.

Exit codes:
- `0` success
- `2` CLI usage error
- `3` file read or decode error
- `4` section id not found
- `1` internal error

### 7.3 `mdq find <file> <query>`
Searches for query matches and maps each match to a section id:
- plain substring search by default
- regex if `--regex` is set
- case-insensitive by default

Options:
- `--format text|json` (default: `text`)
- `--regex`
- `--case-sensitive`
- `--max-matches N` (default `200`)

Behavior notes:
- Search operates on the original file text line by line.
- In substring mode, a line matches if it contains the query at least once.
- In regex mode, a line matches if the regex matches at least once.
- Output is one result per matching source line.
- Matching lines are emitted in ascending source line order.
- Text output format is `L<line> [<section-id-or-dash>] <line-text>`.
- Matches before the first heading use `-` in text output and `root` in JSON.
- `--max-matches` limits the number of emitted result lines.
- JSON match objects are line-based in MVP and do not include column or span metadata.
- An invalid regex is a CLI usage error.

Exit codes:
- `0` success, including zero matches
- `2` CLI usage error
- `3` file read or decode error
- `1` internal error

---

## 8. Acceptance Criteria

MVP is complete when:
1. `mdq tree` prints annotated headings with stable hierarchical ids and correct line ranges.
2. IDs do not include line numbers and are derived from sibling indices in the built tree.
3. Section end lines follow the "next heading with level <= current" rule.
4. Summary lines:
   - are first-block-only within the defined summary scanning window
   - are deterministic and token-efficient plaintext
   - never include base64 content from data-uri images
   - are omitted for empty sections
5. `mdq get` returns the exact original slice for the section range when not truncated.
6. `mdq find` maps each emitted match line to a section id and line number.
7. JSON output for each command conforms to `docs/json-output-v1.md`.
