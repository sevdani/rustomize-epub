//! Image processing module

pub mod dither;
pub mod processor;

pub use dither::dither_to_gray_levels;
pub use processor::{process_image, should_process, is_progressive_jpeg, ImageProcessingReport};
