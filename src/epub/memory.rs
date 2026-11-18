//! EPUB memory operations - work with EPUB content in memory

use std::collections::HashMap;
use std::io::Write;
use crate::{ProcessingError, Result};
use crate::epub::packager::parse_container_xml;

/// In-memory EPUB structure - maps relative paths to file contents
pub type EpubMemory = HashMap<String, Vec<u8>>;

/// Extract an EPUB from bytes into an in-memory structure
pub fn extract_epub_from_bytes(bytes: &[u8]) -> Result<EpubMemory> {
    use zip::ZipArchive;
    
    let reader = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| {
        ProcessingError::new(
            "input bytes",
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to open ZIP archive: {}", e)
        )
    })?;
    
    let mut files = EpubMemory::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            ProcessingError::new(
                i.to_string(),
                crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                format!("Failed to access archive entry {}: {}", i, e)
            )
        })?;
        
        let file_path = file.mangled_name().display().to_string();
        let mut content = Vec::new();
        
        std::io::copy(&mut file, &mut content).map_err(|e| {
            ProcessingError::new(
                &file_path,
                crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                format!("Failed to read file {}: {}", file_path, e)
            )
        })?;
        
        files.insert(file_path, content);
    }
    
    Ok(files)
}

/// Package an EPUB from an in-memory structure to bytes
pub fn package_epub_to_bytes(files: &EpubMemory) -> Result<Vec<u8>> {
    use zip::write::SimpleFileOptions;
    use std::io::Cursor;
    
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buffer);
    
    // Sort entries to ensure directories come before files
    let mut entries: Vec<(&String, &Vec<u8>)> = files.iter().collect();
    entries.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.0.split('/').collect();
        let b_parts: Vec<&str> = b.0.split('/').collect();
        a_parts.len().cmp(&b_parts.len())
    });
    
    for (path, content) in entries {
        if path.ends_with('/') {
            // Directory entry
            zip.add_directory(path.clone(), SimpleFileOptions::default()).map_err(|e| {
                ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to add directory {}: {}", path, e)
                )
            })?;
        } else {
            // File entry
            zip.start_file(path.clone(), SimpleFileOptions::default()).map_err(|e| {
                ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to start file {}: {}", path, e)
                )
            })?;
            
            zip.write(content).map_err(|e| {
                ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to write file {}: {}", path, e)
                )
            })?;
        }
    }
    
    zip.finish().map_err(|e| {
        ProcessingError::new(
            "output".to_string(),
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to finish ZIP archive: {}", e)
        )
    })?;
    
    Ok(buffer.into_inner())
}

/// Find the OPF path from in-memory container.xml content
pub fn find_opf_path_from_content(container_content: &str) -> Result<String> {
    parse_container_xml(container_content).map_err(|e| {
        ProcessingError::new(
            "container.xml",
            crate::error::ProcessingErrorType::XmlParsing(e.to_string()),
            format!("Failed to parse container.xml: {}", e)
        )
    })
}

/// Get container.xml content from in-memory EPUB
pub fn get_container_content(files: &EpubMemory) -> Result<String> {
    let container_path = "META-INF/container.xml";
    
    files.get(container_path)
        .map(|bytes| String::from_utf8(bytes.clone()).map_err(|e| {
            ProcessingError::new(
                container_path.to_string(),
                crate::error::ProcessingErrorType::XmlParsing(e.to_string()),
                format!("Failed to parse container.xml as UTF-8: {}", e)
            )
        }))
        .ok_or_else(|| {
            ProcessingError::new(
                container_path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!("{} not found in EPUB", container_path)),
                "container.xml not found in EPUB"
            )
        })?
}

/// Get file content from in-memory EPUB
pub fn get_file_content(files: &EpubMemory, path: &str) -> Result<String> {
    files.get(path)
        .map(|bytes| String::from_utf8(bytes.clone()).map_err(|e| {
            ProcessingError::new(
                path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!("Failed to parse {} as UTF-8: {}", path, e)),
                format!("Failed to parse file as UTF-8: {}", e)
            )
        }))
        .ok_or_else(|| {
            ProcessingError::new(
                path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!("File '{}' not found in EPUB", path)),
                format!("File '{}' not found in EPUB", path)
            )
        })?
}

/// Get file bytes from in-memory EPUB
pub fn get_file_bytes(files: &EpubMemory, path: &str) -> Result<Vec<u8>> {
    files.get(path)
        .cloned()
        .ok_or_else(|| {
            ProcessingError::new(
                path.to_string(),
                crate::error::ProcessingErrorType::FileNotFound(format!("File '{}' not found in EPUB", path)),
                format!("File '{}' not found in EPUB", path)
            )
        })
}

/// Set file content in in-memory EPUB
pub fn set_file_content(files: &mut EpubMemory, path: String, content: String) {
    files.insert(path, content.into_bytes());
}

/// Set file bytes in in-memory EPUB
pub fn set_file_bytes(files: &mut EpubMemory, path: String, bytes: Vec<u8>) {
    files.insert(path, bytes);
}

/// Remove file from in-memory EPUB
pub fn remove_file(files: &mut EpubMemory, path: &str) -> Option<Vec<u8>> {
    files.remove(path)
}

/// Check if file exists in in-memory EPUB
pub fn file_exists(files: &EpubMemory, path: &str) -> bool {
    files.contains_key(path)
}

