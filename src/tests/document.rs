use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Document, Heading, HeadingKind};

#[test]
fn document_strips_bom_and_indexes_lf_lines() {
    let doc = Document::from_bytes("fixture.md", b"\xEF\xBB\xBF# Title\nline 2\n")
        .expect("fixture should decode");

    assert_eq!(doc.source(), "# Title\nline 2\n");
    assert_eq!(doc.line_count(), 2);
    assert_eq!(doc.line(1), Some("# Title"));
    assert_eq!(doc.line(2), Some("line 2"));
    assert_eq!(doc.slice_lines(1, 2), Some("# Title\nline 2\n"));
    assert_eq!(doc.line_start_offset(1), Some(0));
    assert_eq!(doc.line_end_offset(1), Some(7));
    assert_eq!(doc.line_start_offset(2), Some(8));
    assert_eq!(doc.line_end_offset(2), Some(14));
}

#[test]
fn document_treats_crlf_as_single_line_break() {
    let doc = Document::from_bytes("fixture.md", b"# Title\r\nline 2\r\nlast line")
        .expect("fixture should decode");

    assert_eq!(doc.line_count(), 3);
    assert_eq!(doc.line(1), Some("# Title"));
    assert_eq!(doc.line(2), Some("line 2"));
    assert_eq!(doc.line(3), Some("last line"));
    assert_eq!(doc.slice_lines(1, 2), Some("# Title\r\nline 2\r\n"));
    assert_eq!(doc.line_start_offset(2), Some(9));
    assert_eq!(doc.line_end_offset(2), Some(15));
}

#[test]
fn document_reports_decode_errors() {
    let err = Document::from_bytes("fixture.md", &[0xff, 0xfe]).expect_err("should fail decode");

    assert_eq!(err.code(), "decode_error");
    assert_eq!(err.message(), "Failed to decode file 'fixture.md' as UTF-8");
}

#[test]
fn document_reads_files_and_reports_read_errors() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("mdq-input-test-{unique}"));
    fs::create_dir_all(&base).expect("temp dir should be created");

    let file_path = base.join("fixture.md");
    fs::write(&file_path, "# Title\r\nBody\n").expect("fixture should be written");

    let doc = Document::read(&file_path).expect("fixture should load");
    assert_eq!(doc.line_count(), 2);
    assert_eq!(doc.slice_lines(1, 2), Some("# Title\r\nBody\n"));

    let missing = base.join("missing.md");
    let err = Document::read(&missing).expect_err("missing file should fail");
    assert_eq!(err.code(), "file_read_error");
    assert!(err
        .message()
        .starts_with(&format!("Failed to read file '{}':", missing.display())));

    fs::remove_dir_all(&base).expect("temp dir should be removed");
}

#[test]
fn document_parses_atx_headings_with_source_positions() {
    let doc = Document::from_bytes(
        "fixture.md",
        b"# Title\nIntro\n### Deep Dive ###\nBody\n####### not a heading\n",
    )
    .expect("fixture should decode");

    assert_eq!(
        doc.headings(),
        vec![
            Heading {
                kind: HeadingKind::Atx,
                level: 1,
                title: "Title".to_owned(),
                start_line: 1,
                end_line: 1,
                start_offset: 0,
                end_offset: 7,
            },
            Heading {
                kind: HeadingKind::Atx,
                level: 3,
                title: "Deep Dive".to_owned(),
                start_line: 3,
                end_line: 3,
                start_offset: 14,
                end_offset: 31,
            },
        ]
    );
}

#[test]
fn document_parses_setext_headings_with_source_positions() {
    let doc = Document::from_bytes(
        "fixture.md",
        b"Title\n=====\n\nSubtitle\n-----\nParagraph\n",
    )
    .expect("fixture should decode");

    assert_eq!(
        doc.headings(),
        vec![
            Heading {
                kind: HeadingKind::Setext,
                level: 1,
                title: "Title".to_owned(),
                start_line: 1,
                end_line: 2,
                start_offset: 0,
                end_offset: 11,
            },
            Heading {
                kind: HeadingKind::Setext,
                level: 2,
                title: "Subtitle".to_owned(),
                start_line: 4,
                end_line: 5,
                start_offset: 13,
                end_offset: 27,
            },
        ]
    );
}

#[test]
fn document_rejects_common_non_heading_setext_candidates() {
    let doc = Document::from_bytes(
        "fixture.md",
        b"> quoted\n-----\n- list item\n-----\n    indented\n-----\n",
    )
    .expect("fixture should decode");

    assert!(doc.headings().is_empty());
}
