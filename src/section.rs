use crate::document::{Document, Heading};

pub const ROOT_SECTION_ID: &str = "root";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SummaryKind {
    Paragraph,
    Blockquote,
    List,
    Code,
    Table,
    Image,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SummaryBlock {
    pub kind: SummaryKind,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Section {
    pub id: String,
    pub parent_id: String,
    pub level: u8,
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
    pub heading: Heading,
    pub children: Vec<usize>,
    parent_index: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionIndex {
    sections: Vec<Section>,
}

impl SectionIndex {
    pub fn new(document: &Document) -> Self {
        let headings = document.headings();
        let mut sections: Vec<Section> = Vec::with_capacity(headings.len());
        let mut open_stack: Vec<usize> = Vec::new();

        for heading in headings {
            while let Some(&idx) = open_stack.last() {
                if sections[idx].level < heading.level {
                    break;
                }
                open_stack.pop();
            }

            let parent = open_stack.last().copied();
            let next_index = sections.len();
            sections.push(Section {
                id: String::new(),
                parent_id: String::new(),
                level: heading.level,
                title: heading.title.clone(),
                start_line: heading.start_line,
                end_line: document.line_count(),
                heading,
                children: Vec::new(),
                parent_index: parent,
            });

            if let Some(parent_idx) = parent {
                sections[parent_idx].children.push(next_index);
            }

            open_stack.push(next_index);
        }

        let root_children = sections
            .iter()
            .enumerate()
            .filter_map(|(idx, section)| section.parent_index.is_none().then_some(idx))
            .collect::<Vec<_>>();

        assign_section_ids(&mut sections, &root_children, "s");
        fill_parent_ids(&mut sections);
        fill_section_ranges(&mut sections, document.line_count());

        Self { sections }
    }

    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    pub fn by_id(&self, id: &str) -> Option<&Section> {
        self.sections.iter().find(|section| section.id == id)
    }

    pub fn section_for_line(&self, line_number: usize) -> Option<&Section> {
        self.sections
            .iter()
            .rev()
            .find(|section| section.start_line <= line_number && line_number <= section.end_line)
    }
}

impl Section {
    pub fn summary_block(&self, document: &Document) -> Option<SummaryBlock> {
        let scan_start = self.heading.end_line + 1;
        let scan_end = summary_scan_end(document, self);
        if scan_start > scan_end {
            return None;
        }

        let mut line_number = scan_start;
        while line_number <= scan_end {
            let line = document
                .line(line_number)
                .expect("summary scan line should remain in range");

            if line.trim().is_empty() || is_thematic_break(line) {
                line_number += 1;
                continue;
            }

            if is_fenced_code_start(line) {
                return Some(capture_fenced_code(document, line_number, scan_end));
            }

            if is_blockquote_line(line) {
                return Some(capture_contiguous_block(
                    document,
                    line_number,
                    scan_end,
                    SummaryKind::Blockquote,
                    is_blockquote_line,
                ));
            }

            if is_table_start(document, line_number, scan_end) {
                return Some(capture_table(document, line_number, scan_end));
            }

            if is_list_start(line) {
                return Some(capture_list(document, line_number, scan_end));
            }

            return Some(capture_paragraph_like(document, line_number, scan_end));
        }

        None
    }
}

fn assign_section_ids(sections: &mut [Section], child_indices: &[usize], prefix: &str) {
    for (position, &child_idx) in child_indices.iter().enumerate() {
        let id = if prefix == "s" {
            format!("{prefix}{}", position + 1)
        } else {
            format!("{prefix}-{}", position + 1)
        };

        sections[child_idx].id = id.clone();
        let grandchildren = sections[child_idx].children.clone();
        assign_section_ids(sections, &grandchildren, &id);
    }
}

fn fill_section_ranges(sections: &mut [Section], eof_line: usize) {
    let mut open_stack: Vec<usize> = Vec::new();

    for idx in 0..sections.len() {
        while let Some(&open_idx) = open_stack.last() {
            if sections[open_idx].level < sections[idx].level {
                break;
            }

            sections[open_idx].end_line = sections[idx].start_line.saturating_sub(1);
            open_stack.pop();
        }

        open_stack.push(idx);
    }

    for idx in open_stack {
        sections[idx].end_line = eof_line.max(sections[idx].start_line);
    }
}

fn fill_parent_ids(sections: &mut [Section]) {
    for idx in 0..sections.len() {
        sections[idx].parent_id = sections[idx]
            .parent_index
            .map(|parent_idx| sections[parent_idx].id.clone())
            .unwrap_or_else(|| ROOT_SECTION_ID.to_owned());
    }
}

fn summary_scan_end(document: &Document, section: &Section) -> usize {
    let first_nested_heading = document
        .headings()
        .into_iter()
        .find(|heading| {
            heading.start_line > section.heading.end_line
                && heading.start_line <= section.end_line
                && heading.level > section.level
        })
        .map(|heading| heading.start_line.saturating_sub(1))
        .unwrap_or(section.end_line);

    first_nested_heading.min(section.end_line)
}

fn capture_fenced_code(document: &Document, start_line: usize, scan_end: usize) -> SummaryBlock {
    let opening_line = document
        .line(start_line)
        .expect("opening fence line should exist");
    let (fence_char, fence_len) =
        parse_fence(opening_line).expect("fenced code capture should start on a fence");

    let mut end_line = start_line;
    for line_number in start_line + 1..=scan_end {
        let line = document
            .line(line_number)
            .expect("fenced code line should remain in range");
        if is_fence_closing_line(line, fence_char, fence_len) {
            end_line = line_number;
            break;
        }
        end_line = line_number;
    }

    SummaryBlock {
        kind: SummaryKind::Code,
        start_line,
        end_line,
        lines: collect_lines(document, start_line, end_line),
    }
}

fn capture_contiguous_block<F>(
    document: &Document,
    start_line: usize,
    scan_end: usize,
    kind: SummaryKind,
    matches: F,
) -> SummaryBlock
where
    F: Fn(&str) -> bool,
{
    let mut end_line = start_line;
    for line_number in start_line + 1..=scan_end {
        let line = document
            .line(line_number)
            .expect("block line should remain in range");
        if line.trim().is_empty() || !matches(line) {
            break;
        }
        end_line = line_number;
    }

    SummaryBlock {
        kind,
        start_line,
        end_line,
        lines: collect_lines(document, start_line, end_line),
    }
}

fn capture_table(document: &Document, start_line: usize, scan_end: usize) -> SummaryBlock {
    let mut end_line = (start_line + 1).min(scan_end);
    for line_number in start_line + 2..=scan_end {
        let line = document
            .line(line_number)
            .expect("table line should remain in range");
        if line.trim().is_empty() || !line.contains('|') {
            break;
        }
        end_line = line_number;
    }

    SummaryBlock {
        kind: SummaryKind::Table,
        start_line,
        end_line,
        lines: collect_lines(document, start_line, end_line),
    }
}

fn capture_list(document: &Document, start_line: usize, scan_end: usize) -> SummaryBlock {
    let mut end_line = start_line;
    for line_number in start_line + 1..=scan_end {
        let line = document
            .line(line_number)
            .expect("list line should remain in range");
        if line.trim().is_empty() {
            break;
        }
        end_line = line_number;
    }

    SummaryBlock {
        kind: SummaryKind::List,
        start_line,
        end_line,
        lines: collect_lines(document, start_line, end_line),
    }
}

fn capture_paragraph_like(document: &Document, start_line: usize, scan_end: usize) -> SummaryBlock {
    let mut end_line = start_line;
    for line_number in start_line + 1..=scan_end {
        let line = document
            .line(line_number)
            .expect("paragraph line should remain in range");
        if line.trim().is_empty() {
            break;
        }
        end_line = line_number;
    }

    SummaryBlock {
        kind: if is_image_paragraph_start(
            document
                .line(start_line)
                .expect("paragraph start line should exist"),
        ) {
            SummaryKind::Image
        } else {
            SummaryKind::Paragraph
        },
        start_line,
        end_line,
        lines: collect_lines(document, start_line, end_line),
    }
}

fn collect_lines(document: &Document, start_line: usize, end_line: usize) -> Vec<String> {
    (start_line..=end_line)
        .map(|line_number| {
            document
                .line(line_number)
                .expect("captured line should remain in range")
                .to_owned()
        })
        .collect()
}

fn is_fenced_code_start(line: &str) -> bool {
    parse_fence(line).is_some()
}

fn parse_fence(line: &str) -> Option<(char, usize)> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }

    let trimmed = &line[indent..];
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }

    let fence_len = trimmed.chars().take_while(|ch| *ch == first).count();
    (fence_len >= 3).then_some((first, fence_len))
}

