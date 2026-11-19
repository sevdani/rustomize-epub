/// Configuration types and presets for rustomize-epub
use std::collections::HashMap;

/// Device profile for e-ink devices
#[derive(Debug, Clone)]
pub struct DeviceProfile {
    pub name: String,
    pub width: u32,        // Portrait width
    pub height: u32,       // Portrait height
    pub gray_levels: Vec<u8>,  // e-ink grayscale levels (e.g., [0, 85, 170, 255])
    pub ppi: u32,  // pixels per inch
}

impl DeviceProfile {
    /// Custom device profile
    pub fn custom(width: u32, height: u32, ppi: u32) -> Self {
        Self {
            name: format!("Custom {}x{}", width, height),
            width,
            height,
            gray_levels: vec![0, 85, 170, 255],
            ppi,
        }
    }

    /// Xteink X3 profile (528x792)
    pub fn x3() -> Self {
        Self {
            name: "Xteink X3".to_string(),
            width: 528,
            height: 792,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 259,
        }
    }
    
    /// Xteink X4 profile (480x800)
    pub fn x4() -> Self {
        Self {
            name: "Xteink X4 (Classic/Pro)".to_string(),
            width: 480,
            height: 800,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 219,
        }
    }
    
    /// EEGOREAD A4 profile (552x768)
    pub fn eegoread_a4() -> Self {
        Self {
            name: "EEGOREAD A4".to_string(),
            width: 552,
            height: 768,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 238,
        }
    }
    
    /// BOOX Picco profile (480x800)
    pub fn boox_picco() -> Self {
        Self {
            name: "BOOX Picco".to_string(),
            width: 480,
            height: 800,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 235,
        }
    }
    
    /// PaperMono profile (480x800)
    pub fn paper_mono() -> Self {
        Self {
            name: "PaperMono (Lite)".to_string(),
            width: 480,
            height: 800,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 235,
        }
    }
    
    /// Seeed Sticky profile (480x800)
    pub fn seeed_sticky() -> Self {
        Self {
            name: "Seeed Sticky".to_string(),
            width: 480,
            height: 800,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 235,
        }
    }
    
    /// M5Paper v1.1 profile (1024x1024)
    pub fn m5paper_v1_1() -> Self {
        Self {
            name: "M5Paper v1.1".to_string(),
            width: 540,
            height: 960,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 234,
        }
    }
    
    /// T5 E-Paper S3 Pro Lite profile (1024x1024)
    pub fn t5_epaper_s3_pro_lite() -> Self {
        Self {
            name: "T5 E-Paper S3 Pro Lite".to_string(),
            width: 540,
            height: 960,
            gray_levels: vec![0, 85, 170, 255],
            ppi: 234,
        }
    }
}

/// Image processing options
#[derive(Debug, Clone)]
pub struct ImageOptions {
    pub grayscale: bool,
    pub contrast_boost: bool,
    pub contrast_factor: f32,      // Default 1.5
    pub quality: u8,               // JPEG quality (0-100)
    pub max_width: u32,            // From device profile
    pub max_height: u32,           // From device profile
    pub eink_quantize: bool,       // Quantize to device gray levels
    pub gray_levels: Vec<u8>,      // Device-specific levels
    pub light_novel_mode: bool,    // Rotate landscape images
    pub light_novel_rotate_left: bool,
}

impl ImageOptions {
    /// Create options for a specific device profile
    pub fn for_device(profile: &DeviceProfile) -> Self {
        Self {
            max_width: profile.width,
            max_height: profile.height,
            gray_levels: profile.gray_levels.clone(),
            ..Default::default()
        }
    }
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            grayscale: true,
            contrast_boost: true,
            contrast_factor: 1.5,
            quality: 70,
            max_width: 480,
            max_height: 800,
            eink_quantize: true,
            gray_levels: vec![0, 85, 170, 255],
            light_novel_mode: false,
            light_novel_rotate_left: true,
        }
    }
}

/// Text cleanup options
#[derive(Debug, Clone)]
pub struct TextCleanOptions {
    pub fix_whitespace: bool,
    pub fix_ocr: bool,
    pub normalize_quotes: bool,
    pub fix_encoding: bool,
    pub fix_punctuation: bool,
    pub normalize_unicode: bool,
}

impl Default for TextCleanOptions {
    fn default() -> Self {
        Self {
            fix_whitespace: true,
            fix_ocr: true,
            normalize_quotes: true,
            fix_encoding: true,
            fix_punctuation: true,
            normalize_unicode: true,
        }
    }
}

/// HTML processing options
#[derive(Debug, Clone)]
pub struct HtmlOptions {
    pub strip_attributes: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            strip_attributes: true,
        }
    }
}

