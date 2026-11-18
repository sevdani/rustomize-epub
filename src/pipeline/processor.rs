//! Pipeline processor - main orchestration

use std::collections::HashMap;
use std::path::Path;
use crate::{ProcessingOptions, ProcessingReport, Result};
use crate::epub::packager::{extract_epub_from_bytes, package_epub_to_bytes};
use crate::epub::drm::has_drm_from_bytes;
use crate::epub::structure::{find_content_files_from_content, update_opf_content, update_xhtml_content, update_css_content, fix_svg_covers_content, fix_toc_content};
use crate::image::processor::process_images_from_bytes;
use crate::text::cleaner::clean_text_content_from_bytes;
use crate::html::cleaner::clean_html_content_from_bytes;
use crate::html::font_remover::remove_fonts_from_css_content;
use crate::html::css_cleaner::cleanup_css_content;
use crate::metadata::cleaner::clean_metadata_content;
use crate::toc::generator::{generate_ncx_content_memory, generate_nav_content_memory, add_ncx_to_opf_content, add_nav_to_opf_content};

/// Main processing function using file paths
pub fn process_epub_path(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &ProcessingOptions
) -> Result<ProcessingReport> {
    // Load input file into memory
    let input_bytes = std::fs::read(input_path.as_ref()).map_err(|e| {
        crate::error::ProcessingError::new(
            input_path.as_ref().display().to_string(),
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to read input file: {}", e)
        )
    })?;
    
    // Process in memory
    let (output_bytes, report) = process_epub_memory(&input_bytes, options)?;
    
    // Write output
    std::fs::write(output_path.as_ref(), &output_bytes).map_err(|e| {
        crate::error::ProcessingError::new(
            output_path.as_ref().display().to_string(),
            crate::error::ProcessingErrorType::ZipProcessing(e.to_string()),
            format!("Failed to write output file: {}", e)
        )
    })?;
    
    Ok(report)
}

