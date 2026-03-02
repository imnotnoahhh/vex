# vex Documentation

## Overview

This directory contains the documentation infrastructure for vex, including custom styling and generation scripts.

## Quick Start

Generate and view documentation:
```bash
make docs
```

Or use the preview script:
```bash
./scripts/preview-docs.sh
```

## Documentation Features

### ✅ Pure English
All documentation is written in English, including:
- Module-level documentation (//!)
- Function and type documentation (///)
- Examples and usage notes

### ✅ Modern Styling
Custom CSS theme inspired by GitHub and docs.rs:
- Clean, professional appearance
- GitHub-style syntax highlighting
- Improved typography and readability
- Hidden anchor symbols (§) for cleaner look
- Responsive design for different screen sizes

### ✅ Comprehensive Coverage
Documentation for all 14 modules:
- Core modules: main, error, installer, downloader
- Supporting modules: resolver, cache, shell, lock, switcher
- Tool implementations: node, go, java, rust

## Files

- `custom.css` - Modern CSS theme with GitHub-inspired colors
- `header.html` - HTML header to inject custom styles
- `README.md` - This file
- `IMPROVEMENTS.md` - Detailed list of improvements
- `TRANSLATION_SUMMARY.md` - Translation completion summary

## Styling Details

### Color Scheme
- **Background**: Clean white (#ffffff)
- **Text**: Dark gray (#1a1a1a)
- **Links**: GitHub blue (#0969da)
- **Code blocks**: Light gray background (#f6f8fa)
- **Borders**: Subtle gray (#d0d7de)

### Typography
- **Font**: System font stack (-apple-system, BlinkMacSystemFont, etc.)
- **Size**: 16px base with 1.6 line height
- **Headings**: Bold (600 weight) with proper hierarchy
- **Code**: Monospace with syntax highlighting

### Key Improvements
1. **Hidden anchor symbols** - No more "§" cluttering headings
2. **Better spacing** - Comfortable reading with proper margins
3. **Modern code blocks** - Rounded corners and subtle borders
4. **GitHub-style highlighting** - Familiar syntax colors
5. **Responsive layout** - Works on all screen sizes

## Generation Process

The documentation is generated using:
1. Rust's built-in `cargo doc` command
2. Custom RUSTDOCFLAGS to inject header.html
3. Post-processing to copy custom.css

```bash
RUSTDOCFLAGS="--html-in-header docs/header.html" cargo doc --no-deps
cp docs/custom.css target/doc/
```

## Verification

Check documentation quality:
```bash
./scripts/check-docs.sh
```

This verifies:
- No Chinese characters in doc comments
- Documentation builds without warnings
- Custom styling files are present
- All modules have documentation

## Publishing

The documentation is automatically published to docs.rs when you publish a new crate version:
```bash
cargo publish
```

The custom styling will be applied via the `[package.metadata.docs.rs]` section in Cargo.toml.

## Customization

To modify the documentation appearance:

1. **Edit colors**: Modify the `:root` variables in `custom.css`
2. **Edit typography**: Adjust font sizes and line heights in `custom.css`
3. **Edit layout**: Modify spacing and padding in `custom.css`
4. **Regenerate**: Run `make docs` to see changes

## Examples

View specific module documentation:
- Main: `target/doc/vex/index.html`
- Error handling: `target/doc/vex/error/index.html`
- Installer: `target/doc/vex/installer/index.html`
- Downloader: `target/doc/vex/downloader/index.html`
- Tools: `target/doc/vex/tools/index.html`

## Troubleshooting

### CSS not applied
Make sure to copy custom.css after generating docs:
```bash
cp docs/custom.css target/doc/
```

### Anchor symbols still showing
Clear browser cache and reload the page.

### Documentation warnings
Run `cargo doc` to see specific warnings and fix them in source files.

## Resources

- [Rustdoc Book](https://doc.rust-lang.org/rustdoc/)
- [docs.rs](https://docs.rs/)
- [GitHub Markdown CSS](https://github.com/sindresorhus/github-markdown-css)