/// CSS processing options
#[derive(Debug, Clone)]
pub struct CssOptions {
    pub remove_fonts: bool,
    pub remove_unused_css: bool,
}

impl Default for CssOptions {
    fn default() -> Self {
        Self {
            remove_fonts: true,
            remove_unused_css: true,
        }
    }
}

/// Filename format presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameFormat {
    Original,
    TitleAuthor,
    AuthorTitle,
    TitleOnly,
    Custom,
}

/// Metadata edits for filename generation
#[derive(Debug, Clone, Default)]
pub struct MetadataEdits {
    pub title: Option<String>,
    pub author: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<String>,
    pub language: Option<String>,
}

/// Processing options - matches epubkit logic
pub struct ProcessingOptions {
    pub device: DeviceProfile,
    pub image_options: ImageOptions,
    pub html_options: HtmlOptions,
    pub css_options: CssOptions,
    pub text_options: TextCleanOptions,
    pub remove_fonts: bool,
    pub remove_unused_css: bool,
    pub light_novel_mode: bool,
    pub light_novel_rotate_left: bool,
    pub generate_missing_cover: bool,
    pub clean_metadata: bool,
    pub text_cleanup: bool,
    pub normalize_quotes: bool,
    pub filename_format: FilenameFormat,
    pub filename_template: Option<String>,
    pub original_filename: Option<String>,
    pub metadata_edits: Option<MetadataEdits>,
}

impl ProcessingOptions {
    /// Quick preset: Images + text cleanup only
    pub fn quick() -> Self {
        Self {
            html_options: HtmlOptions::default(),
            remove_unused_css: false,
            remove_fonts: false,
            clean_metadata: false,
            generate_missing_cover: false,
            ..Default::default()
        }
    }
    
    /// Full preset: Complete device optimization
    pub fn full() -> Self {
        Self::default()
    }
    
    /// Custom preset: For fine-grained control
    pub fn custom() -> Self {
        Self::default()
    }
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            device: DeviceProfile::x4(),
            image_options: ImageOptions::default(),
            html_options: HtmlOptions::default(),
            css_options: CssOptions::default(),
            text_options: TextCleanOptions::default(),
            remove_fonts: true,
            remove_unused_css: true,
            light_novel_mode: false,
            light_novel_rotate_left: true,
            generate_missing_cover: true,   // Per user: always generate
            clean_metadata: true,
            text_cleanup: true,
            normalize_quotes: true,
            filename_format: FilenameFormat::AuthorTitle,
            filename_template: None,
            original_filename: None,
            metadata_edits: None,
        }
    }
}

/// Processing report with detailed results
#[derive(Debug, Clone)]
pub struct ProcessingReport {
    pub success: bool,
    pub errors: Vec<crate::error::ProcessingError>,
    pub original_size: u64,
    pub optimized_size: u64,
    pub output_filename: String,
    // Image counts
    pub images_converted: usize,
    pub images_total: usize,
    pub image_formats: HashMap<String, usize>,
    // CSS counts
    pub fonts_removed: usize,
    pub css_rules_removed: usize,
    // Cover
    pub cover_generated: bool,
    pub svg_covers_fixed: usize,
    // TOC
    pub toc_status: String,
    // Metadata
    pub metadata_items_stripped: usize,
    // HTML
    pub whitespace_cleaned: usize,
    pub attrs_stripped: usize,
    // Text cleanup
    pub text_fixes_total: usize,
    pub text_cleanup_summary: String,
    // OS artifacts
    pub os_artifacts_removed: usize,
    // Metadata edits
    pub metadata_edits_applied: bool,
}

