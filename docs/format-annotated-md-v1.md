# Annotated Markdown Format v1 (mdq tree default)

This document defines the exact text output format for:
- `mdq tree --format annotated-md`

The output is valid Markdown and is intended to be read directly.
It includes minimal markers for navigation: section ids and line ranges.

Important: **Only the format markers are ASCII-only.** Titles and summary payload text remain original UTF-8.

---

## 1. Encoding

- Output is UTF-8.
- Format markers are ASCII characters:
  - heading meta: `[s1-2 L10-L14]`
  - summary tags: `P:`, `T:`, etc.
- Titles and summary payload text are emitted as-is (UTF-8), with no escaping.

---

## 2. Document Structure

The output is a sequence of *section blocks* in document order.

A section block contains:
1) Heading line (required)
2) Summary line (optional; omitted if section is empty for summary)
3) Optional blank line (allowed, not required)

No other lines are printed.

---

## 3. Heading Line

### 3.1 Syntax

```

<Hashes><space><Meta><space><Title>

```

- `<Hashes>`: 1..6 `#` characters (same style as Markdown headings)
- `<Meta>`: `[<Id><space>L<Start>-L<End>]`
- `<Title>`: original title text (UTF-8 allowed)

Example:
```

## [s1-2 L10-L14] Install

```

### 3.2 Meta segment

Meta is strictly:
```

[<Id><space>L<Start>-L<End>]

```

- `<Id>` matches: `^s[1-9][0-9]*(?:-[1-9][0-9]*)*$`
- `<Start>` and `<End>` are positive integers
- `Start <= End`
- No additional keys are present in v1.

---

## 4. Summary Line

### 4.1 Presence rule
A summary line is printed if and only if the section has a first content block within its summary scanning window.

If the section has no content before its first nested heading, it is treated as empty for summary and emits no summary line.

### 4.2 Syntax
Summary line is plain text:
```

<Tag>:<space><Payload>

```

`<Tag>` is exactly one of:
- `P` paragraph
- `Q` quote (blockquote)
- `L` list
- `C` code
- `T` table
- `I` image

`<Payload>` is a single line (no newline). UTF-8 allowed.

Examples:
```

P: This section explains the workflow.
C: sh, 3 lines
T: id | title | range (3 cols x 5 rows)
I: alt="arch", src=data-uri

```

---

## 5. Optional Indentation (non-normative)

To improve readability, mdq may indent section blocks by tree depth:
- recommended: 2 spaces per depth level (depth-1 indentation)

Indentation is optional. Parsers must not depend on indentation.

---

## 6. Minimal Parser Guidance (non-normative)

Heading regex (illustrative):
- `^(#{1,6})\s+\[([A-Za-z0-9-]+)\s+L(\d+)-L(\d+)\]\s+(.*)$`

Summary regex:
- `^([PQLCTI]):\s(.*)$`

Summary belongs to the immediately preceding heading.
