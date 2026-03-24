use crate::{Document, SummaryKind};

use super::fixtures::fixture_bytes;

#[test]
fn summary_block_classifies_each_supported_first_block_kind() {
    let bytes = fixture_bytes("blocks/first-block-kinds.md");
    let document = Document::from_bytes("fixture.md", &bytes).expect("fixture should decode");
    let sections = document.section_index();

    let expected = [
        ("s1", SummaryKind::Paragraph, 2, 2),
        ("s2", SummaryKind::Blockquote, 5, 6),
        ("s3", SummaryKind::List, 9, 12),
        ("s4", SummaryKind::Code, 15, 19),
        ("s5", SummaryKind::Table, 22, 25),
        ("s6", SummaryKind::Image, 28, 28),
    ];

    for (id, kind, start_line, end_line) in expected {
        let block = sections
            .by_id(id)
            .expect("fixture section should exist")
            .summary_block(&document)
            .expect("fixture section should have a summary block");

        assert_eq!(block.kind, kind, "unexpected kind for {id}");
        assert_eq!(
            block.start_line, start_line,
            "unexpected start line for {id}"
        );
        assert_eq!(block.end_line, end_line, "unexpected end line for {id}");
    }
}

#[test]
fn summary_block_skips_blank_lines_and_thematic_breaks() {
    let document = Document::from_bytes(
        "fixture.md",
        b"# Intro\n\n---\n***\n> quoted line\n> second line\n",
    )
    .expect("fixture should decode");
    let sections = document.section_index();
    let section = sections.by_id("s1").expect("section should exist");

    let block = section
        .summary_block(&document)
        .expect("section should have a summary block");

    assert_eq!(block.kind, SummaryKind::Blockquote);
    assert_eq!(block.start_line, 5);
    assert_eq!(block.end_line, 6);
}

#[test]
fn summary_block_stops_before_first_nested_heading() {
    let document = Document::from_bytes(
        "fixture.md",
        b"# Parent\n\n## Child\nchild body\n\n# Parent With Body\nintro line\n## Nested\nchild body\n",
    )
    .expect("fixture should decode");
    let sections = document.section_index();

    assert_eq!(
        sections
            .by_id("s1")
            .expect("section should exist")
            .summary_block(&document),
        None
    );

    let block = sections
        .by_id("s2")
        .expect("section should exist")
        .summary_block(&document)
        .expect("section should have a summary block");
    assert_eq!(block.kind, SummaryKind::Paragraph);
    assert_eq!(block.start_line, 7);
    assert_eq!(block.end_line, 7);
}

#[test]
fn code_summary_uses_full_opening_fence_length_when_parsing_language() {
    let document = Document::from_bytes(
        "fixture.md",
        b"# Backticks\n````rust extra\nprintln!(\"hi\");\n````\n\n# Tildes\n~~~~python\nprint('hi')\n~~~~\n",
    )
    .expect("fixture should decode");
    let sections = document.section_index();

    let backticks = sections
        .by_id("s1")
        .expect("section should exist")
        .summary_block(&document)
        .expect("section should have a summary block");
    assert_eq!(backticks.kind, SummaryKind::Code);
    assert_eq!(backticks.payload(), "rust, 1 lines");

    let tildes = sections
        .by_id("s2")
        .expect("section should exist")
        .summary_block(&document)
        .expect("section should have a summary block");
    assert_eq!(tildes.kind, SummaryKind::Code);
    assert_eq!(tildes.payload(), "python, 1 lines");
}
