# md-wiki

A minimal wiki over zettelkästen inspired markdown notes.

- Renders a flat directory of markdown files into a static wiki.
- Provides rudimentary HTML templating for flexibility.
- Supports assets, includes any non-source files in the output as-is.
- Supports basic wiki-style search.
- Statically tracks backlinks: all pages linking to a page are listed under a
  "Linked from" section added to that page.
- Generates category pages: pages tagged with `#categories` are automatically
  collected into category index pages.


## Non-Goals

- Supporting non-standard syntaxes, the source for your wiki should be easy to
  view and edit in any markdown editor/viewer.
  - [[Wikilinks]] are a particular peeve, since they are fully redundant with
    normal markdown links and also require a page index to open...
- Supporting hierarchical markdown files, zettelkasten and wikis alike are
  flat as a concept and gain nothing from a hierarchy. (But if you really want
  to you can make a PR so the code handles hierarchical sources, I just don't
  care for it.)


## Usage

For any degree of regular usage you will want to install it.

From crates:
`cargo install md-wiki`

From repo:
`cargo install --path .`

After that `md-wiki` should be available in your path (assuming you've added
`$HOME/.cargo/bin` to your path). Run `md-wiki --help` for full usage
instructions.

### Example

```bash
md-wiki example example/output --index-filename search-data.js
```

Then open `example/output/index.html` in your browser.

You can also use the current directory as the source:

```bash
md-wiki . output
```

This will process all markdown files in the current directory and output HTML files to the `output` subdirectory.

### Wiki source directory structure

Here follows the files taken as input:
- `*.md`, processed into html files in the output dir.
  (Note that only markdown files directly in the source dir are processed)
- `header.html`, prepended to the generated html for each markdown file.
  Expected to open the `<html>` and `<body>` tags, should containt `<head>`.
- `footer.html`, appended to the generated html for each markdown file.
  Expected to close the `<body>` and `<html>` tags.
- `**/*`, all other files and directories are assumed to be assets and copied
  over as-is.

A wiki source directory isn't required to contain anything, but you should
add at least a markdown file for this CLI tool to be meaningful to use.

### Search index

`md-wiki` can generate a search index for the markdown files it converts. That
index simply maps filenames to the headings that exist within them, not much but
enough for a basic wiki-style search. Use the `--index-filename <filename>` flag
to create a javascript file in the output directory that writes the index to
`window.SEARCH_INDEX_DATA` (javascript because a fetch for a json file errors on
local html files).

Note the `search.js` file in the examples dir that utilizes this index for a
basic search bar.

### Ignoring files and directories

By default, `md-wiki` ignores all files and directories starting with a dot
(like `.git`, `.hidden`, etc.). This can be customized using the `--ignore-paths`
flag, which accepts glob patterns and can be specified multiple times.

**Note:** The special directories `.` (current directory) and `..` (parent directory) 
are always exempt from filtering when used as the source directory path. This allows 
you to use `.` as the input directory while still filtering out dotfiles within it. 
For example, with the default `.*` pattern, `md-wiki . output` will process files in 
the current directory but will still ignore `.git/`, `.hidden`, and other dotfiles.


## Contributing

If this is close but not quite what you need, feel free to open a PR.

## License

MIT License - see LICENSE file for details.
