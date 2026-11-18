# rustomize-epub

High-performance EPUB optimizer for (microcontroller) e-ink devices.

## Processing Pipeline

rustomize-epub implements the same 20-step pipeline as epubkit:

| Step | What it does |
|------|-------------|
| 1 | **DRM check** — detects DRM-protected files and stops early |
| 2 | **Extract** — unpacks the EPUB ZIP structure |
| 3 | **Parse structure** — locates the OPF package file |
| 4 | **Read metadata** — extracts title, author, series, language, cover |
| 5 | **Apply metadata edits** — overwrites title/author if the user edited them in the CLI |
| 6 | **Find content files** — catalogs all XHTML, CSS, image, and font files |
| 7 | **Process images** — converts to baseline JPEG, resizes, applies 4-level grayscale |
| 8 | **Fix SVG covers** — unwraps SVG-wrapped cover images |
| 9 | **Generate cover** — creates title/author cover if missing |
| 10 | **Update references** — rewrites internal hrefs and srcs |
| 11 | **Repair HTML + strip attributes** — fixes malformed XHTML, strips data-*, aria-*, role, tabindex, etc. |
| 12 | **Remove unused CSS** — strips CSS rules that don't match any content |
| 13 | **Remove embedded fonts** — deletes @font-face rules and font files |
| 14 | **Normalize whitespace** — strips excessive empty paragraphs, adds page-break-before to h1/h2 |
| 15 | **Text cleanup** — fixes OCR ligatures, smart quotes, mojibake, punctuation |
| 16 | **Clean metadata** — strips store-specific tags (Amazon, Kobo, Calibre, etc.) |
| 17 | **Fix TOC** — validates and regenerates Table of Contents |
| 18 | **Clean OS artifacts** — removes .DS_Store, Thumbs.db, __MACOSX, etc. |
| 19 | **Repackage** — rebuilds the EPUB ZIP with correct mimetype |
| 20 | **Output** — applies filename preset and writes output file |

## Benefits

- **Written in Rust**: High execution speed with strong memory safety
- **In-memory processing**: Ultra-fast speed and low latency
- **CLI binary**: Run commands for batch processing
- **API**: Use the library to integrate into your own apps

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

# Lists of all arguments
rustomize-epub --help
```

### With Presets

```bash
# Quick optimization (fast) - images + text cleanup only
rustomize-epub -i input.epub -o output.epub --preset quick

# Full optimization (default) - complete device optimization
rustomize-epub -i input.epub -o output.epub --preset full

# Custom preset with fine-grained control
rustomize-epub -i input.epub -o output.epub --preset custom --no-fonts --no-css
```

### With Device Profiles

```bash
# Use X3 device (528x792)
rustomize-epub -i input.epub -o output.epub --device x3

rustomize-epub -i input.epub -o output.epub --device m5

# Custom dimensions
rustomize-epub -i input.epub -o output.epub \
  --device custom \
  --width 600 \
  --height 800 \
  --ppi 234
```

---

## Device Profiles

### Complete Device Reference

| Device | Width | Height | PPI | Gray Levels | Use Case |
|--------|-------|--------|-----|-------------|----------|
| X3 | 528 | 792 | 259 | 4-level | Xteink X3 (portrait) |
| X4 | 480 | 800 | 219 | 4-level | Xteink X4 Classic/Pro (portrait) |
| A4 | 552 | 768 | 238 | 4-level | EEGOREAD A4 (portrait) |
| Picco | 480 | 800 | 235 | 4-level | BOOX Picco (portrait) |
| Mono | 480 | 800 | 235 | 4-level | PaperMono Lite (portrait) |
| Sticky | 480 | 800 | 235 | 4-level | Seeed Sticky (portrait) |
| M5 | 540 | 960 | 234 | 4-level | M5Paper v1.1 (portrait) |
| S3 | 540 | 960 | 234 | 4-level | T5 E-Paper S3 Pro Lite (portrait) |

## Processing Presets

| Preset | Images | Text | Fonts | CSS | Cover | Metadata | Best for |
|--------|--------|------|-------|-----|-------|----------|----------|
| Quick  | Yes    | Yes  | No    | No  | No    | No       | Fast image + text pass |
| Full   | Yes    | Yes  | Yes   | Yes | Yes   | Yes      | Complete device optimization |
| Custom | Custom | Custom| Custom| Custom| Custom| Custom   | Fine-grained control |

## License

MIT
