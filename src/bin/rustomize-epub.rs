/// rustomize-epub CLI binary
use clap::Parser;
use rustomize_epub::{process_epub_path, ProcessingOptions, DeviceProfile, MetadataEdits};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "rustomize-epub")]
#[command(about = "High-performance EPUB optimizer for e-ink devices", long_about = None)]
struct Cli {
    /// Input EPUB file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output EPUB file path
    #[arg(short, long)]
    output: PathBuf,

    /// Device profile (x3, x4, a4, picco, mono, sticky, m5, s3 or custom)
    #[arg(short, long, default_value = "x4")]
    device: String,

    /// Width for custom device profile
    #[arg(long)]
    width: Option<u32>,

    /// Height for custom device profile
    #[arg(long)]
    height: Option<u32>,

    /// Pixel per Inch
    #[arg(long)]
    ppi: Option<u32>,

    /// Preset to use (quick, full, custom)
    #[arg(short, long, default_value = "full")]
    preset: String,

    /// Output filename template
    #[arg(long)]
    filename_template: Option<String>,

    /// Filename preset (original, title-author, author-title, title-only)
    #[arg(long)]
    filename_format: Option<String>,

    /// Skip image processing
    #[arg(long)]
    no_images: bool,

    /// Skip text cleanup
    #[arg(long)]
    no_text: bool,

    /// Skip font removal
    #[arg(long)]
    no_fonts: bool,

    /// Skip CSS cleanup
    #[arg(long)]
    no_css: bool,

    /// Skip metadata cleaning
    #[arg(long)]
    no_metadata: bool,

    /// Skip TOC fix
    #[arg(long)]
    no_toc: bool,

    /// Book title (overrides metadata)
    #[arg(long)]
    title: Option<String>,

    /// Book author (overrides metadata)
    #[arg(long)]
    author: Option<String>,

    /// Book series name
    #[arg(long)]
    series: Option<String>,

    /// Book series index (number)
    #[arg(long)]
    series_index: Option<String>,

    /// Book language code (e.g., en, fr, de)
    #[arg(long)]
    language: Option<String>,

    /// Generate cover if missing
    #[arg(long, default_value = "true")]
    cover: bool,

    /// Enable light novel mode (rotate landscape images)
    #[arg(long)]
    light_novel: bool,

    /// Output directory for optimized files (for batch processing)
    #[arg(long)]
    output_dir: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Quiet mode (only output errors)
    #[arg(short, long, default_value = "false")]
    quiet: bool,

    /// Maximum parallel threads (0 = auto-detect)
    #[arg(long)]
    max_threads: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    // Validate inputs
    if !cli.input.exists() {
        eprintln!("Error: Input file does not exist: {:?}", cli.input);
        std::process::exit(1);
    }

    if !cli.input.extension().is_some_and(|ext| ext == "epub") {
        eprintln!("Error: Input file is not an EPUB: {:?}", cli.input);
        std::process::exit(1);
    }

    // Configure processing options
    let mut options = if cli.preset == "quick" {
        ProcessingOptions::quick()
    } else if cli.preset == "full" {
        ProcessingOptions::full()
    } else if cli.preset == "custom" {
        ProcessingOptions::custom()
    } else {
        eprintln!("Error: Unknown preset: {}", cli.preset);
        std::process::exit(1);
    };

    // Apply metadata edits from CLI
    if cli.title.is_some() || cli.author.is_some() || 
       cli.series.is_some() || cli.series_index.is_some() || 
       cli.language.is_some() {
        options.metadata_edits = Some(MetadataEdits {
            title: cli.title,
            author: cli.author,
            series: cli.series,
            series_index: cli.series_index,
            language: cli.language,
        });
    }

    // Override device profile
    options.device = match cli.device.as_str() {
        "x3" => DeviceProfile::x3(),
        "x4" => DeviceProfile::x4(),
        "a4" => DeviceProfile::eegoread_a4(),
        "picco" => DeviceProfile::boox_picco(),
        "mono" => DeviceProfile::paper_mono(),
        "sticky" => DeviceProfile::seeed_sticky(),
        "m5" => DeviceProfile::m5paper_v1_1(),
        "s3" => DeviceProfile::t5_epaper_s3_pro_lite(),
        _ => {
            if let (Some(width), Some(height), Some(ppi)) = (cli.width, cli.height, cli.ppi) {
                DeviceProfile::custom(width, height, ppi)
            } else {
                eprintln!("Error: Custom device requires --width, --height and --ppi");
                std::process::exit(1);
            }
        }
    };

    // Apply preset overrides
    if cli.no_images {
        options.image_options.grayscale = false;
    }
    if cli.no_text {
        options.text_cleanup = false;
    }
    if cli.no_fonts {
        options.remove_fonts = false;
    }
    if cli.no_css {
        options.remove_unused_css = false;
    }
    if cli.no_metadata {
        options.clean_metadata = false;
    }
    if cli.cover {
        options.generate_missing_cover = true;
    }
    if cli.light_novel {
        options.light_novel_mode = true;
    }

    // Configure parallelism if specified
    if let Some(max_threads) = cli.max_threads {
        if max_threads > 0 {
            // Note: rayon must be enabled for this to work
            #[cfg(feature = "parallel")]
            {
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(max_threads)
                    .build_global();
            }
        }
    }

