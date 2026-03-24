use std::io;
use std::path::PathBuf;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INTERNAL_ERROR: i32 = 1;
pub const EXIT_USAGE_ERROR: i32 = 2;
pub const EXIT_FILE_ERROR: i32 = 3;
pub const EXIT_SECTION_NOT_FOUND: i32 = 4;

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
