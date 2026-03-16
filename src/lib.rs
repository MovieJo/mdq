use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INTERNAL_ERROR: i32 = 1;
pub const EXIT_USAGE_ERROR: i32 = 2;
pub const EXIT_FILE_ERROR: i32 = 3;
pub const EXIT_SECTION_NOT_FOUND: i32 = 4;

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
}
