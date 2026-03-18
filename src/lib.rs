use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INTERNAL_ERROR: i32 = 1;
pub const EXIT_USAGE_ERROR: i32 = 2;
pub const EXIT_FILE_ERROR: i32 = 3;
pub const EXIT_SECTION_NOT_FOUND: i32 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
struct LineRange {
    content_start: usize,
    content_end: usize,
    full_end: usize,
}

#[derive(Debug)]
pub enum InputError {
    FileRead { path: PathBuf, source: io::Error },
    Decode { path: PathBuf },
}

impl InputError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileRead { .. } => "file_read_error",
            Self::Decode { .. } => "decode_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::FileRead { path, source } => {
                format!("Failed to read file '{}': {}", path.display(), source)
            }
            Self::Decode { path } => {
                format!("Failed to decode file '{}' as UTF-8", path.display())
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Document {
    source: String,
    lines: Vec<LineRange>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeadingKind {
    Atx,
    Setext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Heading {
    pub kind: HeadingKind,
    pub level: u8,
    pub title: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl Document {
    pub fn read(path: impl Into<PathBuf>) -> Result<Self, InputError> {
        let path = path.into();
        let bytes = fs::read(&path).map_err(|source| InputError::FileRead {
            path: path.clone(),
            source,
        })?;

        Self::from_bytes(path, &bytes)
    }

    pub fn from_bytes(path: impl Into<PathBuf>, bytes: &[u8]) -> Result<Self, InputError> {
        let path = path.into();
        let source =
            String::from_utf8(bytes.to_vec()).map_err(|_| InputError::Decode { path: path.clone() })?;
        let source = source.strip_prefix('\u{feff}').unwrap_or(&source).to_owned();

        Ok(Self {
            lines: index_lines(&source),
            source,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, line_number: usize) -> Option<&str> {
        let range = self.line_range(line_number)?;
        Some(&self.source[range.content_start..range.content_end])
    }

    pub fn slice_lines(&self, start_line: usize, end_line: usize) -> Option<&str> {
        if start_line == 0 || start_line > end_line {
            return None;
        }

        let start = self.line_range(start_line)?.content_start;
        let end = self.line_range(end_line)?.full_end;
        Some(&self.source[start..end])
    }

    pub fn line_start_offset(&self, line_number: usize) -> Option<usize> {
        Some(self.line_range(line_number)?.content_start)
    }

    pub fn line_end_offset(&self, line_number: usize) -> Option<usize> {
        Some(self.line_range(line_number)?.content_end)
    }

    pub fn headings(&self) -> Vec<Heading> {
        parse_headings(self)
    }

    fn line_range(&self, line_number: usize) -> Option<&LineRange> {
        self.lines.get(line_number.checked_sub(1)?)
    }
}

fn index_lines(source: &str) -> Vec<LineRange> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut line_start = 0;

    for (idx, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            let content_end = if idx > line_start && bytes[idx - 1] == b'\r' {
                idx - 1
            } else {
                idx
            };

            lines.push(LineRange {
                content_start: line_start,
                content_end,
                full_end: idx + 1,
            });
            line_start = idx + 1;
        }
    }

    if line_start < bytes.len() {
        lines.push(LineRange {
            content_start: line_start,
            content_end: bytes.len(),
            full_end: bytes.len(),
        });
    }

    lines
}

fn parse_headings(document: &Document) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut line_number = 1;

    while line_number <= document.line_count() {
        let line = document
            .line(line_number)
            .expect("line_number should always be valid while scanning headings");

        if let Some((level, title)) = parse_atx_heading(line) {
            headings.push(Heading {
                kind: HeadingKind::Atx,
                level,
                title,
                start_line: line_number,
                end_line: line_number,
                start_offset: document
                    .line_start_offset(line_number)
                    .expect("heading line should have a start offset"),
                end_offset: document
                    .line_end_offset(line_number)
                    .expect("heading line should have an end offset"),
            });
            line_number += 1;
            continue;
        }

        if line_number < document.line_count() {
            let next_line = document
                .line(line_number + 1)
                .expect("next line should exist while checking setext headings");
            if let Some(level) = parse_setext_underline(next_line) {
                if is_setext_heading_text(line) {
                    headings.push(Heading {
                        kind: HeadingKind::Setext,
                        level,
                        title: line.trim().to_owned(),
                        start_line: line_number,
                        end_line: line_number + 1,
                        start_offset: document
                            .line_start_offset(line_number)
                            .expect("setext heading should have a start offset"),
                        end_offset: document
                            .line_end_offset(line_number + 1)
                            .expect("setext heading should have an end offset"),
                    });
                    line_number += 2;
                    continue;
                }
            }
        }

        line_number += 1;
    }

    headings
}

fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }

    let trimmed = &line[indent..];
    let marker_len = trimmed.chars().take_while(|ch| *ch == '#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }

    let rest = &trimmed[marker_len..];
    if !rest.is_empty() && !matches!(rest.as_bytes()[0], b' ' | b'\t') {
        return None;
    }

    let content = rest.trim();
    let title = strip_atx_closing_sequence(content);
    Some((marker_len as u8, title.to_owned()))
}

fn strip_atx_closing_sequence(content: &str) -> &str {
    let trimmed = content.trim_end();
    let hash_count = trimmed
        .chars()
        .rev()
        .take_while(|ch| *ch == '#')
        .count();

    if hash_count == 0 {
        return trimmed;
    }

    let without_hashes = &trimmed[..trimmed.len() - hash_count];
    if without_hashes.ends_with([' ', '\t']) {
        without_hashes.trim_end()
    } else {
        trimmed
    }
}

fn parse_setext_underline(line: &str) -> Option<u8> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.chars().all(|ch| ch == '=') {
        return Some(1);
    }

    if trimmed.chars().all(|ch| ch == '-') {
        return Some(2);
    }

    None
}

fn is_setext_heading_text(line: &str) -> bool {
    if line.trim().is_empty() {
        return false;
    }

    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return false;
    }

    let trimmed = line.trim_start();
    if trimmed.starts_with('>') {
        return false;
    }
    if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
        return false;
    }
    if parse_atx_heading(line).is_some() {
        return false;
    }
    if matches_list_marker(trimmed) {
        return false;
    }

    true
}

