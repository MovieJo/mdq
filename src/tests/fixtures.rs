use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn fixture_path(relative: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/fixtures")
        .join(relative)
}

pub(crate) fn fixture_bytes(relative: impl AsRef<Path>) -> Vec<u8> {
    fs::read(fixture_path(relative)).expect("fixture should be readable")
}

pub(crate) struct TempFixture {
    path: PathBuf,
    base: PathBuf,
}

impl TempFixture {
    pub(crate) fn new(relative: impl AsRef<Path>) -> Self {
        let relative = relative.as_ref();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("mdq-test-fixture-{unique}"));
        fs::create_dir_all(&base).expect("temp dir should be created");

        let file_name = relative
            .file_name()
            .expect("fixture path should include a file name");
        let path = base.join(file_name);
        fs::write(&path, fixture_bytes(relative)).expect("fixture should be copied");

        Self { path, base }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

#[test]
fn fixture_corpus_includes_required_contract_coverage() {
    let required = [
        "headings/regular-nested.md",
        "headings/irregular-levels.md",
        "headings/setext-preamble-empty.md",
        "blocks/first-block-kinds.md",
        "edge/find-cases.md",
        "edge/get-truncation.md",
        "edge/data-uri-image.md",
        "edge/zero-match.md",
    ];

    for relative in required {
        assert!(
            fixture_path(relative).is_file(),
            "missing fixture corpus file: {relative}"
        );
    }
}
