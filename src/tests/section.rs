use crate::{Document, ROOT_SECTION_ID};

use super::fixtures::fixture_bytes;

#[test]
fn section_index_builds_stable_ids_and_parent_relations_for_irregular_headings() {
    let bytes = fixture_bytes("headings/irregular-levels.md");
    let doc = Document::from_bytes("fixture.md", &bytes).expect("fixture should decode");

    let index = doc.section_index();
    let sections = index.sections();

    assert_eq!(sections.len(), 6);
    assert_eq!(sections[0].id, "s1");
    assert_eq!(sections[0].parent_id, ROOT_SECTION_ID);
    assert_eq!(sections[0].level, 2);
    assert_eq!(sections[0].children, vec![1]);

    assert_eq!(sections[1].id, "s1-1");
    assert_eq!(sections[1].parent_id, "s1");
    assert_eq!(sections[1].level, 4);

    assert_eq!(sections[2].id, "s2");
    assert_eq!(sections[2].parent_id, ROOT_SECTION_ID);
    assert_eq!(sections[2].level, 1);
    assert_eq!(sections[2].children, vec![3, 5]);

    assert_eq!(sections[3].id, "s2-1");
    assert_eq!(sections[3].parent_id, "s2");
    assert_eq!(sections[3].children, vec![4]);

    assert_eq!(sections[4].id, "s2-1-1");
    assert_eq!(sections[4].parent_id, "s2-1");
    assert_eq!(sections[5].id, "s2-2");
    assert_eq!(sections[5].parent_id, "s2");
}

#[test]
fn section_index_keeps_preamble_and_empty_setext_sections_in_fixture_corpus() {
    let bytes = fixture_bytes("headings/setext-preamble-empty.md");
    let doc = Document::from_bytes("fixture.md", &bytes).expect("fixture should decode");

    let index = doc.section_index();

    assert_eq!(index.sections().len(), 4);
    assert_eq!(index.section_for_line(1), None);
    assert_eq!(
        index.by_id("s1").map(|section| section.title.as_str()),
        Some("Title")
    );
    assert_eq!(
        index.by_id("s2").map(|section| section.title.as_str()),
        Some("Parent Only")
    );
    assert_eq!(
        index.by_id("s1-1").map(|section| section.title.as_str()),
        Some("Empty Section")
    );
    assert_eq!(
        index.by_id("s1-1").map(|section| section.start_line),
        Some(9)
    );
    assert_eq!(
        index.by_id("s1-1").map(|section| section.end_line),
        Some(11)
    );
    assert_eq!(
        index.by_id("s2-1").map(|section| section.title.as_str()),
        Some("Child Section")
    );
    assert_eq!(
        index
            .by_id("s2-1")
            .map(|section| section.parent_id.as_str()),
        Some("s2")
    );
}

#[test]
fn section_index_computes_section_ranges_and_preamble_mapping() {
    let doc = Document::from_bytes(
        "fixture.md",
        b"Preamble\n# Title\nIntro\n## Child\nChild body\n### Deep\nDeep body\n## Last\nTail\n",
    )
    .expect("fixture should decode");

    let index = doc.section_index();

    assert_eq!(index.by_id("s1").map(|section| section.start_line), Some(2));
    assert_eq!(index.by_id("s1").map(|section| section.end_line), Some(9));
    assert_eq!(
        index.by_id("s1-1").map(|section| section.start_line),
        Some(4)
    );
    assert_eq!(index.by_id("s1-1").map(|section| section.end_line), Some(7));
    assert_eq!(
        index.by_id("s1-1-1").map(|section| section.end_line),
        Some(7)
    );
    assert_eq!(
        index.by_id("s1-2").map(|section| section.start_line),
        Some(8)
    );
    assert_eq!(index.by_id("s1-2").map(|section| section.end_line), Some(9));

    assert_eq!(index.section_for_line(1), None);
    assert_eq!(
        index.section_for_line(2).map(|section| section.id.as_str()),
        Some("s1")
    );
    assert_eq!(
        index.section_for_line(5).map(|section| section.id.as_str()),
        Some("s1-1")
    );
    assert_eq!(
        index.section_for_line(7).map(|section| section.id.as_str()),
        Some("s1-1-1")
    );
    assert_eq!(
        index.section_for_line(9).map(|section| section.id.as_str()),
        Some("s1-2")
    );
}

#[test]
fn empty_documents_produce_an_empty_section_index() {
    let doc = Document::from_bytes("fixture.md", b"preamble only\n\nstill no headings\n")
        .expect("fixture should decode");

    let index = doc.section_index();

    assert!(index.is_empty());
    assert!(index.by_id("s1").is_none());
    assert!(index.section_for_line(1).is_none());
}
