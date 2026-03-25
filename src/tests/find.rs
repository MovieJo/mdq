use crate::{run_with_io, EXIT_SUCCESS, EXIT_USAGE_ERROR};

use super::fixtures::{expected_output, expected_output_with_file, TempFixture};

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
        expected_output("golden/find-text-install.out")
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
        expected_output_with_file(
            "snapshots/find-json-regex-case-sensitive.json",
            fixture.path(),
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