/// Process EPUB from in-memory bytes
pub fn process_epub_memory(
    input_bytes: &[u8],
    options: &ProcessingOptions
) -> Result<(Vec<u8>, ProcessingReport)> {
    // Get original size for report
    let original_size = input_bytes.len() as u64;
    
    // Step 1: Check for DRM
    let is_drm = has_drm_from_bytes(input_bytes)?;
    
    if is_drm {
        let mut report = ProcessingReport {
            success: false,
            .. Default::default()
        };
        report.errors.push(crate::error::ProcessingError::new(
            "input",
            crate::error::ProcessingErrorType::ZipProcessing("DRM detected".to_string()),
            "This EPUB is DRM-protected. Please remove DRM first.".to_string()
        ));
        return Ok((Vec::new(), report));
    }
    
    // Step 2: Extract EPUB to in-memory structure
    let mut files = extract_epub_from_bytes(input_bytes)?;
    
    // Step 3: Remove OS artifacts (from in-memory structure)
    let os_artifacts_removed = remove_os_artifacts_from_memory(&mut files);
    
    // Step 4: Find OPF path from container.xml
    let container_content = crate::epub::memory::get_container_content(&files)?;
    let opf_path = crate::epub::packager::parse_container_xml(&container_content)?;
    
    // Step 5: Find content files from OPF content
    let opf_content = crate::epub::memory::get_file_content(&files, &opf_path)?;
    let content_files = find_content_files_from_content(&opf_content, &opf_path);
    
    // Step 6: Process images
    let mut image_report = crate::image::processor::ImageProcessingReport::default();
    let mut image_files = HashMap::new();
    
    if options.image_options.grayscale {
        // Load image files into memory
        for img_path in &content_files.images {
            if let Some(bytes) = files.remove(img_path) {
                image_files.insert(img_path.clone(), bytes);
            }
        }
        
        let (processed_images, img_report) = process_images_from_bytes(image_files, options)?;
        image_report = img_report;
        
        // Build rename map for processed images
        let rename_map: HashMap<String, String> = processed_images.keys().map(|old_path| {
                let old_name = Path::new(old_path).file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(old_path)
                    .to_string();
                (old_path.clone(), old_name)
            })
            .collect();
        
        // Update OPF with image renames
        let new_opf = update_opf_content(&opf_content, &rename_map);
        files.insert(opf_path.clone(), new_opf.into_bytes());
        
        // Update references in XHTML files
        for xhtml_file in &content_files.xhtml {
            if let Some(xhtml_bytes) = files.remove(xhtml_file) {
                let xhtml_content = String::from_utf8(xhtml_bytes).unwrap_or_default();
                let new_content = update_xhtml_content(&xhtml_content, &rename_map);
                files.insert(xhtml_file.clone(), new_content.into_bytes());
            }
        }
        
        // Update references in CSS files
        for css_file in &content_files.css {
            if let Some(css_bytes) = files.remove(css_file) {
                let css_content = String::from_utf8(css_bytes).unwrap_or_default();
                let new_content = update_css_content(&css_content, &rename_map);
                files.insert(css_file.clone(), new_content.into_bytes());
            }
        }
    }
    
    // Step 7: Clean text files
    let mut text_report = crate::text::cleaner::TextCleanReport::default();
    if options.text_cleanup {
        for xhtml_file in &content_files.xhtml {
            if let Some(xhtml_bytes) = files.remove(xhtml_file) {
                let (new_content, report) = clean_text_content_from_bytes(&xhtml_bytes, options);
                text_report.merge(&report);
                files.insert(xhtml_file.clone(), new_content);
            }
        }
    }
    
    // Step 8: Clean HTML
    let mut html_report = crate::html::cleaner::HtmlCleanReport::default();
    for xhtml_file in &content_files.xhtml {
        if let Some(xhtml_bytes) = files.remove(xhtml_file) {
            let (new_content, report) = clean_html_content_from_bytes(&xhtml_bytes, options);
            html_report.merge(&report);
            files.insert(xhtml_file.clone(), new_content);
        }
    }
    
    // Step 9: Remove fonts from CSS
    let mut fonts_removed = 0;
    if options.remove_fonts {
        for css_file in &content_files.css {
            if let Some(css_bytes) = files.remove(css_file) {
                let css_content = String::from_utf8(css_bytes).unwrap_or_default();
                let (new_content, removed) = remove_fonts_from_css_content(&css_content);
                fonts_removed += removed;
                files.insert(css_file.clone(), new_content.into_bytes());
            }
        }
    }
    
    // Step 10: Cleanup CSS
    let mut css_rules_removed = 0;
    if options.css_options.remove_unused_css {
        for css_file in &content_files.css {
            if let Some(css_bytes) = files.remove(css_file) {
                let css_content = String::from_utf8(css_bytes).unwrap_or_default();
                let (new_content, removed) = cleanup_css_content(&css_content);
                css_rules_removed += removed;
                files.insert(css_file.clone(), new_content.into_bytes());
            }
        }
    }
    
    // Step 11: Clean metadata
    let mut metadata_items_stripped = 0;
    let meta_report = if options.clean_metadata {
        let (new_content, report) = clean_metadata_content(&opf_content);
        metadata_items_stripped = report.items_stripped;
        files.insert(opf_path.clone(), new_content.into_bytes());
        report
    } else {
        crate::metadata::cleaner::EnhancedMetadataCleanReport::default()
    };
    
    // Step 12: Fix SVG covers
    let svg_covers_fixed;
    if options.generate_missing_cover {
        let (new_content, fixed) = fix_svg_covers_content(&opf_content);
        svg_covers_fixed = fixed;
        files.insert(opf_path.clone(), new_content.into_bytes());
    } else {
        svg_covers_fixed = 0;
    }
    
    // Step 13: Fix/Generate TOC
    let (toc_valid, toc_status) = fix_toc_content(&opf_content);
    if !toc_valid {
        // Generate TOC if missing
        let ncx_content = generate_ncx_content_memory(&opf_content, &content_files.xhtml)?;
        let nav_content = generate_nav_content_memory(&opf_content, &content_files.xhtml)?;
        
        // Write NCX file
        let ncx_path = opf_path.rsplit('/').next().unwrap_or("toc.ncx");
        files.insert(ncx_path.to_string(), ncx_content.into_bytes());
        
        // Add NCX reference to OPF
        let new_opf = add_ncx_to_opf_content(&opf_content, ncx_path);
        files.insert(opf_path.clone(), new_opf.into_bytes());
        
        // Write NAV file
        let nav_path = opf_path.rsplit('/').next().unwrap_or("nav.xhtml");
        files.insert(nav_path.to_string(), nav_content.into_bytes());
        
        // Add NAV reference to OPF
        let new_opf = add_nav_to_opf_content(&opf_content, nav_path);
        files.insert(opf_path.clone(), new_opf.into_bytes());
    }
    
    // Step 14: Package EPUB back to bytes
    let output_bytes = package_epub_to_bytes(&files)?;
    
    // Get final size
    let optimized_size = output_bytes.len() as u64;
    
    // Build report
    let mut report = ProcessingReport {
        success: true,
        original_size,
        optimized_size,
        output_filename: "processed.epub".to_string(),
        os_artifacts_removed,
        images_converted: image_report.images_processed,
        images_total: content_files.images.len(),
        fonts_removed,
        css_rules_removed,
        cover_generated: svg_covers_fixed > 0,
        svg_covers_fixed,
        toc_status,
        metadata_items_stripped,
        whitespace_cleaned: html_report.whitespace_cleaned,
        attrs_stripped: html_report.attrs_stripped,
        text_fixes_total: text_report.total_fixes(),
        text_cleanup_summary: text_report.summary(),
        .. Default::default()
    };

    
    // Add any errors from image processing
    for error in image_report.errors {
        report.errors.push(error);
    }
    
    // Add any errors from text processing
    for error in text_report.errors {
        report.errors.push(error);
    }
    
    // Add any errors from HTML cleaning
    for error in html_report.errors {
        report.errors.push(error);
    }
    
    // Add any errors from metadata cleaning
    for error in &meta_report.errors {
        report.errors.push(error.clone());
    }
    
    Ok((output_bytes, report))
}

