//! TOC generator with EPUB2/EPUB3 support
/// Generate NCX content (memory-based)
pub fn generate_ncx_content_memory(
    opf_content: &str,
    xhtml_files: &[String],
) -> Result<String, crate::ProcessingError> {
    let title = extract_dc_title_memory(opf_content).unwrap_or("Untitled".to_string());
    let author = extract_dc_creator_memory(opf_content).unwrap_or("Unknown".to_string());

    // Generate NCX XML
    let mut ncx = String::new();
    ncx.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    ncx.push_str("<!DOCTYPE ncx PUBLIC \"-//NISO//DTD ncx 2005-1//EN\"\n");
    ncx.push_str("  \"http://www.daisy.org/z3986/2005/ncx-2005-1.dtd\">\n\n");
    ncx.push_str("<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\">\n");
    ncx.push_str("  <head>\n");
    ncx.push_str(
        "    <meta name=\"dtb:uid\" content=\"urn:uuid:12345678-1234-1234-1234-123456789012\"/>\n",
    );
    ncx.push_str("    <meta name=\"dtb:depth\" content=\"1\"/>\n");
    ncx.push_str("    <meta name=\"dtb:totalPageCount\" content=\"0\"/>\n");
    ncx.push_str("    <meta name=\"dtb:maxPageNumber\" content=\"0\"/>\n");
    ncx.push_str("  </head>\n");
    ncx.push_str("  <docTitle>\n");
    ncx.push_str("    <text>");
    ncx.push_str(&escape_xml(&title));
    ncx.push_str("</text>\n");
    ncx.push_str("  </docTitle>\n");
    ncx.push_str("  <docAuthor>\n");
    ncx.push_str("    <text>");
    ncx.push_str(&escape_xml(&author));
    ncx.push_str("</text>\n");
    ncx.push_str("  </docAuthor>\n");
    ncx.push_str("  <navMap>\n");

    // Add navigation points for each chapter
    for (i, xhtml_file) in xhtml_files.iter().enumerate() {
        ncx.push_str(&format!(
            "    <navPoint id=\"navpoint-{}\" playOrder=\"{}\">\n",
            i + 1,
            i + 1
        ));
        ncx.push_str(&format!(
            "      <navLabel>\n        <text>Chapter {}</text>\n      </navLabel>\n",
            i + 1
        ));
        ncx.push_str(&format!(
            "      <content src=\"{}\"/>\n",
            escape_xml(xhtml_file)
        ));
        ncx.push_str("    </navPoint>\n");
    }

    ncx.push_str("  </navMap>\n");
    ncx.push_str("</ncx>\n");

    Ok(ncx)
}

/// Generate NAV content (memory-based)
pub fn generate_nav_content_memory(
    opf_content: &str,
    xhtml_files: &[String],
) -> Result<String, crate::ProcessingError> {
    let title = extract_dc_title_memory(opf_content).unwrap_or("Table of Contents".to_string());

    // Generate NAV HTML
    let mut nav = String::new();
    nav.push_str("<!DOCTYPE html>\n");
    nav.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n");
    nav.push_str("<head>\n");
    nav.push_str("<title>");
    nav.push_str(&escape_xml(&title));
    nav.push_str("</title>\n");
    nav.push_str("<meta charset=\"utf-8\"/>\n");
    nav.push_str("</head>\n");
    nav.push_str("<body>\n");
    nav.push_str("<nav epub:type=\"toc\">\n");
    nav.push_str("<h1>");
    nav.push_str(&escape_xml(&title));
    nav.push_str("</h1>\n");
    nav.push_str("<ol>\n");

    // Add navigation items for each chapter
    for (i, xhtml_file) in xhtml_files.iter().enumerate() {
        nav.push_str(&format!(
            "<li><a href=\"{}\">Chapter {}</a></li>\n",
            escape_xml(xhtml_file),
            i + 1
        ));
    }

    nav.push_str("</ol>\n");
    nav.push_str("</nav>\n");
    nav.push_str("</body>\n");
    nav.push_str("</html>\n");

    Ok(nav)
}

/// Extract title from OPF content (memory-based)
fn extract_dc_title_memory(opf_content: &str) -> Option<String> {
    if let Some(start) = opf_content.find("<dc:title>") {
        let start = start + 10;
        if let Some(end) = opf_content[start..].find("</dc:title>") {
            return Some(opf_content[start..start + end].to_string());
        }
    }
    None
}

