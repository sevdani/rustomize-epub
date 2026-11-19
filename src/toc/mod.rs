//! TOC handling module

pub mod generator;
pub mod validator;

pub use validator::{validate_nav, validate_ncx, ErrorLevel, ValidationError, ValidationResult};
