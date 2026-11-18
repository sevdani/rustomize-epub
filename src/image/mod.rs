//! Image processing module

pub mod processor;

pub use processor::{process_image, should_process, is_progressive_jpeg, ImageProcessingReport};
