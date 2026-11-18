//! rustomize-epub - High-performance EPUB optimizer for e-ink devices
//!
//! This library provides comprehensive EPUB optimization functionality including:
//! - Image processing with 4-level grayscale quantization
//! - HTML/CSS cleaning and optimization
//! - Text cleanup (ligatures, quotes, mojibake, punctuation)
//! - Metadata extraction and cleaning
//! - TOC validation and regeneration
//!
//! # Example
//!
//! ```no_run
//! use rustomize_epub::{process_epub_path, ProcessingOptions};
//! use std::path::Path;
//!
//! let input = Path::new("input.epub");
//! let output = Path::new("output.epub");
//! let options = ProcessingOptions::default();
//!
//! let report = process_epub_path(input, output, &options).expect("Processing failed");
//! println!("Optimization complete: {}", report.summary());
//! ```

pub mod error;
pub mod types;

pub mod epub;
pub mod image;
pub mod html;
pub mod text;
pub mod metadata;
pub mod toc;
pub mod pipeline;

pub use error::{ProcessingError, ProcessingErrorType, ProcessingErrorCollection, Result};
pub use types::{ProcessingOptions, ProcessingReport, DeviceProfile, ImageOptions};
pub use pipeline::{process_epub_path, process_epub_memory};

// Re-export submodules for direct access to types
pub use text::cleaner::{TextCleanReport};
pub use html::cleaner::HtmlCleanReport;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_profile_exists() {
        let profile = DeviceProfile::x4();
        assert_eq!(profile.width, 480);
    }

    #[test]
    fn test_processing_options_exists() {
        let options = ProcessingOptions::default();
        assert!(options.generate_missing_cover);
    }

    #[test]
    fn test_processing_report_exists() {
        let report = ProcessingReport::default();
        assert!(report.success);
    }

    #[test]
    fn test_processing_error_exists() {
        let error = ProcessingError::new(
            "test",
            ProcessingErrorType::Other("test".to_string()),
            "details",
        );
        assert_eq!(error.item, "test");
    }

    #[test]
    fn test_error_collection_exists() {
        let collection = ProcessingErrorCollection::new();
        assert!(collection.is_empty());
    }

    #[test]
    fn test_result_type_exists() {
        let result: Result<()> = Ok(());
        assert!(result.is_ok());
    }
}