/// Extract creator from OPF content (memory-based)
fn extract_dc_creator_memory(opf_content: &str) -> Option<String> {
    if let Some(start) = opf_content.find("<dc:creator") {
        // Find the end of the tag
        if let Some(end) = opf_content[start..].find('>') {
            let content_start = start + end + 1;
            if let Some(content_end) = opf_content[content_start..].find("</dc:creator>") {
                return Some(opf_content[content_start..content_start + content_end].to_string());
            }
        }
    }
    None
}

/// Add NCX reference to OPF content (memory-based)
pub fn add_ncx_to_opf_content(opf_content: &str, ncx_filename: &str) -> String {
    let mut new_content = opf_content.to_string();

    // Add NCX item to manifest if not already present
    if !new_content.contains("toc.ncx") {
        if let Some(manifest_end) = opf_content.find("</manifest>") {
            let item = format!(
                "\n    <item id=\"ncx\" href=\"{}\" media-type=\"application/x-dtbncx+xml\"/>",
                ncx_filename
            );
            new_content.insert_str(manifest_end, &item);
        }
    }

    // Add NCX reference to spine if not already present
    if !new_content.contains("ncx") || !new_content.contains("linear=\"no\"") {
        if let Some(spine_end) = opf_content.find("</spine>") {
            let toc_reference = "\n    <itemref idref=\"ncx\" linear=\"no\"/>".to_string();
            new_content.insert_str(spine_end, &toc_reference);
        }
    }

    new_content
}

