// Floyd-Steinberg dithering for 4-level grayscale quantization

use image::{DynamicImage, ImageBuffer, Luma};

/// Apply Floyd-Steinberg dithering to convert to 4-level grayscale
///
/// This algorithm distributes quantization error to neighboring pixels,
/// creating the illusion of intermediate shades on e-ink displays.
///
/// # Arguments
/// * `img` - Input image (will be converted to grayscale)
/// * `gray_levels` - Target gray levels (typically [0, 85, 170, 255])
///
/// # Returns
/// * `DynamicImage::ImageLuma8` - Dithered 4-level grayscale image
pub fn dither_to_gray_levels(img: &DynamicImage, gray_levels: &[u8]) -> DynamicImage {
    let img = img.to_luma8();
    let width = img.width() as i32;
    let height = img.height() as i32;
    
    // Create error buffer (f32 for precision)
    // We use a 2D vector for error diffusion
    let mut error_buffer: Vec<Vec<f32>> = vec![vec![0.0; width as usize]; height as usize];
    
    let mut result = ImageBuffer::new(width as u32, height as u32);
    
    for y in 0..height {
        for x in 0..width {
            let px = img.get_pixel(x as u32, y as u32)[0] as f32;
            let error = error_buffer[y as usize][x as usize];
            
            // Apply accumulated error
            let val_with_error = px + error;
            
            // Clamp to valid range before quantization
            let val_with_error = val_with_error.clamp(0.0, 255.0);
            
            // Find closest gray level directly from original + error
            let closest = gray_levels.iter()
                .min_by_key(|&level| {
                    let level_f = *level as f32;
                    let diff = (val_with_error - level_f).abs();
                    diff as u32
                })
                .copied()
                .unwrap_or(0);
            
            result.put_pixel(x as u32, y as u32, Luma([closest]));
            
            // Calculate quantization error
            let quant_error = val_with_error - closest as f32;
            
            // Distribute error to neighbors (Floyd-Steinberg)
            // 7/16 to right neighbor
            if x + 1 < width {
                error_buffer[y as usize][(x + 1) as usize] += quant_error * 7.0 / 16.0;
            }
            // 3/16 to bottom-left
            if x > 0 && y + 1 < height {
                error_buffer[(y + 1) as usize][(x - 1) as usize] += quant_error * 3.0 / 16.0;
            }
            // 5/16 to bottom neighbor
            if y + 1 < height {
                error_buffer[(y + 1) as usize][x as usize] += quant_error * 5.0 / 16.0;
            }
            // 1/16 to bottom-right
            if x + 1 < width && y + 1 < height {
                error_buffer[(y + 1) as usize][(x + 1) as usize] += quant_error * 1.0 / 16.0;
            }
        }
    }
    
    DynamicImage::ImageLuma8(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::GenericImageView;

    #[test]
    fn test_dither_to_gray_levels_basic() {
        // Create a simple 2x2 grayscale image
        let mut img = ImageBuffer::new(2, 2);
        img.put_pixel(0, 0, Luma([100]));
        img.put_pixel(1, 0, Luma([150]));
        img.put_pixel(0, 1, Luma([200]));
        img.put_pixel(1, 1, Luma([250]));
        
        let gray_levels = vec![0, 85, 170, 255];
        let result = dither_to_gray_levels(&DynamicImage::ImageLuma8(img), &gray_levels);
        
        assert_eq!(result.width(), 2);
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn test_dither_with_known_pattern() {
        // Create gradient image
        let width = 4;
        let height = 1;
        let mut img = ImageBuffer::new(width, height);
        
        for x in 0..width {
            let val = (x * 85) as u8;
            img.put_pixel(x, 0, Luma([val]));
        }
        
        let gray_levels = vec![0, 85, 170, 255];
        let result = dither_to_gray_levels(&DynamicImage::ImageLuma8(img), &gray_levels);
        
        assert_eq!(result.width(), width);
        assert_eq!(result.height(), height);
    }

    #[test]
    fn test_dither_preserves_dimensions() {
        let width = 10;
        let height = 10;
        let mut img = ImageBuffer::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let val = ((x + y) * 12) as u8;
                img.put_pixel(x, y, Luma([val]));
            }
        }
        
        let gray_levels = vec![0, 85, 170, 255];
        let result = dither_to_gray_levels(&DynamicImage::ImageLuma8(img), &gray_levels);
        
        assert_eq!(result.width(), width);
        assert_eq!(result.height(), height);
        
        // Verify output is valid 4-level grayscale
        for pixel in result.pixels() {
            // ImageBuffer pixels return (x, y, pixel_value) tuples
            // pixel.2 is the pixel value (Luma([u8]) for Luma8)
            let val = pixel.2[0];
            assert!(gray_levels.contains(&val), "Value {} not in gray_levels {:?}", val, gray_levels);
        }
    }
}
