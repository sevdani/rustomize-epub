//! Filename formatter - generates filenames from EPUB metadata

use crate::types::FilenameFormat;

/// Generate filename from template
/// Supports placeholders: {title}, {author}, {series}, {series_index}, {language}, {original}
pub fn generate_filename_from_template(
    title: &str,
    author: &str,
    series: Option<&str>,
    series_index: Option<&str>,
    language: Option<&str>,
    template: &str,
) -> String {
    let mut result = template.to_string();
    
    // Replace placeholders
    result = result.replace("{title}", title);
    result = result.replace("{author}", author);
    
    if let Some(s) = series {
        result = result.replace("{series}", s);
    } else {
        // Remove {series} placeholder if no series
        result = result.replace("{series}", "");
    }
    
    if let Some(i) = series_index {
        result = result.replace("{series_index}", i);
    } else {
        result = result.replace("{series_index}", "");
    }
    
    if let Some(lang) = language {
        result = result.replace("{language}", lang);
    } else {
        result = result.replace("{language}", "");
    }
    
    // Clean up double spaces and dashes
    result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
    result = result.replace("  ", " ");
    
    sanitize_filename(&result)
}

/// Generate filename from format preset
pub fn generate_filename_from_preset(
    title: &str,
    author: &str,
    preset: FilenameFormat,
    original_name: Option<&str>,
) -> String {
    let filename = match preset {
        FilenameFormat::Original => {
            original_name.map(|s| s.to_string()).unwrap_or_else(|| "book.epub".to_string())
        }
        FilenameFormat::TitleAuthor => format!("{} - {}.epub", sanitize_filename(title), sanitize_filename(author)),
        FilenameFormat::AuthorTitle => format!("{} - {}.epub", sanitize_filename(author), sanitize_filename(title)),
        FilenameFormat::TitleOnly => format!("{}.epub", sanitize_filename(title)),
        FilenameFormat::Custom => {
            // This should have been handled by template-based generation
            original_name.map(|s| s.to_string()).unwrap_or_else(|| "book.epub".to_string())
        }
    };
    
    filename
}

/// Sanitize filename by removing/replace invalid characters
fn sanitize_filename(name: &str) -> String {
    // Remove/replace invalid characters for filenames
    let mut result = name.to_string();
    
    // Replace invalid characters with underscores
    for c in &['<', '>', ':', '"', '/', '\\', '|', '?', '*'] {
        result = result.replace(*c, "_");
    }
    
    // Remove leading/trailing whitespace
    result = result.trim().to_string();
    
    // Replace multiple spaces with single space
    result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
    
    // Remove trailing dots (Windows issue)
    result = result.trim_end_matches('.').to_string();
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename_template_basic() {
        let result = generate_filename_from_template(
            "Title",
            "Author",
            None,
            None,
            None,
            "{title} - {author}.epub",
        );
        assert_eq!(result, "Title - Author.epub");
    }

    #[test]
    fn test_generate_filename_template_with_series() {
        let result = generate_filename_from_template(
            "Title",
            "Author",
            Some("Series"),
            Some("1"),
            None,
            "{author} - {title} ({series} {series_index}).epub",
        );
        assert_eq!(result, "Author - Title (Series 1).epub");
    }

    #[test]
    fn test_generate_filename_template_with_language() {
        let result = generate_filename_from_template(
            "Title",
            "Author",
            None,
            None,
            Some("en"),
            "{title} - {author} ({language}).epub",
        );
        assert_eq!(result, "Title - Author (en).epub");
    }

    #[test]
    fn test_generate_filename_template_sanitizes() {
        let result = generate_filename_from_template(
            "Title: With Colon",
            "Author/With/Slashes",
            None,
            None,
            None,
            "{title} - {author}.epub",
        );
        assert_eq!(result, "Title_ With Colon - Author_With_Slashes.epub");
    }

    #[test]
    fn test_generate_filename_preset_original() {
        let result = generate_filename_from_preset(
            "Title",
            "Author",
            FilenameFormat::Original,
            Some("original.epub"),
        );
        assert_eq!(result, "original.epub");
    }

    #[test]
    fn test_generate_filename_preset_title_author() {
        let result = generate_filename_from_preset(
            "Title",
            "Author",
            FilenameFormat::TitleAuthor,
            None,
        );
        assert_eq!(result, "Title - Author.epub");
    }

    #[test]
    fn test_generate_filename_preset_author_title() {
        let result = generate_filename_from_preset(
            "Title",
            "Author",
            FilenameFormat::AuthorTitle,
            None,
        );
        assert_eq!(result, "Author - Title.epub");
    }

    #[test]
    fn test_generate_filename_preset_title_only() {
        let result = generate_filename_from_preset(
            "Title",
            "Author",
            FilenameFormat::TitleOnly,
            None,
        );
        assert_eq!(result, "Title.epub");
    }

    #[test]
    fn test_sanitize_filename() {
        let result = sanitize_filename("Test: File <Name> .txt");
        // All invalid chars become underscores
        assert_eq!(result, "Test_ File _Name_ .txt");
    }

    #[test]
    fn test_sanitize_filename_trailing_dot() {
        let result = sanitize_filename("Test.");
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_sanitize_filename_multiple_spaces() {
        let result = sanitize_filename("Test   File   Name");
        assert_eq!(result, "Test File Name");
    }

    #[test]
    fn test_generate_filename_from_template_all_fields() {
        let result = generate_filename_from_template(
            "Foundation",
            "Isaac Asimov",
            Some("Foundation"),
            Some("1"),
            Some("en"),
            "{author} - {title} ({series} {series_index}) [{language}].epub",
        );
        assert_eq!(result, "Isaac Asimov - Foundation (Foundation 1) [en].epub");
    }

    #[test]
    fn test_generate_filename_from_template_missing_optional() {
        let result = generate_filename_from_template(
            "Title",
            "Author",
            None,  // No series
            None,  // No series_index
            None,  // No language
            "{title} - {author} ({series} {series_index}) [{language}].epub",
        );
        // Missing fields should be removed (empty)
        assert_eq!(result, "Title - Author ( ) [].epub");
    }
}
