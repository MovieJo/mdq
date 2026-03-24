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
pub use crate::section::{Section, SectionIndex, SummaryBlock, SummaryKind, ROOT_SECTION_ID};

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
            let document = Document::read(&args.file)?;
            render_tree_annotated_md(stdout, &document, &args)?;
            Ok(())
        }
        Commands::Get(args) => {
            let document = Document::read(&args.file)?;
            let sections = document.section_index();
            let section = sections
                .by_id(&args.id)
                .ok_or_else(|| AppError::SectionNotFound {
                    id: args.id.clone(),
                })?;
            let content = get_content(
                &document,
                section.start_line,
                section.end_line,
                args.max_lines,
            )
            .expect("validated section range should always produce content");

            match args.format {
                GetFormat::Text => {
                    render_get_text(stdout, &document, &content, args.with_line_numbers)?
                }
                GetFormat::Json => render_get_json(
                    stdout,
                    &args,
                    section.start_line,
                    section.end_line,
                    &content,
                )?,
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

fn render_tree_annotated_md<W: Write>(
    stdout: &mut W,
    document: &Document,
    args: &TreeArgs,
) -> Result<(), AppError> {
    let sections = document.section_index();
    let mut printed = false;

    for section in sections.sections() {
        if args
            .max_depth
            .is_some_and(|max_depth| section.level > max_depth)
        {
            continue;
        }

        if printed {
            writeln!(stdout).map_err(io_error)?;
        }

        writeln!(
            stdout,
            "{} [{} L{}-L{}] {}",
            "#".repeat(section.level.into()),
            section.id,
            section.start_line,
            section.end_line,
            section.title,
        )
        .map_err(io_error)?;

        if !args.no_summary {
            if let Some(summary) = section.summary_block(document) {
                writeln!(stdout, "{}: {}", summary.tag(), summary.payload()).map_err(io_error)?;
            }
        }

        printed = true;
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct GetContent {
    start_line: usize,
    end_line: usize,
    truncated: bool,
    content: String,
}

fn get_content(
    document: &Document,
    start_line: usize,
    end_line: usize,
    max_lines: Option<usize>,
) -> Option<GetContent> {
    let extracted_end = match max_lines {
        Some(limit) => start_line + limit.saturating_sub(1),
        None => end_line,
    }
    .min(end_line);
    let content = document.slice_lines(start_line, extracted_end)?;

    Some(GetContent {
        start_line,
        end_line,
        truncated: extracted_end < end_line,
        content: content.to_owned(),
    })
}

fn render_get_text<W: Write>(
    stdout: &mut W,
    document: &Document,
    content: &GetContent,
    with_line_numbers: bool,
) -> Result<(), AppError> {
    if !with_line_numbers {
        write!(stdout, "{}", content.content).map_err(io_error)?;
        return Ok(());
    }

    for line_number in content.start_line..=content.start_line + content_line_count(content) - 1 {
        let line = document
            .line(line_number)
            .expect("line number should remain valid while rendering section");
        let slice = document
            .slice_lines(line_number, line_number)
            .expect("line number should remain valid while rendering section");

        write!(stdout, "L{line_number}: {line}").map_err(io_error)?;
        if slice.ends_with('\n') {
            writeln!(stdout).map_err(io_error)?;
        }
    }

    Ok(())
}

fn render_get_json<W: Write>(
    stdout: &mut W,
    args: &GetArgs,
    start_line: usize,
    end_line: usize,
    content: &GetContent,
) -> Result<(), AppError> {
    writeln!(
        stdout,
        "{{\"command\":\"get\",\"file\":\"{}\",\"format\":\"json\",\"id\":\"{}\",\"start_line\":{},\"end_line\":{},\"truncated\":{},\"content\":\"{}\"}}",
        escape_json_string(&args.file.display().to_string()),
        escape_json_string(&args.id),
        start_line,
        end_line,
        content.truncated,
        escape_json_string(&content.content),
    )
    .map_err(io_error)?;

    Ok(())
}

fn content_line_count(content: &GetContent) -> usize {
    content.content.lines().count().max(1)
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
