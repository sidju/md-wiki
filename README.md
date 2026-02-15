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
md-wiki <input_directory> [output_directory]
```

- `input_directory`: Directory containing your markdown files
- `output_directory`: Where HTML files will be created (optional, defaults to current directory)

### Example

```bash
# Convert markdown files in 'wiki' directory to HTML in 'output' directory
md-wiki wiki output

# Using example files
md-wiki example example/output
```

## Directory Structure

Your input directory should contain:

- **Markdown files** (`.md`): Your wiki content
- **header.html** (optional): HTML to prepend to each page
- **footer.html** (optional): HTML to append to each page

If `header.html` or `footer.html` are not provided, default minimal HTML will be used.

### Example Structure

```
wiki/
├── header.html
├── footer.html
├── index.md
├── notes.md
└── zettelkasten.md
```

## How it Works

1. **Scans** all markdown files in the input directory
2. **Analyzes** links between files to build a backlink graph
3. **Converts** each markdown file to HTML
4. **Combines** header + content + backlinks + footer
5. **Outputs** HTML files with the same names as the markdown files

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

md-wiki automatically generates a search index when converting markdown files:

- **`search-data.js`**: JavaScript file that embeds the search index as `window.SEARCH_INDEX_DATA` (global variable)

To enable search in your wiki, include this file in your header template:

```html
<script src="search-data.js"></script>
```

And include the search UI components provided in the example's `resources/search.js`.

The search functionality works both when opening HTML files directly from your filesystem (`file://` protocol) and when serving files via a web server (`http://` protocol).

## License

MIT License - see LICENSE file for details.