impl ProcessingReport {
    /// Generate a human-readable summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if self.images_converted > 0 {
            parts.push(format!("Converted {}/{} images", self.images_converted, self.images_total));
        }
        
        if self.fonts_removed > 0 {
            parts.push(format!("Removed {} embedded fonts", self.fonts_removed));
        }
        
        if self.css_rules_removed > 0 {
            parts.push(format!("Stripped {} unused CSS rules", self.css_rules_removed));
        }
        
        if self.cover_generated {
            parts.push("Generated missing cover image".to_string());
        }
        
        if self.svg_covers_fixed > 0 {
            parts.push(format!("Fixed {} SVG cover wrappers", self.svg_covers_fixed));
        }
        
        if !self.toc_status.is_empty() {
            parts.push(format!("TOC: {}", self.toc_status));
        }
        
        if self.metadata_items_stripped > 0 {
            parts.push(format!("Stripped {} metadata entries", self.metadata_items_stripped));
        }
        
        if self.whitespace_cleaned > 0 {
            parts.push(format!("Cleaned {} empty elements", self.whitespace_cleaned));
        }
        
        if self.attrs_stripped > 0 {
            parts.push(format!("Stripped {} unnecessary attributes", self.attrs_stripped));
        }
        
        if self.text_fixes_total > 0 {
            parts.push(format!("Text cleanup: {}", self.text_cleanup_summary));
        }
        
        if self.os_artifacts_removed > 0 {
            parts.push(format!("Removed {} OS artifacts", self.os_artifacts_removed));
        }
        
        if self.original_size > 0 && self.optimized_size > 0 {
            let reduction = 100.0 * (1.0 - (self.optimized_size as f64) / (self.original_size as f64));
            parts.push(format!(
                "Size: {} → {} ({:.1}% reduction)",
                self.human_size(self.original_size),
                self.human_size(self.optimized_size),
                reduction
            ));
        }
        
        parts.join("; ")
    }
    
    /// Format size in human-readable form
    fn human_size(&self, size: u64) -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl Default for ProcessingReport {
    fn default() -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            original_size: 0,
            optimized_size: 0,
            output_filename: String::new(),
            images_converted: 0,
            images_total: 0,
            image_formats: HashMap::new(),
            fonts_removed: 0,
            css_rules_removed: 0,
            cover_generated: false,
            svg_covers_fixed: 0,
            toc_status: String::new(),
            metadata_items_stripped: 0,
            whitespace_cleaned: 0,
            attrs_stripped: 0,
            text_fixes_total: 0,
            text_cleanup_summary: String::new(),
            os_artifacts_removed: 0,
            metadata_edits_applied: false,
        }
    }
}

