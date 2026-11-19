//! Error types for rustomize-epub
//!
//! This module provides comprehensive error handling for EPUB processing operations,
//! including detailed error context and user-friendly suggestions for troubleshooting.

use thiserror::Error;

/// Result type for rustomize-epub operations
pub type Result<T> = std::result::Result<T, ProcessingError>;

/// Error severity levels for prioritization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Minor issue, processing can continue
    Warning,
    /// Moderate issue, some features may not work
    Error,
    /// Critical issue, processing cannot continue
    Fatal,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "Warning"),
            ErrorSeverity::Error => write!(f, "Error"),
            ErrorSeverity::Fatal => write!(f, "Fatal"),
        }
    }
}

/// Error type for EPUB processing operations
#[derive(Error, Debug, Clone)]
pub enum ProcessingErrorType {
    /// Image processing error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// HTML parsing error
    #[error("HTML parsing error: {0}")]
    HtmlParsing(String),

    /// Text processing error
    #[error("Text processing error: {0}")]
    TextProcessing(String),

    /// Metadata extraction error
    #[error("Metadata extraction error: {0}")]
    MetadataExtraction(String),

    /// ZIP/EPUB processing error
    #[error("EPUB processing error: {0}")]
    ZipProcessing(String),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParsing(String),

    /// File not found error
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Processing error with context about which item failed
#[derive(Error, Debug, Clone)]
pub struct ProcessingError {
    pub item: String,
    #[source]
    pub error_type: ProcessingErrorType,
    pub details: String,
    pub severity: ErrorSeverity,
    pub suggestion: Option<String>,
}

impl ProcessingError {
    /// Create a new processing error with default severity
    pub fn new(
        item: impl Into<String>,
        error_type: ProcessingErrorType,
        details: impl Into<String>,
    ) -> Self {
        let error_type_clone = error_type.clone();
        Self {
            item: item.into(),
            error_type,
            details: details.into(),
            severity: ErrorSeverity::Error,
            suggestion: Self::generate_suggestion(&error_type_clone),
        }
    }

    /// Create a new processing error with specified severity
    pub fn with_severity(
        item: impl Into<String>,
        error_type: ProcessingErrorType,
        details: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        let error_type_clone = error_type.clone();
        Self {
            item: item.into(),
            error_type,
            details: details.into(),
            severity,
            suggestion: Self::generate_suggestion(&error_type_clone),
        }
    }

    /// Generate user-friendly suggestions based on error type
    fn generate_suggestion(error_type: &ProcessingErrorType) -> Option<String> {
        match error_type {
            ProcessingErrorType::ZipProcessing(msg) => {
                if msg.contains("not found") || msg.contains("does not exist") {
                    Some("Check that the input file exists and is a valid EPUB file".to_string())
                } else if msg.contains("zip") || msg.contains("archive") {
                    Some("The EPUB file may be corrupted. Try opening it in a different reader to verify.".to_string())
                } else {
                    Some("Try extracting the EPUB manually to check its contents".to_string())
                }
            }
            ProcessingErrorType::ImageProcessing(msg) => {
                if msg.contains("unsupported") || msg.contains("format") {
                    Some("This image format may not be supported. Consider converting it manually first.".to_string())
                } else if msg.contains("corrupt") || msg.contains("invalid") {
                    Some("The image file may be corrupted. Try replacing it with a different version.".to_string())
                } else {
                    Some(
                        "Try processing without image optimization to isolate the issue"
                            .to_string(),
                    )
                }
            }
            ProcessingErrorType::FileNotFound(msg) => Some(format!(
                "The file '{}' was not found. Verify the file path and try again.",
                msg
            )),
            ProcessingErrorType::PermissionDenied(msg) => Some(format!(
                "Permission denied accessing '{}'. Check file permissions and try again.",
                msg
            )),
            _ => None,
        }
    }