    // Process the EPUB
    let start_time = Instant::now();

    let result = process_epub_path(&cli.input, &cli.output, &options);

    let elapsed = start_time.elapsed();

    match result {
        Ok(report) => {
            if !cli.quiet {
                println!("\n=== Processing Complete ===");
                println!("Original size: {}", format_size(report.original_size));
                println!("Optimized size: {}", format_size(report.optimized_size));
                println!("Size reduction: {:.1}%", 
                    100.0 * (1.0 - (report.optimized_size as f64) / (report.original_size as f64)));
                println!("Elapsed time: {:.2}s", elapsed.as_secs_f64());
                println!("\n{}", report.summary());
                
                if cli.verbose {
                    println!("\n=== Detailed Report ===");
                    println!("Success: {}", report.success);
                    println!("Output filename: {}", report.output_filename);
                    if !report.errors.is_empty() {
                        println!("Errors: {}", report.errors.len());
                        for error in &report.errors {
                            println!("  - {}: {} ({})", error.item, error.details, error.error_type);
                        }
                    }
                }
            }
            
            if cli.verbose || cli.output_dir.is_some() {
                println!("\nOutput file: {:?}", cli.output);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn format_size(size: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(10240), "10.0 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(5 * 1024 * 1024 * 1024), "5.0 GB");
    }

    #[test]
    fn test_format_size_edge_cases() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_cli_parser() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub"]);
        assert_eq!(cli.input.to_string_lossy(), "input.epub");
        assert_eq!(cli.output.to_string_lossy(), "output.epub");
        assert_eq!(cli.device, "x4");
        assert_eq!(cli.preset, "full");
    }

    #[test]
    fn test_cli_parser_with_preset() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub", "-p", "quick"]);
        assert_eq!(cli.preset, "quick");
    }

    #[test]
    fn test_cli_parser_with_device() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub", "-d", "x3"]);
        assert_eq!(cli.device, "x3");
    }

    #[test]
    fn test_cli_parser_with_custom_device() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub", "-d", "custom", "--width", "600", "--height", "800", "--ppi", "234"]);
        assert_eq!(cli.device, "custom");
        assert_eq!(cli.width, Some(600));
        assert_eq!(cli.height, Some(800));
        assert_eq!(cli.ppi, Some(234));
    }

    #[test]
    fn test_cli_parser_with_all_flags() {
        let cli = Cli::parse_from([
            "rustomize-epub",
            "-i", "input.epub",
            "-o", "output.epub",
            "-d", "x4",
            "-p", "full",
            "--no-images",
            "--no-text",
            "--no-fonts",
            "--no-css",
            "--no-metadata",
            "--no-toc",
            "--light-novel",
            "-v",
            "-q",
            "--max-threads", "4",
        ]);
        assert!(cli.no_images);
        assert!(cli.no_text);
        assert!(cli.no_fonts);
        assert!(cli.no_css);
        assert!(cli.no_metadata);
        assert!(cli.no_toc);
        assert!(cli.light_novel);
        assert!(cli.verbose);
        assert!(cli.quiet);
        assert_eq!(cli.max_threads, Some(4));
    }

    #[test]
    fn test_cli_parser_with_filename_options() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub", "--filename-template", "{title}-optimized", "--filename-format", "title-author"]);
        assert_eq!(cli.filename_template, Some("{title}-optimized".to_string()));
        assert_eq!(cli.filename_format, Some("title-author".to_string()));
    }

    #[test]
    fn test_cli_with_output_dir() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub", "--output-dir", "/tmp"]);
        assert_eq!(cli.output_dir, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::parse_from(["rustomize-epub", "-i", "input.epub", "-o", "output.epub"]);
        assert_eq!(cli.device, "x4");
        assert_eq!(cli.preset, "full");
        assert!(!cli.no_images);
        assert!(!cli.no_text);
        assert!(!cli.no_fonts);
        assert!(!cli.no_css);
        assert!(!cli.no_metadata);
        assert!(!cli.no_toc);
        assert!(cli.cover);
        assert!(!cli.light_novel);
        assert!(!cli.verbose);
        assert!(!cli.quiet);
        assert_eq!(cli.max_threads, None);
    }

    #[test]
    fn test_cli_parser_with_title_author() {
        let cli = Cli::parse_from([
            "rustomize-epub",
            "-i", "input.epub",
            "-o", "output.epub",
            "--title", "New Title",
            "--author", "New Author",
        ]);
        assert_eq!(cli.title, Some("New Title".to_string()));
        assert_eq!(cli.author, Some("New Author".to_string()));
    }

    #[test]
    fn test_cli_parser_with_metadata_edits() {
        let cli = Cli::parse_from([
            "rustomize-epub",
            "-i", "input.epub",
            "-o", "output.epub",
            "--title", "New Title",
            "--author", "New Author",
            "--series", "My Series",
            "--series-index", "3",
            "--language", "en",
        ]);
        assert_eq!(cli.title, Some("New Title".to_string()));
        assert_eq!(cli.author, Some("New Author".to_string()));
        assert_eq!(cli.series, Some("My Series".to_string()));
        assert_eq!(cli.series_index, Some("3".to_string()));
        assert_eq!(cli.language, Some("en".to_string()));
    }
}
