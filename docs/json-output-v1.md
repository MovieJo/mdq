# JSON Output v1

This document defines the MVP JSON output contracts for:
- `mdq tree --format json`
- `mdq get --format json`
- `mdq find --format json`

All JSON output is UTF-8 encoded and emitted as a single JSON object.

---

## 1. Common Conventions

- Keys use `snake_case`.
- Line numbers are 1-based.
- Source ordering is preserved.
- Unknown extra fields must not be emitted in v1.
- On success, commands exit with code `0` and emit the success object defined below.
- On failure, commands emit the error object defined in section 5.

---

## 2. `tree`

### 2.1 Shape

```json
{
  "command": "tree",
  "file": "README.md",
  "format": "json",
  "sections": [
    {
      "id": "s1",
      "parent_id": "root",
      "level": 1,
      "title": "Main Header",
      "start_line": 1,
      "end_line": 30,
      "summary": {
        "tag": "P",
        "text": "High-level overview."
      }
    }
  ]
}
```

### 2.2 Field Rules

- `command`: exactly `"tree"`
- `file`: the file path exactly as provided on the command line; do not canonicalize in v1
- `format`: exactly `"json"`
- `sections`: printed sections in document order after applying output filters such as `--max-depth`

Each section object contains:
- `id`: section id
- `parent_id`: `"root"` for root children, otherwise the parent section id
- `level`: heading level `1..6`
- `title`: original heading title text
- `start_line`: section start line
- `end_line`: section end line
- `summary`: omitted if the section has no summary or if `--no-summary` is set

`summary` contains:
- `tag`: one of `P`, `Q`, `L`, `C`, `T`, `I`
- `text`: payload text without the tag prefix

---

## 3. `get`

### 3.1 Shape

```json
{
  "command": "get",
  "file": "README.md",
  "format": "json",
  "id": "s1-2",
  "start_line": 19,
  "end_line": 24,
  "truncated": false,
  "content": "## Data Model\n..."
}
```

### 3.2 Field Rules

- `command`: exactly `"get"`
- `file`: the file path exactly as provided on the command line; do not canonicalize in v1
- `format`: exactly `"json"`
- `id`: requested section id
- `start_line`: section start line
- `end_line`: section end line
- `truncated`: boolean
- `content`: extracted text payload

Behavior:
- Without `--max-lines`, `truncated` is always `false` and `content` is the exact original slice.
- With `--max-lines`, `content` contains only the first `N` lines of the extracted slice and `truncated` is `true` if additional lines were omitted.

---

## 4. `find`

### 4.1 Shape

```json
{
  "command": "find",
  "file": "README.md",
  "format": "json",
  "query": "install",
  "regex": false,
  "case_sensitive": false,
  "matches": [
    {
      "line": 13,
      "section_id": "s1-1",
      "text": "### Install"
    }
  ]
}
```

### 4.2 Field Rules

- `command`: exactly `"find"`
- `file`: the file path exactly as provided on the command line; do not canonicalize in v1
- `format`: exactly `"json"`
- `query`: the original query string
- `regex`: boolean
- `case_sensitive`: boolean
- `matches`: result lines in ascending source line order

Each match object contains:
- `line`: original source line number
- `section_id`: containing section id, or `"root"` for preamble content before the first heading
- `text`: the full original line text, without trailing newline characters

Notes:
- v1 emits at most one match object per source line, even if the line contains multiple matching spans.
- v1 does not include match column ranges or span metadata.

---

## 5. Error Shape

All command failures emit:

```json
{
  "error": {
    "code": "section_not_found",
    "message": "Section id not found: s9"
  }
}
```

Rules:
- Top-level success fields must not be mixed with top-level `error`.
- `error.code` is a stable machine-readable code.
- `error.message` is a human-readable message.

Recommended error codes:
- `usage_error`
- `file_read_error`
- `decode_error`
- `section_not_found`
- `internal_error`