fn is_fence_closing_line(line: &str, fence_char: char, fence_len: usize) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }

    let trimmed = &line[indent..];
    let repeated = trimmed.chars().take_while(|ch| *ch == fence_char).count();
    repeated >= fence_len && trimmed[repeated..].trim().is_empty()
}

fn is_blockquote_line(line: &str) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    indent <= 3 && line[indent..].starts_with('>')
}

fn is_table_start(document: &Document, line_number: usize, scan_end: usize) -> bool {
    if line_number + 1 > scan_end {
        return false;
    }

    let header = document
        .line(line_number)
        .expect("table header line should exist");
    let separator = document
        .line(line_number + 1)
        .expect("table separator line should exist");

    header.contains('|') && is_table_separator_row(separator)
}

fn is_table_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.contains('|') {
        return false;
    }

    trimmed
        .trim_matches('|')
        .split('|')
        .all(|cell| is_table_separator_cell(cell.trim()))
}

fn is_table_separator_cell(cell: &str) -> bool {
    let core = cell.trim_matches(':');
    !core.is_empty() && core.chars().all(|ch| ch == '-')
}

fn is_list_start(line: &str) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }

    matches_list_marker(&line[indent..])
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

fn is_image_paragraph_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("![") {
        return false;
    }

    let Some(alt_end) = trimmed.find("](") else {
        return false;
    };

    trimmed[alt_end + 2..].contains(')')
}

fn is_thematic_break(line: &str) -> bool {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }

    let trimmed = line[indent..].trim();
    if trimmed.len() < 3 {
        return false;
    }

    let marker = trimmed.chars().find(|ch| !ch.is_whitespace());
    match marker {
        Some(ch @ ('-' | '*' | '_')) => {
            let count = trimmed.chars().filter(|current| *current == ch).count();
            count >= 3
                && trimmed
                    .chars()
                    .all(|current| current == ch || current.is_whitespace())
        }
        _ => false,
    }
}
