//! Error handling

use std::{error::Error, fmt, io, string::FromUtf8Error, sync::Arc};

/// AppError object
#[derive(Default, Debug, Clone)]
pub struct AppError {
    /// Error message
    pub message: String,
    /// Error source
    pub source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the message and optionally the source
        if let Some(source) = &self.source {
            write!(f, "{} (caused by: {})", self.message, source)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl AppError {
    /// Create new AppError
    pub fn new(message: String) -> Self {
        Self {
            message,
            source: None,
        }
    }

    /// Create new Normal
    pub fn new_with_source<S: Into<String>>(
        message: S,
        source: Arc<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(source),
        }
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        Self::new(message.to_string())
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(error: FromUtf8Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(error: std::num::ParseFloatError) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Return a reference to the inner error, if present
        // Arc doesn’t allow direct coercion, so we must use `as_ref()` and a cast
        self.source.as_ref().map(|arc| {
            let err: &(dyn Error + 'static) = &**arc;
            err
        })
    }
}

impl<S, B> From<(S, B)> for AppError
where
    S: Into<String>,
    B: std::error::Error + Send + Sync + 'static,
{
    fn from(value: (S, B)) -> Self {
        // value.0 is the string, value.1 is the error
        AppError::new_with_source(value.0.into(), Arc::new(value.1))
    }
}

/// Error handler
#[derive(Debug, Default)]
pub struct ErrorManager {
    /// List of errors
    pub(crate) errors: Vec<AppError>,

    /// Check if it is open
    pub(crate) is_open: bool,

    /// Check if it was open
    pub(crate) was_open: bool,
}

impl ErrorManager {
    /// New Error manager
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Add an error
    pub fn add_error<E: Into<AppError>>(&mut self, error: E) {
        self.errors.push(error.into());
    }

    /// Errors Title
    pub fn title(&self) -> &'static str {
        "Error window"
    }
}