    /// Check if this error is a warning (non-fatal)
    pub fn is_warning(&self) -> bool {
        self.severity == ErrorSeverity::Warning
    }

    /// Check if this error is fatal
    pub fn is_fatal(&self) -> bool {
        self.severity == ErrorSeverity::Fatal
    }
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error processing '{}': {} - {}",
            self.item, self.error_type, self.details
        )
    }
}

/// Collection of errors from processing
#[derive(Debug, Default)]
pub struct ProcessingErrorCollection {
    pub errors: Vec<ProcessingError>,
}

impl ProcessingErrorCollection {
    /// Create a new error collection
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Add an error to the collection
    pub fn add(&mut self, error: ProcessingError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any fatal errors
    pub fn has_fatal_errors(&self) -> bool {
        self.errors.iter().any(|e| e.is_fatal())
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.errors.iter().any(|e| e.is_warning())
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convert to Result, returning first fatal error or first error if none fatal
    pub fn into_result(self) -> Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else if let Some(fatal) = self.errors.iter().find(|e| e.is_fatal()) {
            Err(fatal.clone())
        } else {
            Err(self.errors[0].clone())
        }
    }

    /// Get error summary for display
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            return "No errors".to_string();
        }

        let total = self.errors.len();
        let warnings = self.errors.iter().filter(|e| e.is_warning()).count();
        let errors = self.errors.iter().filter(|e| !e.is_warning()).count();

        let mut parts = Vec::new();
        parts.push(format!("Total: {}", total));
        if warnings > 0 {
            parts.push(format!("Warnings: {}", warnings));
        }
        if errors > 0 {
            parts.push(format!("Errors: {}", errors));
        }

