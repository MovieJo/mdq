use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{run_with_io, EXIT_SECTION_NOT_FOUND, EXIT_SUCCESS};

fn write_fixture(contents: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("mdq-get-test-{unique}"));
    fs::create_dir_all(&base).expect("temp dir should be created");

    let file_path = base.join("fixture.md");
    fs::write(&file_path, contents).expect("fixture should be written");
    file_path
}

#[test]
fn get_text_emits_exact_section_slice() {
    let file_path = write_fixture("# Intro\nline 1\n## Child\nchild line\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            file_path.to_str().expect("temp path should be utf-8"),
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

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}

#[test]
fn get_text_supports_truncation_and_line_numbers() {
    let file_path = write_fixture("# Intro\nline 1\n## Child\nchild line");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            file_path.to_str().expect("temp path should be utf-8"),
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

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}

#[test]
fn get_json_reports_section_bounds_and_truncation() {
    let file_path = write_fixture("# Intro\nline 1\n## Child\nchild line\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            file_path.to_str().expect("temp path should be utf-8"),
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
            file_path.display()
        )
    );

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}

#[test]
fn get_json_emits_section_not_found_error_shape() {
    let file_path = write_fixture("# Intro\nline 1\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            file_path.to_str().expect("temp path should be utf-8"),
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

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}
