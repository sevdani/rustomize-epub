//! Image processing module

pub mod dither;
pub mod processor;

pub use dither::dither_to_gray_levels;
pub use processor::{is_progressive_jpeg, process_image, should_process, ImageProcessingReport};