/// Content files structure for EPUB processing
#[derive(Debug, Clone, Default)]
pub struct ContentFiles {
    pub xhtml: Vec<String>,
    pub css: Vec<String>,
    pub images: Vec<String>,
    pub fonts: Vec<String>,
    pub ncx: Vec<String>,
    pub opf: Vec<String>,
    pub other: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_profile_x3() {
        let profile = DeviceProfile::x3();
        assert_eq!(profile.name, "Xteink X3");
        assert_eq!(profile.width, 528);
        assert_eq!(profile.height, 792);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 259);
    }

    #[test]
    fn test_device_profile_x4() {
        let profile = DeviceProfile::x4();
        assert_eq!(profile.name, "Xteink X4 (Classic/Pro)");
        assert_eq!(profile.width, 480);
        assert_eq!(profile.height, 800);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 219);
    }

    #[test]
    fn test_device_profile_custom() {
        let profile = DeviceProfile::custom(600, 800, 234);
        assert_eq!(profile.name, "Custom 600x800");
        assert_eq!(profile.width, 600);
        assert_eq!(profile.height, 800);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 234);
    }

    #[test]
    fn test_device_profile_eegoread_a4() {
        let profile = DeviceProfile::eegoread_a4();
        assert_eq!(profile.name, "EEGOREAD A4");
        assert_eq!(profile.width, 552);
        assert_eq!(profile.height, 768);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 238);
    }

    #[test]
    fn test_device_profile_boox_picco() {
        let profile = DeviceProfile::boox_picco();
        assert_eq!(profile.name, "BOOX Picco");
        assert_eq!(profile.width, 480);
        assert_eq!(profile.height, 800);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 235);
    }

    #[test]
    fn test_device_profile_paper_mono() {
        let profile = DeviceProfile::paper_mono();
        assert_eq!(profile.name, "PaperMono (Lite)");
        assert_eq!(profile.width, 480);
        assert_eq!(profile.height, 800);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 235);
    }

    #[test]
    fn test_device_profile_seeed_sticky() {
        let profile = DeviceProfile::seeed_sticky();
        assert_eq!(profile.name, "Seeed Sticky");
        assert_eq!(profile.width, 480);
        assert_eq!(profile.height, 800);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 235);
    }

    #[test]
    fn test_device_profile_m5paper_v1_1() {
        let profile = DeviceProfile::m5paper_v1_1();
        assert_eq!(profile.name, "M5Paper v1.1");
        assert_eq!(profile.width, 540);
        assert_eq!(profile.height, 960);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 234);
    }

    #[test]
    fn test_device_profile_t5_epaper_s3_pro_lite() {
        let profile = DeviceProfile::t5_epaper_s3_pro_lite();
        assert_eq!(profile.name, "T5 E-Paper S3 Pro Lite");
        assert_eq!(profile.width, 540);
        assert_eq!(profile.height, 960);
        assert_eq!(profile.gray_levels, vec![0, 85, 170, 255]);
        assert_eq!(profile.ppi, 234);
    }

    #[test]
    fn test_image_options_for_device() {
        let profile = DeviceProfile::x4();
        let options = ImageOptions::for_device(&profile);
        assert_eq!(options.max_width, 480);
        assert_eq!(options.max_height, 800);
        assert_eq!(options.gray_levels, vec![0, 85, 170, 255]);
        assert!(options.grayscale);
        assert!(options.eink_quantize);
    }

    #[test]
    fn test_image_options_default() {
        let options = ImageOptions::default();
        assert!(options.grayscale);
        assert!(options.contrast_boost);
        assert_eq!(options.contrast_factor, 1.5);
        assert_eq!(options.quality, 70);
        assert!(options.eink_quantize);
    }

    #[test]
    fn test_text_clean_options_default() {
        let options = TextCleanOptions::default();
        assert!(options.fix_whitespace);
        assert!(options.fix_ocr);
        assert!(options.normalize_quotes);
        assert!(options.fix_encoding);
        assert!(options.fix_punctuation);
        assert!(options.normalize_unicode);
    }

    #[test]
    fn test_css_options_default() {
        let options = CssOptions::default();
        assert!(options.remove_fonts);
        assert!(options.remove_unused_css);
    }

    #[test]
    fn test_html_options_default() {
        let options = HtmlOptions::default();
        assert!(options.strip_attributes);
    }

    #[test]
    fn test_processing_options_default() {
        let options = ProcessingOptions::default();
        assert_eq!(options.device.name, "Xteink X4 (Classic/Pro)");
        assert!(options.html_options.strip_attributes);
        assert!(options.remove_fonts);
        assert!(options.remove_unused_css);
        assert!(options.generate_missing_cover);
        assert!(options.clean_metadata);
        assert!(options.text_cleanup);
        assert!(options.normalize_quotes);
        assert_eq!(options.filename_format, FilenameFormat::AuthorTitle);
    }

    #[test]
    fn test_processing_options_quick() {
        let options = ProcessingOptions::quick();
        assert!(!options.remove_fonts);
        assert!(!options.remove_unused_css);
        assert!(!options.clean_metadata);
        assert!(!options.generate_missing_cover);
    }

    #[test]
    fn test_processing_options_full() {
        let options = ProcessingOptions::full();
        assert_eq!(options.device.name, "Xteink X4 (Classic/Pro)");
        assert!(options.remove_fonts);
        assert!(options.remove_unused_css);
        assert!(options.generate_missing_cover);
    }

    #[test]
    fn test_processing_report_summary_empty() {
        let report = ProcessingReport::default();
        let summary = report.summary();
        assert!(summary.is_empty());
    }

    #[test]
    fn test_processing_report_summary_with_images() {
        let report = ProcessingReport {
            images_converted: 10,
            images_total: 15,
            ..ProcessingReport::default()
        };
        let summary = report.summary();
        assert!(summary.contains("Converted 10/15 images"));
    }

    #[test]
    fn test_processing_report_summary_with_fonts() {
        let report = ProcessingReport {
            fonts_removed: 5,
            ..ProcessingReport::default()
        };
        let summary = report.summary();
        assert!(summary.contains("Removed 5 embedded fonts"));
    }

    #[test]
    fn test_processing_report_summary_with_size_reduction() {
        let report = ProcessingReport {
            original_size: 1000,
            optimized_size: 500,
            ..ProcessingReport::default()
        };
        let summary = report.summary();
        assert!(summary.contains("Size:"));
        assert!(summary.contains("50.0% reduction"));
    }

    #[test]
    fn test_processing_report_human_size_bytes() {
        let report = ProcessingReport::default();
        assert_eq!(report.human_size(100), "100 B");
        assert_eq!(report.human_size(1023), "1023 B");
    }

    #[test]
    fn test_processing_report_human_size_kb() {
        let report = ProcessingReport::default();
        assert_eq!(report.human_size(1024), "1.0 KB");
        assert_eq!(report.human_size(1024 * 1023), "1023.0 KB");
    }

    #[test]
    fn test_processing_report_human_size_mb() {
        let report = ProcessingReport::default();
        assert_eq!(report.human_size(1024 * 1024), "1.0 MB");
        assert_eq!(report.human_size(1024 * 1024 * 1023), "1023.0 MB");
    }

    #[test]
    fn test_processing_report_human_size_gb() {
        let report = ProcessingReport::default();
        assert_eq!(report.human_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_filename_format_values() {
        assert_eq!(FilenameFormat::Original, FilenameFormat::Original);
        assert_eq!(FilenameFormat::TitleAuthor, FilenameFormat::TitleAuthor);
        assert_ne!(FilenameFormat::AuthorTitle, FilenameFormat::TitleAuthor);
    }

    #[test]
    fn test_metadata_edits_default() {
        let edits = MetadataEdits::default();
        assert!(edits.title.is_none());
        assert!(edits.author.is_none());
        assert!(edits.series.is_none());
        assert!(edits.series_index.is_none());
        assert!(edits.language.is_none());
    }
}
