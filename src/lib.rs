mod cli;
mod document;
mod error;
mod section;

pub use crate::cli::{
    Cli, Commands, FindArgs, FindFormat, GetArgs, GetFormat, TreeArgs, TreeFormat,
};
pub use crate::document::{Document, Heading, HeadingKind};
pub use crate::error::{
    AppError, ErrorFormat, InputError, UsageError, EXIT_FILE_ERROR, EXIT_INTERNAL_ERROR,
    EXIT_SECTION_NOT_FOUND, EXIT_SUCCESS, EXIT_USAGE_ERROR,
};
pub use crate::section::{Section, SectionIndex, ROOT_SECTION_ID};

use std::ffi::OsString;
use std::io::{self, Write};

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use regex::RegexBuilder;

use crate::error::escape_json_string;

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    run_with_io(args, &mut io::stdout(), &mut io::stderr())
}

pub(crate) fn run_with_io<I, T, O, E>(args: I, stdout: &mut O, stderr: &mut E) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    O: Write,
    E: Write,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => match cli.validated() {
            Ok(cli) => {
                let error_format = cli_error_format(&cli);
                match execute(cli, stdout) {
                    Ok(()) => EXIT_SUCCESS,
                    Err(err) => handle_runtime_error(err, error_format, stdout, stderr),
                }
            }
            Err(err) => handle_usage_validation_error(err, stderr),
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

fn execute<W: Write>(cli: Cli, stdout: &mut W) -> Result<(), AppError> {
    match cli.command {
        Commands::Tree(args) => {
            let _document = Document::read(&args.file)?;
            Ok(())
        }
        Commands::Get(args) => {
            let document = Document::read(&args.file)?;
            if document.section_index().by_id(&args.id).is_none() {
                return Err(AppError::SectionNotFound { id: args.id });
            }
            Ok(())
        }
        Commands::Find(args) => {
            let document = Document::read(&args.file)?;
            let matches = find_matches(&document, &args)?;

            match args.format {
                FindFormat::Text => render_find_text(stdout, &matches)?,
                FindFormat::Json => render_find_json(stdout, &args, &matches)?,
            }

            Ok(())
        }
    }
}

fn cli_error_format(cli: &Cli) -> ErrorFormat {
    match &cli.command {
        Commands::Tree(args) => match args.format {
            TreeFormat::AnnotatedMd => ErrorFormat::Text,
            TreeFormat::Json => ErrorFormat::Json,
        },
        Commands::Get(args) => match args.format {
            GetFormat::Text => ErrorFormat::Text,
            GetFormat::Json => ErrorFormat::Json,
        },
        Commands::Find(args) => match args.format {
            FindFormat::Text => ErrorFormat::Text,
            FindFormat::Json => ErrorFormat::Json,
        },
    }
}

fn handle_usage_validation_error<E: Write>(err: UsageError, stderr: &mut E) -> i32 {
    let mut command = Cli::command();
    write!(
        stderr,
        "{}",
        command
            .error(ErrorKind::ValueValidation, err.message())
            .render()
    )
    .expect("failed to print usage error");
    EXIT_USAGE_ERROR
}

fn handle_runtime_error<O: Write, E: Write>(
    err: AppError,
    format: ErrorFormat,
    stdout: &mut O,
    stderr: &mut E,
) -> i32 {
    match format {
        ErrorFormat::Text => {
            writeln!(stderr, "{}", err.message()).expect("failed to print runtime error");
        }
        ErrorFormat::Json => {
            writeln!(stdout, "{}", err.render_json()).expect("failed to print JSON error");
        }
    }

    err.exit_code()
}

#[derive(Debug, Eq, PartialEq)]
struct FindMatch {
    line: usize,
    section_id: String,
    text: String,
}

fn find_matches(document: &Document, args: &FindArgs) -> Result<Vec<FindMatch>, AppError> {
    let matcher = LineMatcher::new(&args.query, args.regex, args.case_sensitive)?;
    let sections = document.section_index();
    let mut matches = Vec::new();

    for line_number in 1..=document.line_count() {
        let line = document
            .line(line_number)
            .expect("line number should remain valid while searching");

        if !matcher.is_match(line) {
            continue;
        }

        let section_id = sections
            .section_for_line(line_number)
            .map(|section| section.id.clone())
            .unwrap_or_else(|| ROOT_SECTION_ID.to_owned());

        matches.push(FindMatch {
            line: line_number,
            section_id,
            text: line.to_owned(),
        });

        if matches.len() >= args.max_matches {
            break;
        }
    }

    Ok(matches)
}

fn render_find_text<W: Write>(stdout: &mut W, matches: &[FindMatch]) -> Result<(), AppError> {
    for item in matches {
        let section_id = if item.section_id == ROOT_SECTION_ID {
            "-"
        } else {
            item.section_id.as_str()
        };
        writeln!(stdout, "L{} [{}] {}", item.line, section_id, item.text).map_err(io_error)?;
    }

    Ok(())
}

fn render_find_json<W: Write>(
    stdout: &mut W,
    args: &FindArgs,
    matches: &[FindMatch],
) -> Result<(), AppError> {
    write!(
        stdout,
        "{{\"command\":\"find\",\"file\":\"{}\",\"format\":\"json\",\"query\":\"{}\",\"regex\":{},\"case_sensitive\":{},\"matches\":[",
        escape_json_string(&args.file.display().to_string()),
        escape_json_string(&args.query),
        args.regex,
        args.case_sensitive,
    )
    .map_err(io_error)?;

    for (index, item) in matches.iter().enumerate() {
        if index > 0 {
            write!(stdout, ",").map_err(io_error)?;
        }

        write!(
            stdout,
            "{{\"line\":{},\"section_id\":\"{}\",\"text\":\"{}\"}}",
            item.line,
            escape_json_string(&item.section_id),
            escape_json_string(&item.text),
        )
        .map_err(io_error)?;
    }

    writeln!(stdout, "]}}").map_err(io_error)?;
    Ok(())
}

struct LineMatcher {
    query: String,
    case_sensitive: bool,
    regex: Option<regex::Regex>,
}

impl LineMatcher {
    fn new(query: &str, regex: bool, case_sensitive: bool) -> Result<Self, AppError> {
        let regex = if regex {
            Some(
                RegexBuilder::new(query)
                    .case_insensitive(!case_sensitive)
                    .build()
                    .map_err(|err| AppError::Usage(UsageError::new(err.to_string())))?,
            )
        } else {
            None
        };

        Ok(Self {
            query: query.to_owned(),
            case_sensitive,
            regex,
        })
    }

    fn is_match(&self, line: &str) -> bool {
        if let Some(regex) = &self.regex {
            return regex.is_match(line);
        }

        if self.case_sensitive {
            line.contains(&self.query)
        } else {
            line.to_ascii_lowercase()
                .contains(&self.query.to_ascii_lowercase())
        }
    }
}

fn io_error(err: io::Error) -> AppError {
    AppError::Internal {
        message: format!("Failed to write output: {err}"),
    }
}

#[cfg(test)]
mod tests;
