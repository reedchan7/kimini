use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    LocalFile(String),
    Transport(String),
    InvalidResponse(String),
    Daemon { code: i64, message: String },
    MissingData,
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalFile(message) => write!(formatter, "Local file error: {message}"),
            Self::Transport(message) => {
                write!(formatter, "Kimi daemon connection failed: {message}")
            }
            Self::InvalidResponse(message) => {
                write!(formatter, "Invalid Kimi daemon response: {message}")
            }
            Self::Daemon { code, message } => {
                write!(formatter, "Kimi daemon error {code}: {message}")
            }
            Self::MissingData => formatter.write_str("Kimi daemon returned no data"),
        }
    }
}

impl std::error::Error for ApiError {}
