//! Error handling

use std::{error::Error, fmt, io, string::FromUtf8Error, sync::Arc};

/// Type for error
#[derive(Default, Debug, Clone)]
enum ErrorType {
    /// Normal error
    #[default]
    Normal,
    /// Fake error
    Fake,
}

/// AppError object
#[derive(Default, Debug)]
pub struct AppError {
    /// Error message
    pub message: String,
    /// Error source
    pub source: Option<Arc<dyn std::error::Error + Send + Sync>>,
    /// Error type
    error_type: ErrorType,
}

impl Clone for AppError {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            source: self.source.clone(),
            error_type: self.error_type.clone(),
        }
    }
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
            error_type: ErrorType::Normal,
        }
    }

    /// Create new Normal
    pub fn new_with_source(source: Arc<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            message: source.to_string(),
            source: Some(source),
            error_type: ErrorType::Normal,
        }
    }

    /// Create fake error
    pub fn new_fake(message: String) -> Self {
        Self {
            message,
            source: None,
            error_type: ErrorType::Fake,
        }
    }

    /// Check if error is fake
    pub fn is_fake(&self) -> bool {
        matches!(self.error_type, ErrorType::Fake)
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
            error_type: ErrorType::Normal,
        }
    }
}

impl From<FromUtf8Error> for AppError {
    fn from(error: FromUtf8Error) -> Self {
        Self {
            message: error.to_string(),
            source: Some(Arc::new(error)),
            error_type: ErrorType::Normal,
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

/// Error handler
#[derive(Debug, Default)]
pub struct ErrorManager {
    /// List of errors
    pub errors: Vec<AppError>,

    /// Check if it is open
    pub is_open: bool,

    /// Check if it was open
    pub was_open: bool,
}

impl ErrorManager {
    /// New Error manager
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: AppError) {
        if error.is_fake() {
            return;
        }
        self.errors.push(error);
    }

    /// Handle an error
    pub fn handle_error<T>(&mut self, error: Result<T, impl Into<AppError>>) -> Option<T> {
        match error {
            Ok(value) => Some(value),
            Err(e) => {
                self.add_error(e.into());
                None
            }
        }
    }

    /// Errors Title
    pub fn title(&self) -> &'static str {
        "Error window"
    }
}
