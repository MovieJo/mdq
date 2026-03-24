use crate::{run_with_io, EXIT_SECTION_NOT_FOUND, EXIT_SUCCESS};

use super::fixtures::TempFixture;

#[test]
fn get_text_emits_exact_section_slice() {
    let fixture = TempFixture::new("edge/get-truncation.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--id",
            "s1",
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
        "# Intro\nline 1\n## Child\nchild line\n"
    );
}

#[test]
fn get_text_supports_truncation_and_line_numbers() {
    let fixture = TempFixture::new("edge/get-truncation.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--id",
            "s1",
            "--max-lines",
            "3",
            "--with-line-numbers",
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
        "L1: # Intro\nL2: line 1\nL3: ## Child\n"
    );
}

#[test]
fn get_json_reports_section_bounds_and_truncation() {
    let fixture = TempFixture::new("edge/get-truncation.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--id",
            "s1-1",
            "--format",
            "json",
            "--max-lines",
            "1",
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
            "{{\"command\":\"get\",\"file\":\"{}\",\"format\":\"json\",\"id\":\"s1-1\",\"start_line\":3,\"end_line\":4,\"truncated\":true,\"content\":\"## Child\\n\"}}\n",
            fixture.path().display()
        )
    );
}

#[test]
fn get_json_emits_section_not_found_error_shape() {
    let fixture = TempFixture::new("edge/zero-match.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--id",
            "s9",
            "--format",
            "json",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SECTION_NOT_FOUND);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        "{\"error\":{\"code\":\"section_not_found\",\"message\":\"Section id not found: s9\"}}\n"
    );
}
