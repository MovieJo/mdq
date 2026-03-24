mod cli;
mod document;
mod error;
mod section;

pub use crate::cli::{
    Cli, Commands, FindArgs, FindFormat, GetArgs, GetFormat, TreeArgs, TreeFormat,
};
pub use crate::document::{Document, Heading, HeadingKind};
pub use crate::error::{
    InputError, UsageError, EXIT_FILE_ERROR, EXIT_INTERNAL_ERROR, EXIT_SECTION_NOT_FOUND,
    EXIT_SUCCESS, EXIT_USAGE_ERROR,
};
pub use crate::section::{Section, SectionIndex, ROOT_SECTION_ID};

use std::ffi::OsString;

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

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
mod tests;
