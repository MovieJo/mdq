use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    run, run_with_io, AppError, InputError, UsageError, EXIT_FILE_ERROR, EXIT_INTERNAL_ERROR,
    EXIT_SECTION_NOT_FOUND, EXIT_USAGE_ERROR,
};

use super::fixtures::TempFixture;

#[test]
fn app_error_maps_codes_exit_codes_and_json_shape() {
    let usage = AppError::from(UsageError::new("bad flag"));
    assert_eq!(usage.code(), "usage_error");
    assert_eq!(usage.exit_code(), EXIT_USAGE_ERROR);
    assert_eq!(
        usage.render_json(),
        "{\"error\":{\"code\":\"usage_error\",\"message\":\"bad flag\"}}"
    );

    let file = AppError::from(InputError::Decode {
        path: "broken.md".into(),
    });
    assert_eq!(file.code(), "decode_error");
    assert_eq!(file.exit_code(), EXIT_FILE_ERROR);

    let missing = AppError::SectionNotFound {
        id: "s9".to_owned(),
    };
    assert_eq!(missing.code(), "section_not_found");
    assert_eq!(missing.exit_code(), EXIT_SECTION_NOT_FOUND);
    assert_eq!(missing.message(), "Section id not found: s9");

    let internal = AppError::Internal {
        message: "boom".to_owned(),
    };
    assert_eq!(internal.code(), "internal_error");
    assert_eq!(internal.exit_code(), EXIT_INTERNAL_ERROR);
}

#[test]
fn tree_returns_file_error_exit_code_for_missing_input() {
    let exit = run(["mdq", "tree", "missing-file.md"]);
    assert_eq!(exit, EXIT_FILE_ERROR);
}

#[test]
fn get_returns_section_not_found_exit_code_for_unknown_id() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let base = std::env::temp_dir().join(format!("mdq-error-test-{unique}"));
    fs::create_dir_all(&base).expect("temp dir should be created");

    let file_path = base.join("fixture.md");
    fs::write(&file_path, "# Title\nBody\n").expect("fixture should be written");

    let exit = run([
        "mdq",
        "get",
        file_path.to_str().expect("temp path should be utf-8"),
        "--id",
        "s9",
    ]);
    assert_eq!(exit, EXIT_SECTION_NOT_FOUND);

    fs::remove_dir_all(&base).expect("temp dir should be removed");
}

#[test]
fn get_json_validation_error_emits_usage_error_object() {
    let fixture = TempFixture::new("edge/zero-match.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "get",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "--id",
            "s1",
            "--format",
            "json",
            "--with-line-numbers",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_USAGE_ERROR);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );
    assert_eq!(
        String::from_utf8(stdout).expect("stdout should be utf-8"),
        "{\"error\":{\"code\":\"usage_error\",\"message\":\"--with-line-numbers can only be used with --format text\"}}\n"
    );
}

#[test]
fn find_json_parse_error_emits_usage_error_object() {
    let fixture = TempFixture::new("edge/zero-match.md");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let exit = run_with_io(
        [
            "mdq",
            "find",
            fixture.path().to_str().expect("temp path should be utf-8"),
            "install",
            "--format",
            "json",
            "--max-matches",
            "nope",
        ],
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, EXIT_USAGE_ERROR);
    assert_eq!(
        String::from_utf8(stderr).expect("stderr should be utf-8"),
        ""
    );

    let stdout = String::from_utf8(stdout).expect("stdout should be utf-8");
    assert_eq!(
        stdout,
        "{\"error\":{\"code\":\"usage_error\",\"message\":\"invalid value 'nope' for '--max-matches <N>': invalid digit found in string\"}}\n"
    );
}