/// List all files in in-memory EPUB
pub fn list_files(files: &EpubMemory) -> Vec<String> {
    files.keys().cloned().collect()
}

/// Get directory path from a file path
pub fn get_parent_path(path: &str) -> Option<String> {
    path.rfind('/').map(|pos| path[..pos].to_string())
}

/// Join paths correctly for EPUB structure
pub fn join_paths(base: &str, relative: &str) -> String {
    if base.is_empty() {
        return relative.to_string();
    }
    
    let base = base.trim_end_matches('/');
    let relative = relative.trim_start_matches('/');
    
    if relative.is_empty() {
        base.to_string()
    } else {
        format!("{}/{}", base, relative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_parent_path() {
        assert_eq!(get_parent_path("OEBPS/chapter1.xhtml"), Some("OEBPS".to_string()));
    }

    #[test]
    fn test_get_parent_path_root() {
        assert_eq!(get_parent_path("chapter1.xhtml"), None);
    }

    #[test]
    fn test_join_paths_empty_base() {
        assert_eq!(join_paths("", "chapter1.xhtml"), "chapter1.xhtml");
    }

    #[test]
    fn test_join_paths_with_base() {
        assert_eq!(join_paths("OEBPS", "chapter1.xhtml"), "OEBPS/chapter1.xhtml");
    }

    #[test]
    fn test_join_paths_trailing_slash() {
        assert_eq!(join_paths("OEBPS/", "chapter1.xhtml"), "OEBPS/chapter1.xhtml");
    }

    #[test]
    fn test_join_paths_leading_slash() {
        assert_eq!(join_paths("OEBPS", "/chapter1.xhtml"), "OEBPS/chapter1.xhtml");
    }

    #[test]
    fn test_join_paths_base_only() {
        assert_eq!(join_paths("OEBPS", ""), "OEBPS");
    }

    #[test]
    fn test_join_paths_nested() {
        assert_eq!(join_paths("OEBPS/EPUB", "chapter1.xhtml"), "OEBPS/EPUB/chapter1.xhtml");
    }

    #[test]
    fn test_find_opf_path_from_content() {
        let container_content = "<rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>";
        let result = find_opf_path_from_content(container_content);
        assert_eq!(result.unwrap(), "OEBPS/content.opf");
    }

    #[test]
    fn test_find_opf_path_from_content_error() {
        let container_content = "<other></other>";
        let result = find_opf_path_from_content(container_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_container_content_success() {
        let mut files = EpubMemory::new();
        files.insert("META-INF/container.xml".to_string(), b"<?xml version=\"1.0\"?>".to_vec());
        
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
    fn test_get_file_bytes_success() {
        let mut files = EpubMemory::new();
        files.insert("test.bin".to_string(), vec![1, 2, 3, 4]);
        
        let result = get_file_bytes(&files, "test.bin");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_get_file_bytes_not_found() {
        let files = EpubMemory::new();
        let result = get_file_bytes(&files, "missing.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_file_content() {
        let mut files = EpubMemory::new();
        set_file_content(&mut files, "test.txt".to_string(), "Hello World".to_string());
        
        assert!(files.contains_key("test.txt"));
        assert_eq!(files.get("test.txt").unwrap(), &b"Hello World".to_vec());
    }

    #[test]
    fn test_set_file_bytes() {
        let mut files = EpubMemory::new();
        set_file_bytes(&mut files, "test.bin".to_string(), vec![1, 2, 3]);
        
        assert!(files.contains_key("test.bin"));
        assert_eq!(files.get("test.bin").unwrap(), &vec![1, 2, 3]);
    }

    #[test]
    fn test_remove_file() {
        let mut files = EpubMemory::new();
        files.insert("test.txt".to_string(), b"Hello".to_vec());
        
        let removed = remove_file(&mut files, "test.txt");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), b"Hello".to_vec());
        assert!(!files.contains_key("test.txt"));
    }

    #[test]
    fn test_file_exists() {
        let mut files = EpubMemory::new();
        files.insert("test.txt".to_string(), b"Hello".to_vec());
        
        assert!(file_exists(&files, "test.txt"));
        assert!(!file_exists(&files, "missing.txt"));
    }

    #[test]
    fn test_list_files() {
        let mut files = EpubMemory::new();
        files.insert("a.txt".to_string(), vec![]);
        files.insert("b.txt".to_string(), vec![]);
        
        let files_list = list_files(&files);
        assert_eq!(files_list.len(), 2);
        assert!(files_list.contains(&"a.txt".to_string()));
        assert!(files_list.contains(&"b.txt".to_string()));
    }

    #[test]
    fn test_extract_epub_from_bytes_empty() {
        let bytes = vec![];
        let result = extract_epub_from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_package_epub_to_bytes_empty() {
        let files = EpubMemory::new();
        let result = package_epub_to_bytes(&files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_package_epub_to_bytes_with_files() {
        let mut files = EpubMemory::new();
        files.insert("test.txt".to_string(), vec![1, 2, 3]);
        
        let result = package_epub_to_bytes(&files);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 0);
    }

    #[test]
    fn test_package_epub_to_bytes_with_directories() {
        let mut files = EpubMemory::new();
        files.insert("dir/".to_string(), vec![]);
        files.insert("dir/file.txt".to_string(), vec![1, 2, 3]);
        
        let result = package_epub_to_bytes(&files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_epub_memory_type_alias() {
        let memory: EpubMemory = HashMap::new();
        assert!(memory.is_empty());
    }
}
