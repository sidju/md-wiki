# Hashtag Categories

#design #markdown

md-wiki uses `#hashtags` for categories instead of folders, YAML frontmatter, or other alternatives.

## Why Categories?

Categories group related notes across different topics. But how to represent them while staying readable?

**Alternatives considered:**
1. **Folder hierarchies** - Breaks the flat zettelkasten principle
2. **YAML frontmatter** - Invisible when reading the markdown source
3. **Links to category pages** - Works, but category membership isn't obvious
4. **Hashtags** - Inline, visible, widely understood

## Why Hashtags

`#hashtags` win because:
- Readable in plain text without processing
- Widely understood convention (social media, etc.)
- Easy to search with simple tools: `grep "#design"`
- Work inline without breaking document flow
- md-wiki automatically generates category index pages

## The Trade-off

Hashtags aren't defined in standard markdown, so this is technically an extension.

**But it's pragmatic:**
- Source remains readable in any viewer
- No special syntax or parsing needed
- Feature is additive - documents work fine without hashtags
- Any tool can recognize them with basic pattern matching

A single concession that maintains the spirit of application agnosticism.

Related: [Markdown Links](markdown-links.md), [Flat Structure](flat-structure.md)
