//! Standardized error handling for Tauri commands
//!
//! This module provides a consistent error structure for all IPC commands.

use serde::Serialize;

/// Error codes for categorizing command failures
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // General errors
    Unknown,
    InvalidInput,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    Timeout,

    // Service errors
    ServiceNotFound,
    ServiceStartFailed,
    ServiceStopFailed,
    ServiceNotRunning,
    ServiceAlreadyRunning,

    // Process errors
    ProcessNotFound,
    ProcessKillFailed,
    ProcessStartFailed,

    // Port errors
    PortInUse,
    PortScanFailed,
    InvalidPort,

    // File system errors
    FileNotFound,
    DirectoryNotFound,
    FileReadFailed,
    FileWriteFailed,
    PathInvalid,

    // Database errors
    DatabaseConnectionFailed,
    DatabaseQueryFailed,
    DatabaseNotFound,

    // Configuration errors
    ConfigInvalid,
    ConfigReadFailed,
    ConfigWriteFailed,

    // Project errors
    ProjectNotFound,
    ProjectDetectionFailed,
    ProjectAlreadyExists,

    // Domain/Hosts errors
    DomainInvalid,
    DomainConflict,
    HostsFileFailed,

    // Environment errors
    EnvFileFailed,
    EnvProfileNotFound,

    // Installer errors
    InstallFailed,
    DownloadFailed,
    ExtractionFailed,
    ComponentNotFound,
}

/// Structured error type for Tauri commands
/// This provides consistent error information to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Error code for programmatic handling
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details (e.g., stack trace, context)
    pub details: Option<String>,
    /// Whether this error is potentially recoverable by retrying
    pub is_retryable: bool,
}

impl CommandError {
    /// Create a new command error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            is_retryable: false,
        }
    }

    /// Create an error with details
    pub fn with_details(code: ErrorCode, message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Some(details.into()),
            is_retryable: false,
        }
    }

    /// Mark this error as retryable
    pub fn retryable(mut self) -> Self {
        self.is_retryable = true;
        self
    }

    /// Create a not found error
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::new(
            ErrorCode::NotFound,
            format!("{} '{}' not found", resource, id),
        )
    }

    /// Create a service not found error
    pub fn service_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::ServiceNotFound,
            format!("Service '{}' not found", id),
        )
    }

    /// Create a project not found error
    pub fn project_not_found(id: &str) -> Self {
        Self::new(
            ErrorCode::ProjectNotFound,
            format!("Project '{}' not found", id),
        )
    }

    /// Create a file not found error
    pub fn file_not_found(path: &str) -> Self {
        Self::new(
            ErrorCode::FileNotFound,
            format!("File not found: {}", path),
        )
    }

    /// Create an unknown error from any error type
    pub fn from_error<E: std::fmt::Display>(error: E) -> Self {
        Self::new(ErrorCode::Unknown, error.to_string())
    }

    /// Create an unknown error with context
    pub fn from_error_with_context<E: std::fmt::Display>(error: E, context: &str) -> Self {
        Self::with_details(
            ErrorCode::Unknown,
            format!("{}: {}", context, error),
            error.to_string(),
        )
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CommandError {}

// Conversion from common error types
impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        let code = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            std::io::ErrorKind::TimedOut => ErrorCode::Timeout,
            _ => ErrorCode::Unknown,
        };
        Self::with_details(code, format!("IO error: {}", err), err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(msg: String) -> Self {
        Self::new(ErrorCode::Unknown, msg)
    }
}

impl From<&str> for CommandError {
    fn from(msg: &str) -> Self {
        Self::new(ErrorCode::Unknown, msg.to_string())
    }
}

/// Result type alias for Tauri commands
pub type CommandResult<T> = Result<T, CommandError>;

/// Extension trait to easily convert errors to CommandError
pub trait IntoCommandError<T> {
    fn cmd_err(self, context: &str) -> CommandResult<T>;
    fn cmd_err_code(self, code: ErrorCode, context: &str) -> CommandResult<T>;
}

impl<T, E: std::fmt::Display> IntoCommandError<T> for Result<T, E> {
    fn cmd_err(self, context: &str) -> CommandResult<T> {
        self.map_err(|e| CommandError::from_error_with_context(e, context))
    }

    fn cmd_err_code(self, code: ErrorCode, context: &str) -> CommandResult<T> {
        self.map_err(|e| CommandError::with_details(code, context, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CommandError::new(ErrorCode::ServiceNotFound, "Apache not found");
        assert_eq!(err.message, "Apache not found");
        assert!(!err.is_retryable);
    }

    #[test]
    fn test_retryable_error() {
        let err = CommandError::new(ErrorCode::Timeout, "Request timed out").retryable();
        assert!(err.is_retryable);
    }

    #[test]
    fn test_helper_methods() {
        let err = CommandError::service_not_found("apache");
        assert!(err.message.contains("apache"));

        let err = CommandError::project_not_found("my-project");
        assert!(err.message.contains("my-project"));
    }
}
