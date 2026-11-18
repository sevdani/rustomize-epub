# rustomize-epub

High-performance EPUB optimizer for e-ink devices.

## Features

- **Image Processing**: Convert images to baseline JPEG with 4-level grayscale quantization
- **HTML/CSS Cleaning**: Remove unused CSS, fonts, and unnecessary attributes
- **Text Cleanup**: Fix OCR ligatures, smart quotes, mojibake, punctuation issues
- **Metadata Cleaning**: Strip store-specific metadata
- **TOC Handling**: Validate and regenerate Table of Contents
- **Cover Generation**: Automatically generate cover if missing
- **Device Profiles**: Xteink X3/X4 or custom e-ink device profiles

## Installation

```bash
# Using cargo
cargo install --git https://github.com/your-username/rustomize-epub

# From source
git clone https://github.com/your-username/rustomize-epub
cd rustomize-epub
cargo build --release
```

## Usage

### Basic Usage

```bash
rustomize-epub -i input.epub -o output.epub
```

### With Presets

```bash
# Quick optimization (fast)
rustomize-epub -i input.epub -o output.epub --preset quick

# Full optimization (default)
rustomize-epub -i input.epub -o output.epub --preset full
```

### Custom Device Profile

```bash
# Use X3 device (528x792)
rustomize-epub -i input.epub -o output.epub --device x3

# Custom dimensions
rustomize-epub -i input.epub -o output.epub --device custom --width 528 --height 792
```

### Light Novel Mode

```bash
rustomize-epub -i input.epub -o output.epub --light-novel
```

### Skip Processing Steps

```bash
rustomize-epub -i input.epub -o output.epub --no-images --no-text
```

### Verbose Output

```bash
rustomize-epub -i input.epub -o output.epub --verbose
```

### Batch Processing

```bash
for f in *.epub; do
    rustomize-epub -i "$f" -o "optimized/${f%.epub}-optimized.epub" --preset full
done
```

## Command-Line Options

```
High-performance EPUB optimizer for e-ink devices

Usage: rustomize-epub [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>            Input EPUB file path
  -o, --output <OUTPUT>          Output EPUB file path
  -d, --device <DEVICE>          Device profile (x3, x4, or custom) [default: x4]
      --width <WIDTH>            Width for custom device profile
      --height <HEIGHT>          Height for custom device profile
  -p, --preset <PRESET>          Preset to use (quick, full, custom) [default: full]
      --filename-template <FILENAME_TEMPLATE>
                                 Output filename template
      --filename-format <FILENAME_FORMAT>
                                 Filename preset (original, title-author, author-title, title-only)
      --no-images                Skip image processing
      --no-text                  Skip text cleanup
      --no-fonts                 Skip font removal
      --no-css                   Skip CSS cleanup
      --no-metadata              Skip metadata cleaning
      --no-toc                   Skip TOC fix
      --[no-]cover               Generate cover if missing [default: true]
      --light-novel              Enable light novel mode (rotate landscape images)
  -o, --output-dir <OUTPUT_DIR>  Output directory for optimized files
  -v, --verbose                  Verbose output
  -q, --quiet                    Quiet mode (only output errors)
      --max-threads <MAX_THREADS>
                                 Maximum parallel threads (0 = auto-detect)
  -h, --help                     Print help
```

## Presets

| Preset | Images | Text | Fonts | CSS | Cover | Metadata | Speed |
|--------|--------|------|-------|-----|-------|----------|-------|
| Quick  | Yes    | Yes  | No    | No  | No    | No       | Fast  |
| Full   | Yes    | Yes  | Yes   | Yes | Yes   | Yes      | Medium|
| Custom | Custom | Custom| Custom| Custom| Custom| Custom   | Fast  |

## Device Profiles

| Device | Width | Height | Gray Levels |
|--------|-------|--------|-------------|
| X3     | 528   | 792    | 4-level     |
| X4     | 480   | 800    | 4-level     |

## Filename Templates

Use `{title}`, `{author}`, `{year}`, `{series}`, `{series_index}`, `{language}`, `{original}`

Example: `{author} - {title}.epub`

## Progress

Work in progress. See `plan.md` for implementation status.

## License

MIT
