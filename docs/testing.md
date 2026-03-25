# Testing Guide

This document defines the minimum fixture and assertion set required for the MVP.

---

## 1. Core Fixture Cases

The test corpus should include Markdown fixtures for:

- Regular nested headings with body content
- Irregular heading levels such as `h2 -> h4 -> h1`
- Setext headings
- Empty sections
- Parent sections with children only and no body content
- Preamble content before the first heading
- Paragraph, blockquote, list, fenced code, table, and image as the first block
- Data URI images
- CRLF line endings
- Search queries with zero matches
- Search queries with multiple matching lines
- Regex queries
- `get --max-lines` truncation cases

---

## 2. Assertions by Command

### `tree`

- Section ids match the sibling-index hierarchy.
- Fixture and golden outputs override parser-default behavior when they differ.
- Line ranges follow the next heading with level `<= current` rule.
- Setext headings are recognized.
- Summary output uses only the first block in the summary scanning window.
- Empty sections emit no summary.
- Data URI summaries never include base64 payload text.
- `--max-depth` changes output inclusion only, not computed ids or line ranges.

### `get`

- Returned content exactly matches the original slice when not truncated.
- `--with-line-numbers` uses original source line numbers.
- `--max-lines` truncates after extraction.
- Missing section ids return exit code `4`.

### `find`

- Matches are reported in ascending line order.
- Preamble matches map to `root` in JSON and `-` in text output.
- Text output emits one line per matching source line.
- JSON output does not include match-span metadata in MVP.
- Case sensitivity and regex flags alter matching as specified.
- Zero matches return exit code `0`.

---

## 3. Golden Output

For text output formats, golden files are recommended for:

- `tree --format annotated-md`
- `get --with-line-numbers`
- `find` text output

For JSON output, snapshot tests are recommended with deterministic field ordering.

---

## 4. MVP Acceptance Criteria Mapping

- Acceptance criteria 1 and 2: `src/tests/tree.rs` golden and snapshot coverage for annotated tree output, ids, line ranges, and summaries
- Acceptance criteria 3 and 4: `src/tests/tree.rs` snapshot coverage for JSON field ordering and section payload stability
- Acceptance criteria 5: `src/tests/get.rs` exact-slice assertion plus golden and snapshot coverage for truncation and line-numbered output
- Acceptance criteria 6: `src/tests/find.rs` golden and snapshot coverage for text mapping, JSON mapping, and regex/case-sensitive behavior
- Acceptance criteria 7: `src/tests/error.rs` and `src/tests/get.rs` runtime error shape and exit-code assertions
