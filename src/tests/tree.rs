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
