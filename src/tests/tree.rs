use crate::{run_with_io, EXIT_SUCCESS};

use super::fixtures::{expected_output, expected_output_with_file, TempFixture};

#[test]
fn tree_annotated_md_renders_headings_ranges_and_summaries() {
    let fixture = TempFixture::new("blocks/first-block-kinds.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "tree",
            fixture.path().to_str().expect("temp path should be utf-8"),
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        expected_output("golden/tree-annotated-md-first-block-kinds.out")
    );
}

#[test]
fn tree_annotated_md_supports_depth_filter_and_summary_suppression() {
    let fixture = TempFixture::new("headings/regular-nested.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "tree",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--max-depth",
            "2",
            "--no-summary",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        "# [s1 L1-L6] Intro\n\n## [s1-1 L3-L6] Child\n"
    );
}

#[test]
fn tree_json_renders_sections_and_summaries() {
    let fixture = TempFixture::new("blocks/first-block-kinds.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "tree",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--format",
            "json",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        expected_output_with_file("snapshots/tree-json-first-block-kinds.json", fixture.path())
    );
}

#[test]
fn tree_json_supports_depth_filter_and_summary_suppression() {
    let fixture = TempFixture::new("headings/regular-nested.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "tree",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--format",
            "json",
            "--max-depth",
            "2",
            "--no-summary",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        format!(
            "{{\"command\":\"tree\",\"file\":\"{}\",\"format\":\"json\",\"sections\":[{{\"id\":\"s1\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Intro\",\"start_line\":1,\"end_line\":6}},{{\"id\":\"s1-1\",\"parent_id\":\"s1\",\"level\":2,\"title\":\"Child\",\"start_line\":3,\"end_line\":6}}]}}\n",
            fixture.path().display()
        )
    );
}
