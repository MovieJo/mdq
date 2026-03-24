use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{run_with_io, EXIT_SUCCESS, EXIT_USAGE_ERROR};

fn write_fixture(contents: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("mdq-find-test-{unique}"));
    fs::create_dir_all(&base).expect("temp dir should be created");

    let file_path = base.join("fixture.md");
    fs::write(&file_path, contents).expect("fixture should be written");
    file_path
}

#[test]
fn find_text_searches_lines_maps_sections_and_limits_matches() {
    let file_path = write_fixture(
        "Preamble install note\n# Intro\ninstall here\n## Child\nInstall again\nclosing line\n",
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            file_path.to_str().expect("temp path should be utf-8"),
            "install",
            "--max-matches",
            "2",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(String::from_utf8(stderr).expect("stderr should be utf-8"), "");
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        "L1 [-] Preamble install note\nL3 [s1] install here\n"
    );

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}

#[test]
fn find_json_emits_root_matches_and_regex_case_rules() {
    let file_path = write_fixture("preamble\n# Intro\nInstall\ninstall\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            file_path.to_str().expect("temp path should be utf-8"),
            "^Install$",
            "--format",
            "json",
            "--regex",
            "--case-sensitive",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_SUCCESS);
    assert_eq!(String::from_utf8(stderr).expect("stderr should be utf-8"), "");
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        format!(
            "{{\"command\":\"find\",\"file\":\"{}\",\"format\":\"json\",\"query\":\"^Install$\",\"regex\":true,\"case_sensitive\":true,\"matches\":[{{\"line\":3,\"section_id\":\"s1\",\"text\":\"Install\"}}]}}\n",
            file_path.display()
        )
    );

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}

#[test]
fn find_invalid_regex_is_a_usage_error() {
    let file_path = write_fixture("# Intro\nbody\n");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            file_path.to_str().expect("temp path should be utf-8"),
            "(",
            "--regex",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_USAGE_ERROR);
    assert_eq!(String::from_utf8(stdout).expect("stdout should be utf-8"), "");

    let stderr = String::from_utf8(stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("regex parse error"));

    fs::remove_dir_all(file_path.parent().expect("fixture should have a parent"))
        .expect("temp dir should be removed");
}
