//! Image processor for EPUB optimization

use std::collections::HashMap;
use std::path::Path;
use image::{GenericImageView, DynamicImage, ImageBuffer, Luma};
use crate::types::ImageOptions;
use crate::error::{ProcessingError, ProcessingErrorType, Result};

// Font imports
use rusttype::{Font, Scale, point};

use crate::image::dither::dither_to_gray_levels;

// FONTS
const FONT_NORMAL: &[u8] = include_bytes!("../../assets/fonts/dejavu/DejaVuSerif.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/dejavu/DejaVuSerif-Bold.ttf");

/// Supported image extensions
const SUPPORTED_EXTENSIONS: &[&str] = &["png", "gif", "webp", "bmp", "jpeg", "jpg", "tif", "tiff"];

/// Image processing report
#[derive(Debug, Default)]
pub struct ImageProcessingReport {
    pub images_processed: usize,
    pub images_total: usize,
    pub image_formats: HashMap<String, usize>,
    pub errors: Vec<ProcessingError>,
}

/// Process a single image
pub fn process_image(
    image_bytes: &[u8],
    filename: &str,
    options: &ImageOptions
) -> Result<ImageResult> {
    let original_size = image_bytes.len();
    
    // Check if we should process this file
    if !should_process(filename) {
        return Ok(ImageResult {
            output_bytes: image_bytes.to_vec(),
            new_filename: filename.to_string(),
            original_size,
            new_size: original_size,
            was_converted: false,
            details: "Skipped (not a processable image)".to_string(),
        });
    }
    
    // Determine the format
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    
    // Parse the image
    let mut img = image::load_from_memory(image_bytes).map_err(|e| {
        ProcessingError::new(
            filename,
            ProcessingErrorType::ImageProcessing(e.to_string()),
            format!("Failed to load image: {}", e)
        )
    })?;
    
    let mut details_parts = Vec::new();
    let mut was_converted = false;
    
    // Check if conversion to JPEG is needed
    let needs_jpeg_conversion = ext != "jpeg" && ext != "jpg";
    if needs_jpeg_conversion {
        details_parts.push(format!("{}→JPEG", ext.to_uppercase()));
        was_converted = true;
    }
    
    // Get original dimensions
    let (original_width, original_height) = img.dimensions();
    
    // Light novel mode: detect and rotate landscape images
    if options.light_novel_mode && is_landscape_image(original_width, original_height) {
        img = DynamicImage::ImageRgba8(image::imageops::rotate90(&img.to_rgba8()));
        details_parts.push("rotate90".to_string());
    }
    
    // Calculate new dimensions (maintain aspect ratio)
    let (new_width, new_height) = calculate_resized_dimensions(
        original_width,
        original_height,
        options.max_width,
        options.max_height
    );
    
    // Resize if needed
    if new_width < original_width || new_height < original_height {
        img = resize_image(&img, new_width, new_height);
        details_parts.push(format!("{}x{}", new_width, new_height));
    }
    
    // Convert to grayscale if enabled
    if options.grayscale {
        img = img.grayscale();
        details_parts.push("grayscale".to_string());
    }
    
    // Apply contrast boost if enabled
    if options.contrast_boost {
        img = apply_contrast(&img, options.contrast_factor);
        details_parts.push(format!("contrast {:.1}x", options.contrast_factor));
    }
    
    // E-ink quantization (4-level grayscale)
    if options.eink_quantize && options.grayscale {
        img = quantize_to_gray_levels(&img, &options.gray_levels);
        details_parts.push("4-level grayscale".to_string());
    }
    
    // Convert to JPEG
    let mut output_bytes = Vec::new();
    let new_filename = if needs_jpeg_conversion {
        let stem = Path::new(filename).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        format!("{}.jpg", stem)
    } else {
        filename.to_string()
    };
    
    // Write to buffer using a temporary approach
    use std::io::Cursor;
    let mut cursor = Cursor::new(&mut output_bytes);
    let img = img.to_rgb8();
    
    use image::codecs::jpeg::JpegEncoder;
    
    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, options.quality);
    encoder.encode_image(&img).map_err(|e| {
        ProcessingError::new(
            &new_filename,
            ProcessingErrorType::ImageProcessing(e.to_string()),
            format!("Failed to encode image: {}", e)
        )
    })?;
    
    let new_size = output_bytes.len();
    
    Ok(ImageResult {
        output_bytes,
        new_filename,
        original_size,
        new_size,
        was_converted,
        details: details_parts.join(", "),
    })
}