fn matches_list_marker(line: &str) -> bool {
    if line.starts_with("- ") || line.starts_with("* ") || line.starts_with("+ ") {
        return true;
    }

    let mut digits = 0usize;
    for ch in line.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
            continue;
        }
        return digits > 0 && ch == '.' && line[digits + 1..].starts_with(' ');
    }

    false
}

#[derive(Debug, Parser)]
#[command(
    name = "mdq",
    version,
    about = "Navigate Markdown files through tree, get, and find commands.",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn validated(self) -> Result<Self, UsageError> {
        match &self.command {
            Commands::Get(args) => args.validate()?,
            Commands::Tree(args) => args.validate()?,
            Commands::Find(args) => args.validate()?,
        }

        Ok(self)
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print an annotated section outline for a Markdown file.
    Tree(TreeArgs),
    /// Extract the exact Markdown slice for a section id.
    Get(GetArgs),
    /// Search matching lines and map them to section ids.
    Find(FindArgs),
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum TreeFormat {
    AnnotatedMd,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum GetFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum FindFormat {
    Text,
    Json,
}

#[derive(Debug, Args)]
pub struct TreeArgs {
    /// Markdown file to inspect.
    pub file: PathBuf,

    /// Output format.
    #[arg(long, value_enum, default_value_t = TreeFormat::AnnotatedMd)]
    pub format: TreeFormat,

    /// Maximum heading depth to print.
    #[arg(long, value_name = "N")]
    pub max_depth: Option<u8>,

    /// Suppress summary lines.
    #[arg(long)]
    pub no_summary: bool,
}

impl TreeArgs {
    fn validate(&self) -> Result<(), UsageError> {
        if matches!(self.max_depth, Some(0)) {
            return Err(UsageError::new("--max-depth must be greater than 0"));
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct GetArgs {
    /// Markdown file to inspect.
    pub file: PathBuf,

    /// Section id to extract.
    #[arg(long, value_name = "SECTION_ID")]
    pub id: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = GetFormat::Text)]
    pub format: GetFormat,

    /// Limit emitted lines after extraction.
    #[arg(long, value_name = "N")]
    pub max_lines: Option<usize>,

    /// Prefix text output with original file line numbers.
    #[arg(long)]
    pub with_line_numbers: bool,
}

impl GetArgs {
    fn validate(&self) -> Result<(), UsageError> {
        if matches!(self.max_lines, Some(0)) {
            return Err(UsageError::new("--max-lines must be greater than 0"));
        }

        if self.with_line_numbers && self.format != GetFormat::Text {
            return Err(UsageError::new(
                "--with-line-numbers can only be used with --format text",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct FindArgs {
    /// Markdown file to inspect.
    pub file: PathBuf,

    /// Literal or regex query to search for.
    pub query: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = FindFormat::Text)]
    pub format: FindFormat,

    /// Interpret query as a regular expression.
    #[arg(long)]
    pub regex: bool,

    /// Match with ASCII case sensitivity.
    #[arg(long)]
    pub case_sensitive: bool,

    /// Stop after emitting N matching lines.
    #[arg(long, value_name = "N", default_value_t = 200)]
    pub max_matches: usize,
}

impl FindArgs {
    fn validate(&self) -> Result<(), UsageError> {
        if self.max_matches == 0 {
            return Err(UsageError::new("--max-matches must be greater than 0"));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UsageError {
    message: String,
}

impl UsageError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => match cli.validated() {
            Ok(cli) => execute(cli),
            Err(err) => {
                let mut command = Cli::command();
                command
                    .error(ErrorKind::ValueValidation, err.message())
                    .print()
                    .expect("failed to print usage error");
                EXIT_USAGE_ERROR
            }
        },
        Err(err) => {
            let kind = err.kind();
            err.print().expect("failed to print clap output");
            if matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                EXIT_SUCCESS
            } else {
                EXIT_USAGE_ERROR
            }
        }
    }
}

fn execute(cli: Cli) -> i32 {
    match cli.command {
        Commands::Tree(_) | Commands::Get(_) | Commands::Find(_) => EXIT_SUCCESS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn parse_ok(args: &[&str]) -> Cli {
        Cli::try_parse_from(args)
            .expect("args should parse")
            .validated()
            .expect("args should validate")
    }

    #[test]
    fn tree_command_uses_documented_defaults() {
        let cli = parse_ok(&["mdq", "tree", "README.md"]);

        match cli.command {
            Commands::Tree(args) => {
                assert_eq!(args.file, PathBuf::from("README.md"));
                assert_eq!(args.format, TreeFormat::AnnotatedMd);
                assert_eq!(args.max_depth, None);
                assert!(!args.no_summary);
            }
            _ => panic!("expected tree command"),
        }
    }

    #[test]
    fn get_command_accepts_documented_flags() {
        let cli = parse_ok(&[
            "mdq",
            "get",
            "README.md",
            "--id",
            "s1-2",
            "--max-lines",
            "10",
            "--with-line-numbers",
        ]);

        match cli.command {
            Commands::Get(args) => {
                assert_eq!(args.file, PathBuf::from("README.md"));
                assert_eq!(args.id, "s1-2");
                assert_eq!(args.format, GetFormat::Text);
                assert_eq!(args.max_lines, Some(10));
                assert!(args.with_line_numbers);
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn get_command_rejects_line_numbers_for_json() {
        let cli = Cli::try_parse_from([
            "mdq",
            "get",
            "README.md",
            "--id",
            "s1",
            "--format",
            "json",
            "--with-line-numbers",
        ])
        .expect("args should parse");

        let err = cli.validated().expect_err("validation should fail");
        assert_eq!(
            err.message(),
            "--with-line-numbers can only be used with --format text"
        );
    }

    #[test]
    fn numeric_limits_reject_zero() {
        let cli = Cli::try_parse_from(["mdq", "tree", "README.md", "--max-depth", "0"])
            .expect("args should parse");
        let err = cli.validated().expect_err("validation should fail");
        assert_eq!(err.message(), "--max-depth must be greater than 0");

        let cli = Cli::try_parse_from([
            "mdq",
            "get",
            "README.md",
            "--id",
            "s1",
            "--max-lines",
            "0",
        ])
        .expect("args should parse");
        let err = cli.validated().expect_err("validation should fail");
        assert_eq!(err.message(), "--max-lines must be greater than 0");

        let cli = Cli::try_parse_from([
            "mdq",
            "find",
            "README.md",
            "install",
            "--max-matches",
            "0",
        ])
        .expect("args should parse");
        let err = cli.validated().expect_err("validation should fail");
        assert_eq!(err.message(), "--max-matches must be greater than 0");
    }

    #[test]
    fn find_command_accepts_search_flags() {
        let cli = parse_ok(&[
            "mdq",
            "find",
            "README.md",
            "install",
            "--format",
            "json",
            "--regex",
            "--case-sensitive",
            "--max-matches",
            "5",
        ]);

        match cli.command {
            Commands::Find(args) => {
                assert_eq!(args.file, PathBuf::from("README.md"));
                assert_eq!(args.query, "install");
                assert_eq!(args.format, FindFormat::Json);
                assert!(args.regex);
                assert!(args.case_sensitive);
                assert_eq!(args.max_matches, 5);
            }
            _ => panic!("expected find command"),
        }
    }

    #[test]
    fn find_command_uses_documented_default_max_matches() {
        let cli = parse_ok(&["mdq", "find", "README.md", "install"]);

        match cli.command {
            Commands::Find(args) => {
                assert_eq!(args.file, PathBuf::from("README.md"));
                assert_eq!(args.query, "install");
                assert_eq!(args.format, FindFormat::Text);
                assert!(!args.regex);
                assert!(!args.case_sensitive);
                assert_eq!(args.max_matches, 200);
            }
            _ => panic!("expected find command"),
        }
    }

    #[test]
    fn missing_get_id_is_a_parse_error() {
        let result = Cli::try_parse_from(["mdq", "get", "README.md"]);
        assert!(result.is_err());
    }

    #[test]
    fn document_strips_bom_and_indexes_lf_lines() {
        let doc = Document::from_bytes("fixture.md", b"\xEF\xBB\xBF# Title\nline 2\n")
            .expect("fixture should decode");

        assert_eq!(doc.source(), "# Title\nline 2\n");
        assert_eq!(doc.line_count(), 2);
        assert_eq!(doc.line(1), Some("# Title"));
        assert_eq!(doc.line(2), Some("line 2"));
        assert_eq!(doc.slice_lines(1, 2), Some("# Title\nline 2\n"));
        assert_eq!(doc.line_start_offset(1), Some(0));
        assert_eq!(doc.line_end_offset(1), Some(7));
        assert_eq!(doc.line_start_offset(2), Some(8));
        assert_eq!(doc.line_end_offset(2), Some(14));
    }

    #[test]
    fn document_treats_crlf_as_single_line_break() {
        let doc = Document::from_bytes("fixture.md", b"# Title\r\nline 2\r\nlast line")
            .expect("fixture should decode");

        assert_eq!(doc.line_count(), 3);
        assert_eq!(doc.line(1), Some("# Title"));
        assert_eq!(doc.line(2), Some("line 2"));
        assert_eq!(doc.line(3), Some("last line"));
        assert_eq!(doc.slice_lines(1, 2), Some("# Title\r\nline 2\r\n"));
        assert_eq!(doc.line_start_offset(2), Some(9));
        assert_eq!(doc.line_end_offset(2), Some(15));
    }

    #[test]
    fn document_reports_decode_errors() {
        let err = Document::from_bytes("fixture.md", &[0xff, 0xfe]).expect_err("should fail decode");

        assert_eq!(err.code(), "decode_error");
        assert_eq!(err.message(), "Failed to decode file 'fixture.md' as UTF-8");
    }

    #[test]
    fn document_reads_files_and_reports_read_errors() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("mdq-input-test-{unique}"));
        fs::create_dir_all(&base).expect("temp dir should be created");

        let file_path = base.join("fixture.md");
        fs::write(&file_path, "# Title\r\nBody\n").expect("fixture should be written");

        let doc = Document::read(&file_path).expect("fixture should load");
        assert_eq!(doc.line_count(), 2);
        assert_eq!(doc.slice_lines(1, 2), Some("# Title\r\nBody\n"));

        let missing = base.join("missing.md");
        let err = Document::read(&missing).expect_err("missing file should fail");
        assert_eq!(err.code(), "file_read_error");
        assert!(
            err.message()
                .starts_with(&format!("Failed to read file '{}':", missing.display()))
        );

        fs::remove_dir_all(&base).expect("temp dir should be removed");
    }

    #[test]
    fn document_parses_atx_headings_with_source_positions() {
        let doc = Document::from_bytes(
            "fixture.md",
            b"# Title\nIntro\n### Deep Dive ###\nBody\n####### not a heading\n",
        )
        .expect("fixture should decode");

        assert_eq!(
            doc.headings(),
            vec![
                Heading {
                    kind: HeadingKind::Atx,
                    level: 1,
                    title: "Title".to_owned(),
                    start_line: 1,
                    end_line: 1,
                    start_offset: 0,
                    end_offset: 7,
                },
                Heading {
                    kind: HeadingKind::Atx,
                    level: 3,
                    title: "Deep Dive".to_owned(),
                    start_line: 3,
                    end_line: 3,
                    start_offset: 14,
                    end_offset: 31,
                },
            ]
        );
    }

    #[test]
    fn document_parses_setext_headings_with_source_positions() {
        let doc = Document::from_bytes(
            "fixture.md",
            b"Title\n=====\n\nSubtitle\n-----\nParagraph\n",
        )
        .expect("fixture should decode");

        assert_eq!(
            doc.headings(),
            vec![
                Heading {
                    kind: HeadingKind::Setext,
                    level: 1,
                    title: "Title".to_owned(),
                    start_line: 1,
                    end_line: 2,
                    start_offset: 0,
                    end_offset: 11,
                },
                Heading {
                    kind: HeadingKind::Setext,
                    level: 2,
                    title: "Subtitle".to_owned(),
                    start_line: 4,
                    end_line: 5,
                    start_offset: 13,
                    end_offset: 27,
                },
            ]
        );
    }

    #[test]
    fn document_rejects_common_non_heading_setext_candidates() {
        let doc = Document::from_bytes(
            "fixture.md",
            b"> quoted\n-----\n- list item\n-----\n    indented\n-----\n",
        )
        .expect("fixture should decode");

        assert!(doc.headings().is_empty());
    }
}