/// Remove OS artifacts from in-memory EPUB
fn remove_os_artifacts_from_memory(files: &mut HashMap<String, Vec<u8>>) -> usize {
    let mut removed = 0;
    let os_artifacts = [".DS_Store", "Thumbs.db", "desktop.ini", "._.DS_Store"];
    let os_artifact_dirs = ["__MACOSX", ".git", ".svn"];
    
    // Remove OS artifact files
    for artifact in &os_artifacts {
        if files.remove(*artifact).is_some() {
            removed += 1;
        }
    }
    
    // Remove OS artifact directories
    let paths_to_remove: Vec<String> = files.keys()
        .filter(|k| {
            for dir in &os_artifact_dirs {
                let dir_prefix = format!("{}/", dir);
                if k.starts_with(&dir_prefix) || *k == *dir {
                    return true;
                }
            }
            false
        })
        .cloned()
        .collect();
    
    for path in paths_to_remove {
        if files.remove(&path).is_some() {
            removed += 1;
        }
    }
    
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_os_artifacts_from_memory_empty() {
        let mut files = HashMap::new();
        let removed = remove_os_artifacts_from_memory(&mut files);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_remove_os_artifacts_from_memory_files() {
        let mut files = HashMap::new();
        files.insert(".DS_Store".to_string(), vec![]);
        files.insert("Thumbs.db".to_string(), vec![]);
        files.insert("normal.txt".to_string(), vec![1, 2, 3]);
        
        let removed = remove_os_artifacts_from_memory(&mut files);
        assert_eq!(removed, 2);
        assert!(files.contains_key("normal.txt"));
    }

    #[test]
    fn test_remove_os_artifacts_from_memory_directories() {
        let mut files = HashMap::new();
        files.insert("__MACOSX/".to_string(), vec![]);
        files.insert("__MACOSX/file".to_string(), vec![]);
        files.insert("normal.txt".to_string(), vec![1, 2, 3]);
        
        let removed = remove_os_artifacts_from_memory(&mut files);
        assert!(removed >= 1);
        assert!(files.contains_key("normal.txt"));
    }

    #[test]
    fn test_process_epub_path_not_found() {
        let input_path = std::path::PathBuf::from("/nonexistent/path.epub");
        let output_path = std::path::PathBuf::from("/tmp/output.epub");
        let options = crate::ProcessingOptions::default();
        
        let result = process_epub_path(&input_path, &output_path, &options);
        
        assert!(result.is_err());
    }
}
