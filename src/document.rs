use std::fs;
use std::path::PathBuf;

use crate::error::InputError;
use crate::section::SectionIndex;

#[derive(Clone, Debug, Eq, PartialEq)]
struct LineRange {
    content_start: usize,
    content_end: usize,
    full_end: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Document {
    source: String,
    lines: Vec<LineRange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeadingKind {
    Atx,
    Setext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heading {
    pub kind: HeadingKind,
    pub level: u8,
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Document {
    pub fn read(path: impl Into<PathBuf>) -> Result<Self, InputError> {
        let path = path.into();
        let bytes = fs::read(&path).map_err(|source| InputError::FileRead {
            path: path.clone(),
            source,
        })?;

        Self::from_bytes(path, &bytes)
    }

    pub fn from_bytes(path: impl Into<PathBuf>, bytes: &[u8]) -> Result<Self, InputError> {
        let path = path.into();
        let source = String::from_utf8(bytes.to_vec())
            .map_err(|_| InputError::Decode { path: path.clone() })?;
        let source = source
            .strip_prefix('\u{feff}')
            .unwrap_or(&source)
            .to_owned();

        Ok(Self {
            lines: index_lines(&source),
            source,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, line_number: usize) -> Option<&str> {
        let range = self.line_range(line_number)?;
        Some(&self.source[range.content_start..range.content_end])
    }

    pub fn slice_lines(&self, start_line: usize, end_line: usize) -> Option<&str> {
        if start_line == 0 || start_line > end_line {
            return None;
        }

        let start = self.line_range(start_line)?.content_start;
        let end = self.line_range(end_line)?.full_end;
        Some(&self.source[start..end])
    }

    pub fn line_start_offset(&self, line_number: usize) -> Option<usize> {
        Some(self.line_range(line_number)?.content_start)
    }

    pub fn line_end_offset(&self, line_number: usize) -> Option<usize> {
        Some(self.line_range(line_number)?.content_end)
    }

    pub fn headings(&self) -> Vec<Heading> {
        parse_headings(self)
    }

    pub fn section_index(&self) -> SectionIndex {
        SectionIndex::new(self)
    }

    fn line_range(&self, line_number: usize) -> Option<&LineRange> {
        self.lines.get(line_number.checked_sub(1)?)
    }
}

fn index_lines(source: &str) -> Vec<LineRange> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut line_start = 0;

    for (idx, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            let content_end = if idx > line_start && bytes[idx - 1] == b'\r' {
                idx - 1
            } else {
                idx
            };

            lines.push(LineRange {
                content_start: line_start,
                content_end,
                full_end: idx + 1,
            });
            line_start = idx + 1;
        }
    }

    if line_start < bytes.len() {
        lines.push(LineRange {
            content_start: line_start,
            content_end: bytes.len(),
            full_end: bytes.len(),
        });
    }

    lines
}

fn parse_headings(document: &Document) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut line_number = 1;

    while line_number <= document.line_count() {
        let line = document
            .line(line_number)
            .expect("line_number should always be valid while scanning headings");

        if let Some((level, title)) = parse_atx_heading(line) {
            headings.push(Heading {
                kind: HeadingKind::Atx,
                level,
                title,
                start_line: line_number,
                end_line: line_number,
                start_offset: document
                    .line_start_offset(line_number)
                    .expect("heading line should have a start offset"),
                end_offset: document
                    .line_end_offset(line_number)
                    .expect("heading line should have an end offset"),
            });
            line_number += 1;
            continue;
        }

        if line_number < document.line_count() {
            let next_line = document
                .line(line_number + 1)
                .expect("next line should exist while checking setext headings");
            if let Some(level) = parse_setext_underline(next_line) {
                if is_setext_heading_text(line) {
                    headings.push(Heading {
                        kind: HeadingKind::Setext,
                        level,
                        title: line.trim().to_owned(),
                        start_line: line_number,
                        end_line: line_number + 1,
                        start_offset: document
                            .line_start_offset(line_number)
                            .expect("setext heading should have a start offset"),
                        end_offset: document
                            .line_end_offset(line_number + 1)
                            .expect("setext heading should have an end offset"),
                    });
                    line_number += 2;
                    continue;
                }
            }
        }

        line_number += 1;
    }

    headings
}

fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }

    let trimmed = &line[indent..];
    let marker_len = trimmed.chars().take_while(|ch| *ch == '#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }

    let rest = &trimmed[marker_len..];
    if !rest.is_empty() && !matches!(rest.as_bytes()[0], b' ' | b'\t') {
        return None;
    }

    let content = rest.trim();
    let title = strip_atx_closing_sequence(content);
    Some((marker_len as u8, title.to_owned()))
}

fn strip_atx_closing_sequence(content: &str) -> &str {
    let trimmed = content.trim_end();
    let hash_count = trimmed.chars().rev().take_while(|ch| *ch == '#').count();

    if hash_count == 0 {
        return trimmed;
    }

    let without_hashes = &trimmed[..trimmed.len() - hash_count];
    if without_hashes.ends_with([' ', '\t']) {
        without_hashes.trim_end()
    } else {
        trimmed
    }
}

fn parse_setext_underline(line: &str) -> Option<u8> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.chars().all(|ch| ch == '=') {
        return Some(1);
    }

    if trimmed.chars().all(|ch| ch == '-') {
        return Some(2);
    }

    None
}

fn is_setext_heading_text(line: &str) -> bool {
    if line.trim().is_empty() {
        return false;
    }

    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }

    let trimmed = line.trim_start();
    if trimmed.starts_with('>') {
        return false;
    }
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return false;
    }
    if parse_atx_heading(line).is_some() {
        return false;
    }
    if matches_list_marker(trimmed) {
        return false;
    }

    true
}

fn matches_list_marker(line: &str) -> bool {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return true;
    }

    let mut digits = 0usize;
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
            continue;
        }
        return digits > 0 && ch == '.' && line[digits + 1..].starts_with(' ');
    }

    false
}
