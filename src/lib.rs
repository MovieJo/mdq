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

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => match cli.validated() {
            Ok(cli) => {
                let error_format = cli_error_format(&cli);
                match execute(cli) {
                    Ok(()) => EXIT_SUCCESS,
                    Err(err) => handle_runtime_error(err, error_format),
                }
            }
            Err(err) => handle_usage_validation_error(err),
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

fn execute(cli: Cli) -> Result<(), AppError> {
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
            let _document = Document::read(&args.file)?;
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

fn handle_usage_validation_error(err: UsageError) -> i32 {
    let mut command = Cli::command();
    command
        .error(ErrorKind::ValueValidation, err.message())
        .print()
        .expect("failed to print usage error");
    EXIT_USAGE_ERROR
}

fn handle_runtime_error(err: AppError, format: ErrorFormat) -> i32 {
    match format {
        ErrorFormat::Text => {
            writeln!(io::stderr(), "{}", err.message()).expect("failed to print runtime error");
        }
        ErrorFormat::Json => {
            writeln!(io::stdout(), "{}", err.render_json()).expect("failed to print JSON error");
        }
    }

    err.exit_code()
}

#[cfg(test)]
mod tests;
