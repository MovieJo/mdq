use crate::{run_with_io, EXIT_SUCCESS, EXIT_USAGE_ERROR};

use super::fixtures::TempFixture;

#[test]
fn find_text_searches_lines_maps_sections_and_limits_matches() {
    let fixture = TempFixture::new("edge/find-cases.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "install",
            "--max-matches",
            "2",
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
        "L1 [-] Preamble install note\nL3 [s1] install here\n"
    );
}

#[test]
fn find_json_emits_root_matches_and_regex_case_rules() {
    let fixture = TempFixture::new("edge/find-cases.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "^Install again$",
            "--format",
            "json",
            "--regex",
            "--case-sensitive",
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
            "{{\"command\":\"find\",\"file\":\"{}\",\"format\":\"json\",\"query\":\"^Install again$\",\"regex\":true,\"case_sensitive\":true,\"matches\":[{{\"line\":5,\"section_id\":\"s1-1\",\"text\":\"Install again\"}}]}}\n",
            fixture.path().display()
        )
    );
}

#[test]
fn find_invalid_regex_is_a_usage_error() {
    let fixture = TempFixture::new("edge/zero-match.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "(",
            "--regex",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_USAGE_ERROR);
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        ""
    );

    let stderr = String::from_utf8(stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("regex parse error"));
}
