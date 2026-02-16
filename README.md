# md-wiki

A minimal static wiki generator using markdown files as input. Strives to work as a Zettelkästen viewer with automatic backlink generation.

## Features

- 📝 Converts markdown files to HTML with proper formatting
- 🔗 Automatic backlink detection - shows which pages link to each page
- 🎨 Customizable header and footer templates
- ⚡ Fast and lightweight Rust implementation
- 🗂️ Preserves directory structure

## Installation

### Building from source

```bash
cargo build --release
```

The binary will be available at `target/release/md-wiki`.

## Usage

```bash
md-wiki [OPTIONS] <INPUT_DIRECTORY> [OUTPUT_DIRECTORY]
```

- `INPUT_DIRECTORY`: Directory containing your markdown files
- `OUTPUT_DIRECTORY`: Where HTML files will be created (optional, defaults to current directory)
- `--search-index <PATH>`: Optional path where search index will be written

### Examples

```bash
# Convert markdown files in 'wiki' directory to HTML in 'output' directory
md-wiki wiki output

# Generate with search index for search functionality
md-wiki wiki output --search-index output/search-data.js

# Using example files
md-wiki example example/output
```

## Directory Structure

Your input directory can contain:

- **Markdown files** (`.md`): Your wiki content - will be converted to HTML
  - **Important**: Markdown files must be in the source directory root (not in subdirectories)
  - Markdown files in subdirectories will be copied as-is without processing (with a warning)
- **Other files** (CSS, JS, images, etc.): Will be copied as-is to the output directory
  - Can be organized in subdirectories
- **header.html** (optional): HTML to prepend to each page
- **footer.html** (optional): HTML to append to each page

All files (except `header.html` and `footer.html`) will be copied to the output directory, maintaining the same directory structure. Markdown files in the root directory will be converted to HTML with the `.html` extension.

If `header.html` or `footer.html` are not provided, default minimal HTML will be used.

### Example Structure

```
wiki/
├── header.html          # Optional template
├── footer.html          # Optional template
├── index.md            # ✓ Converted to index.html
├── notes.md            # ✓ Converted to notes.html
├── zettelkasten.md     # ✓ Converted to zettelkasten.html
├── style.css           # Copied as-is
└── assets/
    ├── search.js       # Copied as-is
    ├── notes.md        # ⚠ Warning: copied as-is (not in root)
    └── images/
        └── logo.png    # Copied as-is
```

## How it Works

1. **Scans** all files in the input directory recursively
2. **Analyzes** markdown files in the root directory to build a backlink graph
3. **Converts** each root-level markdown file to HTML
4. **Copies** all other files to output (preserving directory structure)
5. **Combines** header + content + backlinks + footer for each HTML page
6. **Optionally generates** search index if `--search-index` flag is provided

## Backlinks

When you link to other markdown files using `[Link Text](filename.md)`, md-wiki will:

- Convert the link to point to the HTML version: `filename.html`
- Add a "Linked from" section at the bottom of `filename.html` showing all pages that link to it

This creates a bidirectional link structure, perfect for Zettelkästen-style note-taking.

## Example

See the `example/` directory for a sample wiki with header, footer, and interconnected pages.

To generate the example:

```bash
cargo build
./target/debug/md-wiki example example/output
```

Then open `example/output/index.html` in your browser.

## Search Functionality

md-wiki can optionally generate a search index when converting markdown files:

```bash
# Generate wiki with search index
md-wiki example example/output --search-index example/output/search-data.js
```

This creates a JavaScript file that embeds the search index as `window.SEARCH_INDEX_DATA` (global variable).

To enable search in your wiki, include this file in your header template:

```html
<script src="search-data.js"></script>
```

And include the search UI components. The example's `resources/search.js` provides a reference implementation.

**Note:** If you don't need search functionality, simply omit the `--search-index` flag and no search index will be generated.

The search functionality works both when opening HTML files directly from your filesystem (`file://` protocol) and when serving files via a web server (`http://` protocol).

## License

MIT License - see LICENSE file for details.


