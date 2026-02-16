use pulldown_cmark::{Event, Tag};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Represents a heading in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub text: String,
    pub id: String,
}

/// Represents a document with its headings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub path: String,
    pub headings: Vec<Heading>,
}

/// Search index structure
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchIndex {
    pub documents: Vec<Document>,
}

/// Analyzer that tracks backlinks and categories by analyzing markdown events
pub struct Analyzer {
    /// Maps from target HTML filename to a list of source HTML filenames that link to it
    backlinks: HashMap<String, Vec<String>>,
    /// Maps from category name to a list of HTML filenames that belong to that category
    categories: HashMap<String, Vec<String>>,
    /// Current source file being processed (HTML filename)
    current_file: String,
    /// Tracks headings for the current file
    current_headings: Vec<Heading>,
    /// All documents with their headings
    documents: Vec<Document>,
    /// Flag to track if we're inside a heading
    in_heading: bool,
    /// Current heading text being built
    current_heading_text: String,
    /// Current heading ID
    current_heading_id: Option<String>,
}

impl Analyzer {
    /// Create a new Analyzer for tracking backlinks and categories
    pub fn new() -> Self {
        Self {
            backlinks: HashMap::new(),
            categories: HashMap::new(),
            current_file: String::new(),
            current_headings: Vec::new(),
            documents: Vec::new(),
            in_heading: false,
            current_heading_text: String::new(),
            current_heading_id: None,
        }
    }

    /// Set the current file being processed
    pub fn set_current_file(&mut self, filename: String) {
        // Save previous file's headings if any
        if !self.current_file.is_empty() && !self.current_headings.is_empty() {
            let html_path = self.current_file.replace(".md", ".html");
            self.documents.push(Document {
                path: html_path,
                headings: self.current_headings.clone(),
            });
        }
        
        self.current_file = filename;
        self.current_headings.clear();
    }

    /// Analyze an event, tracking backlinks and categories, and return the event unchanged
    pub fn analyze<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        // Track links for backlinks (already translated to .html by link_translator)
        if let Event::Start(Tag::Link { ref dest_url, .. }) = event {
            let link = dest_url.to_string();
            // Handle links with or without fragments (e.g., "file.html" or "file.html#heading")
            let base_link = if let Some(pos) = link.find('#') {
                &link[..pos]
            } else {
                &link
            };
            // Only track relative wiki links (no scheme or host)
            // Skip URLs with schemes like http://, https://, ftp://, file://, etc.
            // Skip protocol-relative URLs starting with //
            // This allows relative paths (page.html, ./page.html, ../page.html)
            // and absolute wiki paths (/page.html) while filtering external URLs
            let is_relative = !base_link.contains("://") && !base_link.starts_with("//");
            
            if is_relative && base_link.ends_with(".html") {
                let backlink_list = self.backlinks
                    .entry(base_link.to_string())
                    .or_default();
                // Only add if not already present (deduplicate)
                if !backlink_list.contains(&self.current_file) {
                    backlink_list.push(self.current_file.clone());
                }
            }
        }
        
        // Track categories (hashtags)
        if let Event::Text(ref text) = event {
            let text_str = text.as_ref();
            
            for (idx, _) in text_str.match_indices('#') {
                // Check if # is at start or preceded by whitespace/punctuation
                let valid_prefix = if idx == 0 {
                    true
                } else {
                    // Find the character before # (respecting UTF-8 boundaries)
                    let mut prev_idx = idx - 1;
                    while prev_idx > 0 && !text_str.is_char_boundary(prev_idx) {
                        prev_idx -= 1;
                    }
                    if let Some(prev_char) = text_str[prev_idx..idx].chars().next() {
                        prev_char.is_whitespace() || matches!(prev_char, '(' | '[' | '{')
                    } else {
                        false
                    }
                };
                
                if valid_prefix {
                    // Extract category name after #
                    let after_hash = &text_str[idx + 1..];
                    let category_end = after_hash
                        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
                        .unwrap_or(after_hash.len());
                    
                    let category = &after_hash[..category_end];
                    
                    if !category.is_empty() {
                        let category_list = self.categories
                            .entry(category.to_string())
                            .or_default();
                        if !category_list.contains(&self.current_file) {
                            category_list.push(self.current_file.clone());
                        }
                    }
                }
            }
        }
        
        // Track headings
        match &event {
            Event::Start(Tag::Heading { id, .. }) => {
                self.in_heading = true;
                self.current_heading_text.clear();
                self.current_heading_id = id.as_ref().map(|s| s.to_string());
            }
            Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                if self.in_heading {
                    let heading_id = if let Some(id) = &self.current_heading_id {
                        id.clone()
                    } else {
                        // Slugify the heading text
                        slugify(&self.current_heading_text)
                    };
                    
                    self.current_headings.push(Heading {
                        text: self.current_heading_text.clone(),
                        id: heading_id,
                    });
                    
                    self.in_heading = false;
                    self.current_heading_text.clear();
                    self.current_heading_id = None;
                }
            }
            Event::Text(text) if self.in_heading => {
                self.current_heading_text.push_str(text);
            }
            _ => {}
        }
        
        event
    }

    /// Get the backlinks map
    pub fn get_backlinks(&self) -> &HashMap<String, Vec<String>> {
        &self.backlinks
    }
    
    /// Get the categories map
    pub fn get_categories(&self) -> &HashMap<String, Vec<String>> {
        &self.categories
    }
    
    /// Finalize analysis and return search index
    pub fn finalize(mut self) -> SearchIndex {
        // Add the last file's headings
        if !self.current_file.is_empty() && !self.current_headings.is_empty() {
            let html_path = self.current_file.replace(".md", ".html");
            self.documents.push(Document {
                path: html_path,
                headings: self.current_headings,
            });
        }
        
        SearchIndex {
            documents: self.documents,
        }
    }
}

/// Convert text to a URL-friendly slug for heading IDs
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() || c == '-' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