        parts.join(", ")
    }

    /// Get all errors formatted for display
    pub fn formatted_list(&self) -> String {
        self.errors
            .iter()
            .enumerate()
            .map(|(i, e)| format!("  {}. {} ({})", i + 1, e.details, e.severity))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_display() {
        assert_eq!(format!("{}", ErrorSeverity::Warning), "Warning");
        assert_eq!(format!("{}", ErrorSeverity::Error), "Error");
        assert_eq!(format!("{}", ErrorSeverity::Fatal), "Fatal");
    }

    #[test]
    fn test_processing_error_type_display() {
        let error = ProcessingErrorType::ImageProcessing("test".to_string());
        assert_eq!(format!("{}", error), "Image processing error: test");

        let error = ProcessingErrorType::FileNotFound("test.txt".to_string());
        assert_eq!(format!("{}", error), "File not found: test.txt");
    }

    #[test]
    fn test_processing_error_new() {
        let error = ProcessingError::new(
            "test_item",
            ProcessingErrorType::FileNotFound("test.txt".to_string()),
            "File not found details",
        );
        assert_eq!(error.item, "test_item");
        assert!(matches!(
            error.error_type,
            ProcessingErrorType::FileNotFound(_)
        ));
        assert_eq!(error.details, "File not found details");
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn test_processing_error_with_severity() {
        let error = ProcessingError::with_severity(
            "test_item",
            ProcessingErrorType::ImageProcessing("test".to_string()),
            "Image error details",
            ErrorSeverity::Warning,
        );
        assert_eq!(error.severity, ErrorSeverity::Warning);
        assert!(error.suggestion.is_some());
    }

    #[test]
    fn test_processing_error_suggestion_zip_not_found() {
        let error = ProcessingError::new(
            "test.epub",
            ProcessingErrorType::ZipProcessing("file not found".to_string()),
            "File not found",
        );
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.unwrap().contains("input file"));
    }

    #[test]
    fn test_processing_error_suggestion_zip_corrupted() {
        let error = ProcessingError::new(
            "test.epub",
            ProcessingErrorType::ZipProcessing("invalid zip archive".to_string()),
            "Invalid archive",
        );
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.unwrap().contains("corrupted"));
    }

    #[test]
    fn test_processing_error_suggestion_image_format() {
        let error = ProcessingError::new(
            "test.jpg",
            ProcessingErrorType::ImageProcessing("unsupported format".to_string()),
            "Unsupported format",
        );
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.unwrap().contains("format"));
    }

    #[test]
    fn test_processing_error_suggestion_file_not_found() {
        let error = ProcessingError::new(
            "missing.txt",
            ProcessingErrorType::FileNotFound("missing.txt".to_string()),
            "File missing",
        );
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.unwrap().contains("missing.txt"));
    }

    #[test]
    fn test_processing_error_suggestion_permission_denied() {
        let error = ProcessingError::new(
            "protected.txt",
            ProcessingErrorType::PermissionDenied("protected.txt".to_string()),
            "Permission denied",
        );
        assert!(error.suggestion.is_some());
        assert!(error.suggestion.unwrap().contains("protected.txt"));
    }

    #[test]
    fn test_processing_error_is_warning() {
        let error = ProcessingError::with_severity(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
            ErrorSeverity::Warning,
        );
        assert!(error.is_warning());
        assert!(!error.is_fatal());
    }

    #[test]
    fn test_processing_error_is_fatal() {
        let error = ProcessingError::with_severity(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
            ErrorSeverity::Fatal,
        );
        assert!(error.is_fatal());
        assert!(!error.is_warning());
    }

    #[test]
    fn test_processing_error_collection_new() {
        let collection = ProcessingErrorCollection::new();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_processing_error_collection_add() {
        let mut collection = ProcessingErrorCollection::new();
        let error = ProcessingError::new(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
        );
        collection.add(error);
        assert!(!collection.is_empty());
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn test_processing_error_collection_has_errors() {
        let mut collection = ProcessingErrorCollection::new();
        assert!(!collection.has_errors());
        collection.add(ProcessingError::new(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
        ));
        assert!(collection.has_errors());
    }

    #[test]
    fn test_processing_error_collection_has_warnings() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::with_severity(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
            ErrorSeverity::Warning,
        ));
        assert!(collection.has_warnings());
    }

    #[test]
    fn test_processing_error_collection_has_fatal_errors() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::with_severity(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
            ErrorSeverity::Fatal,
        ));
        assert!(collection.has_fatal_errors());
    }

    #[test]
    fn test_processing_error_collection_into_result_ok() {
        let collection = ProcessingErrorCollection::new();
        let result = collection.into_result();
        assert!(result.is_ok());
    }

    #[test]
    fn test_processing_error_collection_into_result_error() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::new(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
        ));
        let result = collection.into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_processing_error_collection_summary_empty() {
        let collection = ProcessingErrorCollection::new();
        assert_eq!(collection.summary(), "No errors");
    }

    #[test]
    fn test_processing_error_collection_summary_with_errors() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::new(
            "test1",
            ProcessingErrorType::Other("test1".to_string()),
            "details1",
        ));
        collection.add(ProcessingError::new(
            "test2",
            ProcessingErrorType::Other("test2".to_string()),
            "details2",
        ));
        let summary = collection.summary();
        assert!(summary.contains("Total: 2"));
    }

    #[test]
    fn test_processing_error_collection_summary_with_warnings() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::with_severity(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
            ErrorSeverity::Warning,
        ));
        collection.add(ProcessingError::new(
            "test2",
            ProcessingErrorType::Other("test2".to_string()),
            "details2",
        ));
        let summary = collection.summary();
        assert!(summary.contains("Warnings: 1"));
        assert!(summary.contains("Errors: 1"));
    }

    #[test]
    fn test_processing_error_formatted_list() {
        let mut collection = ProcessingErrorCollection::new();
        collection.add(ProcessingError::new(
            "test1",
            ProcessingErrorType::Other("test1".to_string()),
            "details1",
        ));
        let formatted = collection.formatted_list();
        assert!(formatted.contains("details1"));
        assert!(formatted.contains("Error"));
    }
}
