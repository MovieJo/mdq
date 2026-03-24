use std::io;
use std::path::PathBuf;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INTERNAL_ERROR: i32 = 1;
pub const EXIT_USAGE_ERROR: i32 = 2;
pub const EXIT_FILE_ERROR: i32 = 3;
pub const EXIT_SECTION_NOT_FOUND: i32 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorFormat {
    Text,
    Json,
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

#[derive(Debug)]
pub enum AppError {
    Usage(UsageError),
    Input(InputError),
    SectionNotFound { id: String },
    Internal { message: String },
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage_error",
            Self::Input(err) => err.code(),
            Self::SectionNotFound { .. } => "section_not_found",
            Self::Internal { .. } => "internal_error",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Usage(err) => err.message().to_owned(),
            Self::Input(err) => err.message(),
            Self::SectionNotFound { id } => format!("Section id not found: {id}"),
            Self::Internal { message } => message.clone(),
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => EXIT_USAGE_ERROR,
            Self::Input(_) => EXIT_FILE_ERROR,
            Self::SectionNotFound { .. } => EXIT_SECTION_NOT_FOUND,
            Self::Internal { .. } => EXIT_INTERNAL_ERROR,
        }
    }

    pub fn render_json(&self) -> String {
        format!(
            "{{\"error\":{{\"code\":\"{}\",\"message\":\"{}\"}}}}",
            escape_json_string(self.code()),
            escape_json_string(&self.message())
        )
    }
}

impl From<InputError> for AppError {
    fn from(value: InputError) -> Self {
        Self::Input(value)
    }
}

impl From<UsageError> for AppError {
    fn from(value: UsageError) -> Self {
        Self::Usage(value)
    }
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0C}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch <= '\u{1F}' => {
                escaped.push_str(&format!("\\u{:04x}", ch as u32));
            }
            _ => escaped.push(ch),
        }
    }

    escaped
}