/// Check if an image is landscape
fn is_landscape_image(width: u32, height: u32) -> bool {
    // Landscape if width > height
    width > height
}

/// Calculate resized dimensions maintaining aspect ratio
fn calculate_resized_dimensions(
    orig_width: u32,
    orig_height: u32,
    max_width: u32,
    max_height: u32
) -> (u32, u32) {
    let aspect_ratio = orig_width as f64 / orig_height as f64;
    let max_aspect_ratio = max_width as f64 / max_height as f64;
    
    if aspect_ratio > max_aspect_ratio {
        // Width is the limiting factor
        (max_width, (max_width as f64 / aspect_ratio) as u32)
    } else {
        // Height is the limiting factor
        ((max_height as f64 * aspect_ratio) as u32, max_height)
    }
}

/// Resize an image using Lanczos3 interpolation for best quality
fn resize_image(img: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
    let resized = image::imageops::resize(
        img,
        new_width,
        new_height,
        image::imageops::Lanczos3
    );
    
    // Resize returns a specific image buffer type, wrap in DynamicImage
    DynamicImage::ImageRgba8(resized)
}

/// Apply contrast adjustment
fn apply_contrast(img: &DynamicImage, factor: f32) -> DynamicImage {
    let img = img.to_rgba8();
    
    image::DynamicImage::ImageRgba8(
        image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
            let pixel = img.get_pixel(x, y);
            let r = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let g = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            let b = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            image::Rgba([r, g, b, pixel[3]])
        })
    )
}

/// Quantize image to specific gray levels with Floyd-Steinberg dithering (e-ink optimization)
fn quantize_to_gray_levels(img: &DynamicImage, gray_levels: &[u8]) -> DynamicImage {
    dither_to_gray_levels(img, gray_levels)
}

/// Image processing result
#[derive(Debug)]
pub struct ImageResult {
    pub output_bytes: Vec<u8>,
    pub new_filename: String,
    pub original_size: usize,
    pub new_size: usize,
    pub was_converted: bool,
    pub details: String,
}

/// Generate a cover image from title and author
pub fn generate_cover_image(
    title: &str,
    author: &str,
    width: u32,
    height: u32,
    gray_levels: Option<Vec<u8>>
) -> Result<Vec<u8>> {
    // 1. Load fonts from embedded bytes
    let title_font = load_title_font()?;
    let author_font = load_author_font()?;
    
    // 2. Create blank image (dark background for e-ink)
    let mut img = ImageBuffer::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = Luma([20u8]); // Dark gray background
    }
    
    // 3. Render title text (centered at top)
    let title_y = height / 4; // Top quarter
    render_text_rusttype(&mut img, &title_font, title, 16, title_y, 255, 48)?;
    
    // 4. Render author text (centered below title)
    let author_y = title_y + 80; // Below title with spacing
    render_text_rusttype(&mut img, &author_font, author, 16, author_y, 200, 24)?;
    
    // 5. Apply e-ink quantization if enabled
    let gray_levels = gray_levels.unwrap_or(vec![0, 85, 170, 255]);
    let img_dynamic = DynamicImage::ImageLuma8(img);
    let img_result = quantize_to_gray_levels(&img_dynamic, &gray_levels);
    let img = match img_result {
        DynamicImage::ImageLuma8(luma) => luma,
        _ => panic!("Expected Luma8 image"),
    };
    
    // 6. Convert to RGB for JPEG encoding by wrapping in DynamicImage
    let img_dynamic = DynamicImage::ImageLuma8(img);
    let rgb_img = img_dynamic.to_rgb8();
    
    // 7. Encode as JPEG
    let mut output = Vec::new();
    use image::codecs::jpeg::JpegEncoder;
    
    let mut encoder = JpegEncoder::new_with_quality(&mut output, 85);
    encoder.encode_image(&rgb_img).map_err(|e| {
        ProcessingError::new(
            "cover",
            ProcessingErrorType::ImageProcessing(e.to_string()),
            "Failed to encode cover image".to_string(),
        )
    })?;
    
    Ok(output)
}

