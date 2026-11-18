//! EPUB packager - ZIP extraction and repackaging

use crate::Result;
use std::io::Write;
use zip::{write::SimpleFileOptions, ZipArchive};

// Re-export memory operations
pub use crate::epub::memory::*;

/// Extract an EPUB from bytes into an in-memory structure
pub fn extract_epub_from_bytes(bytes: &[u8]) -> Result<EpubMemory> {
    use std::io::Cursor;

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| {
        crate::error::ProcessingError::new(
            "input bytes",
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to open ZIP archive: {}", e)
        )
    })?;
    
    let mut files = EpubMemory::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            crate::error::ProcessingError::new(
                i.to_string(),
                crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                format!("Failed to access archive entry {}: {}", i, e)
            )
        })?;
        
        let file_path_str = file.mangled_name().display().to_string();
        let mut content = Vec::new();
        
        std::io::copy(&mut file, &mut content).map_err(|e| {
            crate::error::ProcessingError::new(
                &file_path_str,
                crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                format!("Failed to read file {}: {}", file_path_str, e)
            )
        })?;
        
        files.insert(file_path_str, content);
    }
    
    Ok(files)
}

/// Package an EPUB from an in-memory structure to bytes
pub fn package_epub_to_bytes(files: &EpubMemory) -> Result<Vec<u8>> {
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
                crate::error::ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to add directory {}: {}", path, e)
                )
            })?;
        } else {
            // File entry
            zip.start_file(path.clone(), SimpleFileOptions::default()).map_err(|e| {
                crate::error::ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to start file {}: {}", path, e)
                )
            })?;
            
            zip.write(content).map_err(|e| {
                crate::error::ProcessingError::new(
                    path.clone(),
                    crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
                    format!("Failed to write file {}: {}", path, e)
                )
            })?;
        }
    }
    
    zip.finish().map_err(|e| {
        crate::error::ProcessingError::new(
            "output".to_string(),
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to finish ZIP archive: {}", e)
        )
    })?;
    
    Ok(buffer.into_inner())
}

/// Parse container.xml and extract the OPF path
pub fn parse_container_xml(content: &str) -> Result<String> {
    // Simple XML parsing for container.xml
    // Format: <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>
    
    // Find the rootfile element
    let start_tag = "<rootfile";
    let end_tag = "/>";
    
    let start_pos = content.find(start_tag).ok_or_else(|| {
        crate::error::ProcessingError::new(
            "container.xml",
            crate::error::ProcessingErrorType::XmlParsing("Failed to find <rootfile> in container.xml".to_string()),
            "Failed to find <rootfile> in container.xml"
        )
    })?;
    
    let end_pos = content[start_pos..].find(end_tag).ok_or_else(|| {
        crate::error::ProcessingError::new(
            "container.xml",
            crate::error::ProcessingErrorType::XmlParsing("Failed to find end of <rootfile> in container.xml".to_string()),
            "Failed to find end of <rootfile> in container.xml"
        )
    })?;
    
    let rootfile_element = &content[start_pos..start_pos + end_pos + end_tag.len()];
    
    // Extract full-path attribute
    let full_path_prefix = "full-path=\"";
    let full_path_start = rootfile_element.find(full_path_prefix).ok_or_else(|| {
        crate::error::ProcessingError::new(
            "container.xml",
            crate::error::ProcessingErrorType::XmlParsing("Failed to find full-path attribute in container.xml".to_string()),
            "Failed to find full-path attribute in container.xml"
        )
    })?;
    
    let full_path_start = full_path_start + full_path_prefix.len();
    let full_path_end = rootfile_element[full_path_start..].find('"').ok_or_else(|| {
        crate::error::ProcessingError::new(
            "container.xml",
            crate::error::ProcessingErrorType::XmlParsing("Failed to parse full-path value in container.xml".to_string()),
            "Failed to parse full-path value in container.xml"
        )
    })?;
    
    let opf_path = rootfile_element[full_path_start..full_path_start + full_path_end].to_string();
    
    Ok(opf_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_container_xml_basic() {
        let content = "<rootfile full-path=\"OEBPS/content.opf\" media-type=\"application/oebps-package+xml\"/>";
        
        let result = parse_container_xml(content);
        assert_eq!(result.unwrap(), "OEBPS/content.opf");
    }

    #[test]
    fn test_parse_container_xml_not_found() {
        let content = "<other></other>";
        
        let result = parse_container_xml(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_container_xml_with_path() {
        let content = "<rootfile full-path=\"EPUB/content.opf\" media-type=\"application/oebps-package+xml\"/>";
        
        let result = parse_container_xml(content);
        assert_eq!(result.unwrap(), "EPUB/content.opf");
    }

    #[test]
    fn test_parse_container_xml_with_nested_path() {
        let content = "<rootfile full-path=\"OEBPS/Book/content.opf\" media-type=\"application/oebps-package+xml\"/>";
        
        let result = parse_container_xml(content);
        assert_eq!(result.unwrap(), "OEBPS/Book/content.opf");
    }

    #[test]
    fn test_extract_epub_from_bytes_invalid() {
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
    fn test_os_artifact_patterns() {
        // Verify the OS artifact constants exist
        let _os_artifacts = [".DS_Store", "Thumbs.db", "desktop.ini", "._.DS_Store"];
        let _os_artifact_dirs = ["__MACOSX", ".git", ".svn"];
    }

    #[test]
    fn test_parse_container_xml_invalid_xml() {
        let content = "not valid xml";
        let result = parse_container_xml(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_container_xml_missing_rootfile() {
        let content = "<rootfile></rootfile>";
        let result = parse_container_xml(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_container_xml_empty() {
        let content = "";
        let result = parse_container_xml(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_container_xml_no_full_path() {
        let content = "<rootfile></rootfile>";
        let result = parse_container_xml(content);
        assert!(result.is_err());
    }
}