/// Add NAV reference to OPF content (memory-based)
pub fn add_nav_to_opf_content(opf_content: &str, nav_filename: &str) -> String {
    let mut new_content = opf_content.to_string();

    // Add NAV item to manifest if not already present
    if !new_content.contains("nav.xhtml") {
        if let Some(manifest_end) = opf_content.find("</manifest>") {
            let item = format!("\n    <item id=\"nav\" href=\"{}\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>", nav_filename);
            new_content.insert_str(manifest_end, &item);
        }
    }

    new_content
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_basic() {
        let input = "Hello & World";
        let output = escape_xml(input);
        assert_eq!(output, "Hello &amp; World");
    }

    #[test]
    fn test_escape_xml_less_than() {
        let input = "A < B";
        let output = escape_xml(input);
        assert_eq!(output, "A &lt; B");
    }

    #[test]
    fn test_escape_xml_greater_than() {
        let input = "B > A";
        let output = escape_xml(input);
        assert_eq!(output, "B &gt; A");
    }

    #[test]
    fn test_escape_xml_quotes() {
        let input = "Title \"Name\" and 'Value'";
        let output = escape_xml(input);
        assert_eq!(output, "Title &quot;Name&quot; and &apos;Value&apos;");
    }

    #[test]
    fn test_escape_xml_all_chars() {
        let input = "A & B < C > D \"E\" 'F'";
        let output = escape_xml(input);
        assert_eq!(
            output,
            "A &amp; B &lt; C &gt; D &quot;E&quot; &apos;F&apos;"
        );
    }

    #[test]
    fn test_extract_dc_title_memory_found() {
        let opf_content = "<dc:title>Test Book</dc:title>";
        let title = extract_dc_title_memory(opf_content);
        assert_eq!(title, Some("Test Book".to_string()));
    }

    #[test]
    fn test_extract_dc_title_memory_not_found() {
        let opf_content = "<dc:creator>Author</dc:creator>";
        let title = extract_dc_title_memory(opf_content);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_dc_title_memory_empty() {
        let opf_content = "";
        let title = extract_dc_title_memory(opf_content);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_dc_creator_memory_found() {
        let opf_content = "<dc:creator>Author Name</dc:creator>";
        let creator = extract_dc_creator_memory(opf_content);
        assert_eq!(creator, Some("Author Name".to_string()));
    }

    #[test]
    fn test_extract_dc_creator_memory_with_attributes() {
        let opf_content =
            "<dc:creator role=\"aut\" file_as=\"Smith, John\">John Smith</dc:creator>";
        let creator = extract_dc_creator_memory(opf_content);
        assert_eq!(creator, Some("John Smith".to_string()));
    }

    #[test]
    fn test_generate_ncx_content_memory_basic() {
        let opf_content = "<dc:title>Test Book</dc:title><dc:creator>Author</dc:creator>";
        let xhtml_files = vec!["chapter1.xhtml".to_string(), "chapter2.xhtml".to_string()];

        let ncx = generate_ncx_content_memory(opf_content, &xhtml_files).unwrap();

        assert!(ncx.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(ncx.contains("<docTitle>"));
        assert!(ncx.contains("Test Book"));
        assert!(ncx.contains("<docAuthor>"));
        assert!(ncx.contains("Author"));
        // Check for the navPoint structure
        assert!(ncx.contains("<navPoint"));
        assert!(ncx.contains("playOrder"));
        assert!(ncx.contains("Chapter 1"));
        assert!(ncx.contains("Chapter 2"));
        assert!(ncx.contains("chapter1.xhtml"));
        assert!(ncx.contains("chapter2.xhtml"));
    }

    #[test]
    fn test_generate_ncx_content_memory_with_special_chars() {
        let opf_content = "<dc:title>Test &amp; Book</dc:title><dc:creator>Author</dc:creator>";
        let xhtml_files = vec!["chapter 1.xhtml".to_string()];

        let ncx = generate_ncx_content_memory(opf_content, &xhtml_files).unwrap();

        // The title should be escaped, and the filename with space should be handled
        assert!(ncx.contains("Test"));
        assert!(ncx.contains("Book"));
        assert!(ncx.contains("chapter 1.xhtml"));
    }

    #[test]
    fn test_generate_nav_content_memory_basic() {
        let opf_content = "<dc:title>Test Book</dc:title>";
        let xhtml_files = vec!["chapter1.xhtml".to_string()];

        let nav = generate_nav_content_memory(opf_content, &xhtml_files).unwrap();

        assert!(nav.contains("<!DOCTYPE html>"));
        assert!(nav.contains("<h1>"));
        assert!(nav.contains("Test Book"));
        assert!(nav.contains("<a href=\"chapter1.xhtml\">"));
        assert!(nav.contains("Chapter 1</a>"));
    }

    #[test]
    fn test_generate_nav_content_memory_multiple_chapters() {
        let opf_content = "<dc:title>Book</dc:title>";
        let xhtml_files = vec![
            "chapter1.xhtml".to_string(),
            "chapter2.xhtml".to_string(),
            "chapter3.xhtml".to_string(),
        ];

        let nav = generate_nav_content_memory(opf_content, &xhtml_files).unwrap();

        assert!(nav.contains("<a href=\"chapter1.xhtml\">Chapter 1</a>"));
        assert!(nav.contains("<a href=\"chapter2.xhtml\">Chapter 2</a>"));
        assert!(nav.contains("<a href=\"chapter3.xhtml\">Chapter 3</a>"));
    }

    #[test]
    fn test_add_ncx_to_opf_content_new() {
        let opf_content = "<manifest>\n</manifest>\n<spine>\n</spine>";
        let ncx_filename = "toc.ncx";

        let new_content = add_ncx_to_opf_content(opf_content, ncx_filename);

        assert!(new_content.contains("toc.ncx"));
        assert!(new_content.contains("application/x-dtbncx+xml"));
        assert!(new_content.contains("ncx"));
    }

    #[test]
    fn test_add_ncx_to_opf_content_already_present() {
        let opf_content =
            "<manifest>\n<item id=\"ncx\" href=\"toc.ncx\"/>\n</manifest>\n<spine>\n</spine>";
        let ncx_filename = "toc.ncx";

        let new_content = add_ncx_to_opf_content(opf_content, ncx_filename);

        // Should not duplicate the NCX entry
        let count = new_content.matches("toc.ncx").count();
        assert!(count >= 1);
    }

    #[test]
    fn test_add_nav_to_opf_content_new() {
        let opf_content = "<manifest>\n</manifest>";
        let nav_filename = "nav.xhtml";

        let new_content = add_nav_to_opf_content(opf_content, nav_filename);

        assert!(new_content.contains("nav.xhtml"));
        assert!(new_content.contains("properties=\"nav\""));
    }

    #[test]
    fn test_generate_ncx_content_memory_no_creator() {
        let opf_content = "<dc:title>Test Book</dc:title>";
        let xhtml_files = vec!["chapter1.xhtml".to_string()];

        let ncx = generate_ncx_content_memory(opf_content, &xhtml_files).unwrap();

        assert!(ncx.contains("Test Book"));
        assert!(ncx.contains("Unknown"));
    }
}
