use crate::document::{Document, Heading};

pub const ROOT_SECTION_ID: &str = "root";

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
