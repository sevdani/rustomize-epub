//! TOC handling module

pub mod generator;
pub mod validator;

pub use validator::{validate_ncx, validate_nav, ValidationResult, ValidationError, ErrorLevel};
