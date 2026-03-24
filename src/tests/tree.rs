use crate::{run_with_io, EXIT_SUCCESS};

use super::fixtures::TempFixture;

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
        "# [s1 L1-L3] Paragraph\nP: Plain paragraph summary text.\n\n# [s2 L4-L7] Blockquote\nQ: quoted line one quoted line two\n\n# [s3 L8-L13] List\nL: first item; second item; third item\n\n# [s4 L14-L20] Code\nC: rust, 3 lines\n\n# [s5 L21-L26] Table\nT: name | value (2 cols x 2 rows)\n\n# [s6 L27-L28] Image\nI: alt=\"diagram\", src=path\n"
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
        format!(
            "{{\"command\":\"tree\",\"file\":\"{}\",\"format\":\"json\",\"sections\":[{{\"id\":\"s1\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Paragraph\",\"start_line\":1,\"end_line\":3,\"summary\":{{\"tag\":\"P\",\"text\":\"Plain paragraph summary text.\"}}}},{{\"id\":\"s2\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Blockquote\",\"start_line\":4,\"end_line\":7,\"summary\":{{\"tag\":\"Q\",\"text\":\"quoted line one quoted line two\"}}}},{{\"id\":\"s3\",\"parent_id\":\"root\",\"level\":1,\"title\":\"List\",\"start_line\":8,\"end_line\":13,\"summary\":{{\"tag\":\"L\",\"text\":\"first item; second item; third item\"}}}},{{\"id\":\"s4\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Code\",\"start_line\":14,\"end_line\":20,\"summary\":{{\"tag\":\"C\",\"text\":\"rust, 3 lines\"}}}},{{\"id\":\"s5\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Table\",\"start_line\":21,\"end_line\":26,\"summary\":{{\"tag\":\"T\",\"text\":\"name | value (2 cols x 2 rows)\"}}}},{{\"id\":\"s6\",\"parent_id\":\"root\",\"level\":1,\"title\":\"Image\",\"start_line\":27,\"end_line\":28,\"summary\":{{\"tag\":\"I\",\"text\":\"alt=\\\"diagram\\\", src=path\"}}}}]}}\n",
            fixture.path().display()
        )
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