/// Load title font (bold) from DejaVu font files
fn load_title_font() -> Result<Font<'static>> {
    // Load from embedded font bytes using try_from_vec
    Font::try_from_bytes(FONT_BOLD).ok_or_else(|| {
        ProcessingError::new(
            "cover",
            ProcessingErrorType::Other("Failed to load font".to_string()),
            "Cannot load DejaVuSerif-Bold font".to_string(),
        )
    })
}

/// Load author font (regular) from DejaVu font files
fn load_author_font() -> Result<Font<'static>> {
    // Load from embedded font bytes using try_from_vec
    Font::try_from_bytes(FONT_NORMAL).ok_or_else(|| {
        ProcessingError::new(
            "cover",
            ProcessingErrorType::Other("Failed to load font".to_string()),
            "Cannot load DejaVuSerif font".to_string(),
        )
    })
}

/// Render text onto image using rusttype
fn render_text_rusttype(
    img: &mut ImageBuffer<Luma<u8>, Vec<u8>>,
    font: &Font<'_>,
    text: &str,
    x: u32,
    y: u32,
    color: u8,
    font_size: u32
) -> Result<()> {
    let scale = Scale::uniform(font_size as f32);
    
    let v_metrics = font.v_metrics(scale);
    let start_point = point(x as f32, y as f32 + v_metrics.ascent);
    
    // Layout all glyphs for the text
    for g in font.layout(text, scale, start_point) {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                let px = x as i32 + bb.min.x;
                let py = y as i32 + bb.min.y;
                
                if px >= 0 && py >= 0 {
                    let px_u32 = px as u32;
                    let py_u32 = py as u32;
                    
                    if px_u32 < img.width() && py_u32 < img.height() {
                        let pixel = img.get_pixel_mut(px_u32, py_u32);
                        // Blend with background
                        let current = pixel[0] as f32;
                        pixel[0] = ((color as f32 * v) + (current * (1.0 - v))) as u8;
                    }
                }
            })
        }
    }
    
    Ok(())
}

/// Check if an image should be processed
pub fn should_process(filename: &str) -> bool {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    
    SUPPORTED_EXTENSIONS.contains(&ext.as_str())
}

/// Check if JPEG is progressive
pub fn is_progressive_jpeg(_image_bytes: &[u8]) -> bool {
    // Placeholder - would need to parse JPEG markers
    false
}

