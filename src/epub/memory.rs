//! EPUB memory operations - work with EPUB content in memory

use crate::{ProcessingError, Result};
use std::collections::HashMap;

/// In-memory EPUB structure - maps relative paths to file contents
pub type EpubMemory = HashMap<String, Vec<u8>>;

/// Get container.xml content from in-memory EPUB
pub fn get_container_content(files: &EpubMemory) -> Result<String> {
    let container_path = "META-INF/container.xml";

    files
        .get(container_path)
        .map(|bytes| {
            String::from_utf8(bytes.clone()).map_err(|e| {
                ProcessingError::new(
                    container_path.to_string(),
                    crate::error::ProcessingErrorType::XmlParsing(e.to_string()),
                    format!("Failed to parse container.xml as UTF-8: {}", e),
                )
            })
        })
        .ok_or_else(|| {
            ProcessingError::new(
                container_path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!(
                    "{} not found in EPUB",
                    container_path
                )),
                "container.xml not found in EPUB",
            )
        })?
}

/// Get file content from in-memory EPUB
pub fn get_file_content(files: &EpubMemory, path: &str) -> Result<String> {
    files
        .get(path)
        .map(|bytes| {
            String::from_utf8(bytes.clone()).map_err(|e| {
                ProcessingError::new(
                    path.to_string(),
                    crate::error::ProcessingErrorType::FileNotFound(format!(
                        "Failed to parse {} as UTF-8: {}",
                        path, e
                    )),
                    format!("Failed to parse file as UTF-8: {}", e),
                )
            })
        })
        .ok_or_else(|| {
            ProcessingError::new(
                path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!(
                    "File '{}' not found in EPUB",
                    path
                )),
                format!("File '{}' not found in EPUB", path),
            )
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_container_content_success() {
        let mut files = EpubMemory::new();
        files.insert(
            "META-INF/container.xml".to_string(),
            b"<?xml version=\"1.0\"?>".to_vec(),
        );

        let result = get_container_content(&files);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<?xml"));
    }

    #[test]
    fn test_get_container_content_not_found() {
        let files = EpubMemory::new();
        let result = get_container_content(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_container_content_invalid_utf8() {
        let mut files = EpubMemory::new();
        files.insert("META-INF/container.xml".to_string(), vec![0xff, 0xfe]);

        let result = get_container_content(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_content_success() {
        let mut files = EpubMemory::new();
        files.insert("test.txt".to_string(), b"Hello World".to_vec());

        let result = get_file_content(&files, "test.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World");
    }

    #[test]
    fn test_get_file_content_not_found() {
        let files = EpubMemory::new();
        let result = get_file_content(&files, "missing.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_file_content_invalid_utf8() {
        let mut files = EpubMemory::new();
        files.insert("test.txt".to_string(), vec![0xff, 0xfe]);

        let result = get_file_content(&files, "test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_epub_memory_type_alias() {
        let memory: EpubMemory = HashMap::new();
        assert!(memory.is_empty());
    }
}
