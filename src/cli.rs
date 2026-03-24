use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::error::UsageError;

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