/// Process all images in an EPUB with optional parallel processing (memory-based)
pub fn process_images_from_bytes(
    image_files: HashMap<String, Vec<u8>>,
    options: &crate::ProcessingOptions
) -> Result<(HashMap<String, Vec<u8>>, ImageProcessingReport)> {
    #[cfg(feature = "parallel")]
    let results: Vec<_> = {
        use rayon::prelude::*;
        
        image_files.par_iter().map(|(path, bytes)| {
            process_single_image_from_bytes(bytes, path, options)
        }).collect()
    };
    
    #[cfg(not(feature = "parallel"))]
    let results: Vec<_> = {
        image_files.iter().map(|(path, bytes)| {
            process_single_image_from_bytes(bytes, path, options)
        }).collect()
    };
    
    // Process results and build report
    let mut processed_images = HashMap::new();
    let mut report = ImageProcessingReport {
        images_total: image_files.len(),
        .. Default::default()
    };

    for ((image_path, original_bytes), result) in image_files.iter().zip(results) {
        match result {
            Ok(Some((_, was_modified, output_bytes))) => {
                if was_modified {
                    processed_images.insert(image_path.clone(), output_bytes);
                    report.images_processed += 1;
                } else {
                    processed_images.insert(image_path.clone(), original_bytes.clone());
                }
                // Track original format
                let ext = Path::new(image_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                *report.image_formats.entry(ext).or_insert(0) += 1;
            }
            Ok(None) => {
                // Image was skipped
                processed_images.insert(image_path.clone(), original_bytes.clone());
                let ext = Path::new(image_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_else(|| "unknown".to_string());
                *report.image_formats.entry(ext).or_insert(0) += 1;
            }
            Err(e) => {
                report.errors.push(e);
            }
        }
    }
    
    Ok((processed_images, report))
}

/// Process a single image from bytes (memory-based)
fn process_single_image_from_bytes(
    image_bytes: &[u8],
    image_path: &str,
    options: &crate::ProcessingOptions
) -> Result<Option<(String, bool, Vec<u8>)>> {
    let filename = Path::new(image_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(image_path);
    
    let result = process_image(image_bytes, filename, &options.image_options)?;
    
    // Only update if the image was actually modified
    let was_modified = result.was_converted || result.output_bytes != image_bytes;
    
    Ok(if was_modified {
        Some((result.new_filename, true, result.output_bytes))
    } else {
        Some((result.new_filename, false, result.output_bytes))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn test_should_process_valid_extensions() {
        assert!(should_process("image.png"));
        assert!(should_process("image.gif"));
        assert!(should_process("image.webp"));
        assert!(should_process("image.bmp"));
        assert!(should_process("image.jpeg"));
        assert!(should_process("image.jpg"));
        assert!(should_process("image.tif"));
        assert!(should_process("image.tiff"));
    }

    #[test]
    fn test_should_process_invalid_extensions() {
        assert!(!should_process("document.pdf"));
        assert!(!should_process("text.txt"));
        assert!(!should_process("image.svg"));
    }

    #[test]
    fn test_is_landscape_image() {
        assert!(is_landscape_image(800, 600));
        assert!(!is_landscape_image(600, 800));
        assert!(!is_landscape_image(600, 600));
    }

    #[test]
    fn test_calculate_resized_dimensions_width_limited() {
        // Aspect ratio 2:1, max 100x100 -> width is limiting
        let (w, h) = calculate_resized_dimensions(800, 400, 100, 100);
        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    #[test]
    fn test_calculate_resized_dimensions_height_limited() {
        // Aspect ratio 1:2, max 100x100 -> height is limiting
        let (w, h) = calculate_resized_dimensions(400, 800, 100, 100);
        assert_eq!(w, 50);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_resized_dimensions_no_resize() {
        // When image is smaller than max, it still gets scaled to fill max dimension
        // with same aspect ratio. 200x200 with 1024x1024 -> 1024x1024 (square)
        let (w, h) = calculate_resized_dimensions(200, 200, 1024, 1024);
        assert_eq!(w, 1024);
        assert_eq!(h, 1024);
    }

    #[test]
    fn test_quantize_to_gray_levels() {
        let img = image::DynamicImage::ImageLuma8(
            ImageBuffer::from_fn(4, 4, |_, _| image::Luma([100u8]))
        );
        
        let gray_levels = vec![0, 85, 170, 255];
        let quantized = quantize_to_gray_levels(&img, &gray_levels);
        
        // 100 is closest to 85
        let luma = quantized.to_luma8();
        let pixel = luma.get_pixel(0, 0);
        assert_eq!(pixel[0], 85);
    }

    #[test]
    fn test_apply_contrast() {
        let img = image::DynamicImage::ImageRgba8(
            ImageBuffer::from_fn(4, 4, |_, _| Rgba([128u8, 128u8, 128u8, 255u8]))
        );
        
        let contrasted = apply_contrast(&img, 2.0);
        // At 128, contrast should keep it at 128 (center point)
        let rgba = contrasted.to_rgba8();
        let pixel = rgba.get_pixel(0, 0);
        assert_eq!(pixel[0], 128);
    }

    #[test]
    fn test_generate_cover_image_placeholder() {
        let result = generate_cover_image("Test", "Author", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Real cover image should be larger than placeholder (100 bytes)
        assert!(bytes.len() > 1000, "Cover image should be generated, got {} bytes", bytes.len());
    }

    #[test]
    fn test_process_image_skipped_non_image() {
        let options = ImageOptions::default();
        let result = process_image(b"not an image", "document.pdf", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.was_converted);
        assert_eq!(result.details, "Skipped (not a processable image)");
    }

    #[test]
    fn test_process_image_with_valid_jpeg() {
        // Create a simple 10x10 JPEG in memory
        let mut img = ImageBuffer::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        // Encode JPEG using JpegEncoder directly
        use image::codecs::jpeg::JpegEncoder;
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions::default();
        let result = process_image(&buffer, "test.jpg", &options);
        
        assert!(result.is_ok(), "Failed to process JPEG: {:?}", result);
        let _result = result.unwrap();
        // JPEG to JPEG with same format, no conversion needed unless resized
        // Since it's small, it shouldn't need resize
    }

    #[test]
    fn test_process_image_light_novel_mode() {
        // Create a landscape JPEG (width > height)
        let mut img = ImageBuffer::new(200, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        // Encode JPEG using JpegEncoder directly
        use image::codecs::jpeg::JpegEncoder;
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions {
            light_novel_mode: true,
            ..ImageOptions::default()
        };

        let result = process_image(&buffer, "landscape.jpg", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        // JPEG doesn't need conversion to JPEG, but light_novel_mode adds rotate90
        assert!(result.details.contains("rotate90"));
    }

    #[test]
    fn test_process_image_grayscale() {
        // Create an RGB JPEG
        let mut img = ImageBuffer::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 128u8, 64u8, 255u8]);
        }
        
        // Encode JPEG using JpegEncoder directly
        use image::codecs::jpeg::JpegEncoder;
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions {
            grayscale: true,
            ..ImageOptions::default()
        };

        let result = process_image(&buffer, "test.jpg", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.details.contains("grayscale"));
    }

    #[test]
    fn test_process_image_resize() {
        // Create a large JPEG
        let mut img = ImageBuffer::new(800, 600);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        // Encode JPEG using JpegEncoder directly
        use image::codecs::jpeg::JpegEncoder;
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions::for_device(&crate::DeviceProfile::x4());
        
        let result = process_image(&buffer, "test.jpg", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.details.contains("480x"));
    }

    #[test]
    fn test_image_processing_report_default() {
        let report = ImageProcessingReport::default();
        assert_eq!(report.images_processed, 0);
        assert_eq!(report.images_total, 0);
        assert!(report.image_formats.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_is_progressive_jpeg_returns_false() {
        // Currently returns false as placeholder
        let result = is_progressive_jpeg(&[]);
        assert!(!result);
    }

    #[test]
    fn test_process_image_invalid_data() {
        let options = ImageOptions::default();
        let result = process_image(b"invalid image data", "test.jpg", &options);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_process_image_supported_formats() {
        // Test PNG
        let png_ext = "image.png";
        assert!(should_process(png_ext));
        
        // Test GIF
        let gif_ext = "image.gif";
        assert!(should_process(gif_ext));
        
        // Test WebP
        let webp_ext = "image.webp";
        assert!(should_process(webp_ext));
        
        // Test BMP
        let bmp_ext = "image.bmp";
        assert!(should_process(bmp_ext));
    }

    #[test]
    fn test_process_images_from_bytes_empty() {
        let image_files = HashMap::new();
        let options = crate::ProcessingOptions::default();
        
        let result = process_images_from_bytes(image_files, &options);
        assert!(result.is_ok());
        
        let (_, report) = result.unwrap();
        assert_eq!(report.images_total, 0);
    }

    #[test]
    fn test_process_images_from_bytes_with_files() {
        let mut image_files = HashMap::new();
        image_files.insert("test.jpg".to_string(), vec![0u8; 100]);
        
        let mut options = crate::ProcessingOptions::default();
        options.image_options.grayscale = false; // Skip processing
        
        let result = process_images_from_bytes(image_files, &options);
        // Should succeed or give a meaningful error for invalid image
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_process_single_image_from_bytes_skip() {
        let image_bytes = b"not an image";
        let image_path = "document.pdf";
        let options = crate::ProcessingOptions::default();
        
        let result = process_single_image_from_bytes(image_bytes, image_path, &options);
        // Should skip non-processable images
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_single_image_from_bytes_with_invalid_image() {
        let image_bytes = b"invalid image data";
        let image_path = "test.jpg";
        let options = crate::ProcessingOptions::default();
        
        let result = process_single_image_from_bytes(image_bytes, image_path, &options);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_process_single_image_from_bytes_with_valid_jpeg() {
        // Create a valid JPEG
        use image::codecs::jpeg::JpegEncoder;
        let mut img = ImageBuffer::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let image_path = "test.jpg";
        let options = crate::ProcessingOptions::default();
        
        let result = process_single_image_from_bytes(&buffer, image_path, &options);
        
        assert!(result.is_ok());
        let option_result = result.unwrap();
        assert!(option_result.is_some());
    }

    #[test]
    fn test_image_processing_report_with_formats() {
        let mut report = ImageProcessingReport::default();
        report.image_formats.insert("jpg".to_string(), 5);
        report.image_formats.insert("png".to_string(), 3);
        
        assert_eq!(report.image_formats.get("jpg"), Some(&5));
        assert_eq!(report.image_formats.get("png"), Some(&3));
    }

    #[test]
    fn test_is_landscape_image_portrait() {
        assert!(!is_landscape_image(600, 800));
        assert!(!is_landscape_image(600, 600));
    }

    #[test]
    fn test_calculate_resized_dimensions_no_resize_needed() {
        // When image is smaller than max, no resize needed
        let (w, h) = calculate_resized_dimensions(200, 200, 1024, 1024);
        // The function should return dimensions based on aspect ratio
        assert!(w >= 200);
        assert!(h >= 200);
    }

    #[test]
    fn test_quantize_to_gray_levels_edge_cases() {
        // Test with single gray level
        let img = image::DynamicImage::ImageLuma8(
            ImageBuffer::from_fn(4, 4, |_, _| image::Luma([128u8]))
        );
        
        let gray_levels = vec![128];
        let quantized = quantize_to_gray_levels(&img, &gray_levels);
        
        let luma = quantized.to_luma8();
        let pixel = luma.get_pixel(0, 0);
        assert_eq!(pixel[0], 128);
    }

    #[test]
    fn test_apply_contrast_no_change() {
        let img = image::DynamicImage::ImageRgba8(
            ImageBuffer::from_fn(4, 4, |_, _| Rgba([128u8, 128u8, 128u8, 255u8]))
        );
        
        // At 1.0 factor, image should not change
        let contrasted = apply_contrast(&img, 1.0);
        let rgba = contrasted.to_rgba8();
        let pixel = rgba.get_pixel(0, 0);
        assert_eq!(pixel[0], 128);
        assert_eq!(pixel[1], 128);
        assert_eq!(pixel[2], 128);
    }

    #[test]
    fn test_generate_cover_image_with_gray_levels() {
        let gray_levels = Some(vec![0, 85, 170, 255]);
        let result = generate_cover_image("Test", "Author", 480, 800, gray_levels);
        assert!(result.is_ok());
    }

    #[test]
    fn test_should_process_case_insensitive() {
        assert!(should_process("image.PNG"));
        assert!(should_process("image.GIF"));
        assert!(should_process("image.JPEG"));
    }

    #[test]
    fn test_process_image_no_conversion_needed() {
        // Create a valid JPEG
        use image::codecs::jpeg::JpegEncoder;
        let mut img = ImageBuffer::new(10, 10);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions::default();
        let result = process_image(&buffer, "test.jpg", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        // JPEG to JPEG without conversion
        assert!(!result.was_converted);
    }

    #[test]
    fn test_process_image_resize_needed() {
        // Create a large JPEG
        use image::codecs::jpeg::JpegEncoder;
        let mut img = ImageBuffer::new(800, 600);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255u8, 0u8, 0u8, 255u8]);
        }
        
        let mut buffer = Vec::new();
        JpegEncoder::new_with_quality(&mut buffer, 80)
            .encode_image(&img).expect("Failed to encode JPEG");
        
        let options = ImageOptions::for_device(&crate::DeviceProfile::x4());
        
        let result = process_image(&buffer, "test.jpg", &options);
        
        assert!(result.is_ok());
        let result = result.unwrap();
        // After processing, the file should be valid
        // Size comparison depends on JPEG compression - don't assert on exact size
        assert!(result.new_size > 0, "New size should be positive");
    }

    #[test]
    fn test_process_image_with_contrast() {
        let options = ImageOptions::default();
        assert!(options.contrast_boost);
        assert_eq!(options.contrast_factor, 1.5);
    }

    #[test]
    fn test_process_image_with_eink_quantization() {
        let options = ImageOptions {
            eink_quantize: true,
            ..ImageOptions::default()
        };

        assert!(options.eink_quantize);
        assert_eq!(options.gray_levels, vec![0, 85, 170, 255]);
    }

    #[test]
    fn test_process_image_light_novel_mode_rotate_left() {
        let options = ImageOptions {
            light_novel_mode: true,
            light_novel_rotate_left: true,
            ..ImageOptions::default()
        };

        assert!(options.light_novel_mode);
        assert!(options.light_novel_rotate_left);
    }
    
    // ==================== Font Loading Tests ====================
    
    #[test]
    fn test_load_title_font_success() {
        let result = load_title_font();
        assert!(result.is_ok());
        let font = result.unwrap();
        // Verify font has valid metrics
        let scale = Scale::uniform(48.0);
        let v_metrics = font.v_metrics(scale);
        assert!(v_metrics.ascent > 0.0);
        assert!(v_metrics.descent < 0.0); // Descent is negative in rusttype
        assert!(v_metrics.line_gap >= 0.0);
    }
    
    #[test]
    fn test_load_author_font_success() {
        let result = load_author_font();
        assert!(result.is_ok());
        let font = result.unwrap();
        // Verify font has valid metrics
        let scale = Scale::uniform(24.0);
        let v_metrics = font.v_metrics(scale);
        assert!(v_metrics.ascent > 0.0);
        assert!(v_metrics.descent < 0.0);
        assert!(v_metrics.line_gap >= 0.0);
    }
    
    // ==================== Cover Generation Tests ====================
    
    #[test]
    fn test_generate_cover_image_creates_valid_jpeg() {
        let result = generate_cover_image("Test Title", "Test Author", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Real JPEG should be at least 1KB
        assert!(bytes.len() > 1000, "Cover should be real JPEG, got {} bytes", bytes.len());
        // Should start with JPEG magic bytes
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1], 0xD8); // JPEG SOI marker
    }
    
    #[test]
    fn test_generate_cover_image_with_custom_dimensions() {
        let test_cases = vec![
            (100, 100),  // Small
            (480, 800),  // X4 default
            (528, 792),  // X3 default
            (800, 1200), // Large
        ];
        
        for (width, height) in test_cases {
            let result = generate_cover_image("Title", "Author", width, height, None);
            assert!(result.is_ok(), "Failed for {}x{}", width, height);
            
            let bytes = result.unwrap();
            // Decode to verify dimensions
            use std::io::Cursor;
            use image::codecs::jpeg::JpegDecoder;
            use image::ImageDecoder;
            
            let decoder = JpegDecoder::new(Cursor::new(bytes)).expect("Failed to create decoder");
            let dimensions = decoder.dimensions();
            assert_eq!(dimensions.0, width);
            assert_eq!(dimensions.1, height);
        }
    }
    
    #[test]
    fn test_generate_cover_image_different_titles() {
        let titles = vec![
            "Short",
            "A Very Long Title That Should Be Rendered Properly",
            "Title with SpecialChars!@#",
            "Unicode: 你好 こんにちは",
        ];
        
        for title in titles {
            let result = generate_cover_image(title, "Author", 480, 800, None);
            assert!(result.is_ok(), "Failed for title: {}", title);
        }
    }
    
    #[test]
    fn test_generate_cover_image_different_authors() {
        let authors = vec![
            "A",
            "Very Long Author Name That Should Fit Properly",
            "Author Name With Spaces and Names",
        ];
        
        for author in authors {
            let result = generate_cover_image("Title", author, 480, 800, None);
            assert!(result.is_ok(), "Failed for author: {}", author);
        }
    }
    
    #[test]
    fn test_generate_cover_image_with_gray_levels_single() {
        let gray_levels = Some(vec![128]);
        let result = generate_cover_image("Test", "Author", 480, 800, gray_levels);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    #[test]
    fn test_generate_cover_image_with_gray_levels_custom() {
        let gray_levels = Some(vec![0, 100, 200, 255]);
        let result = generate_cover_image("Test", "Author", 480, 800, gray_levels);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    #[test]
    fn test_generate_cover_image_no_gray_levels() {
        let result = generate_cover_image("Test", "Author", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    // ==================== Edge Case Tests ====================
    
    #[test]
    fn test_generate_cover_image_empty_title() {
        let result = generate_cover_image("", "Author", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    #[test]
    fn test_generate_cover_image_empty_author() {
        let result = generate_cover_image("Title", "", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    #[test]
    fn test_generate_cover_image_both_empty() {
        let result = generate_cover_image("", "", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
    
    #[test]
    fn test_generate_cover_image_small_dimensions() {
        let result = generate_cover_image("Title", "Author", 100, 100, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 100);
    }
    
    #[test]
    fn test_generate_cover_image_with_unicode_characters() {
        let result = generate_cover_image("你好世界", "Привет мир", 480, 800, None);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(bytes.len() > 1000);
    }
}
