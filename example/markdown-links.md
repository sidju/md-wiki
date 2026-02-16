# Markdown Links Over Wikilinks

#design #markdown

Standard markdown links `[text](file.md)` work everywhere. Wikilinks `[[file]]` require specialized tools with file indices.

## The Wikilink Approach

Obsidian and similar tools use wikilinks for ergonomics:
- Shorter syntax: `[[file]]` vs `[text](file.md)`
- No need to specify link text
- Autocomplete from file index

**The cost:** Your notes require a tool that maintains a file index and resolves wikilinks. Opening them in a standard markdown viewer shows broken `[[references]]`.

## The Standard Link Approach

Standard markdown links work in any viewer:
- GitHub, GitLab render them correctly
- VS Code, Vim, any editor can follow them
- Static site generators handle them natively
- No file index needed

**The cost:** Slightly more verbose, renaming files breaks links.

## My Choice

Standard links because:
- Notes remain readable without specialized tools
- Works everywhere markdown is supported
- No lock-in to tools that support wikilinks
- Simplicity - links are just links

Related: [Hashtag Categories](hashtag-categories.md), [Tooling Philosophy](tooling-philosophy.md)
