use pulldown_cmark::{Event, Tag};
use std::collections::HashMap;

/// Analyzer that tracks backlinks by analyzing markdown events
pub struct Analyzer {
    /// Maps from target filename to a list of source filenames that link to it
    backlinks: HashMap<String, Vec<String>>,
    /// Current source file being processed
    current_file: String,
}

impl Analyzer {
    /// Create a new Analyzer for tracking backlinks
    pub fn new() -> Self {
        Self {
            backlinks: HashMap::new(),
            current_file: String::new(),
        }
    }

    /// Set the current file being processed
    pub fn set_current_file(&mut self, filename: String) {
        self.current_file = filename;
    }

    /// Analyze an event, tracking backlinks, and return the event unchanged
    pub fn analyze<'a>(&mut self, event: Event<'a>) -> Event<'a> {
        // Track links for backlinks
        if let Event::Start(Tag::Link { ref dest_url, .. }) = event {
            let link = dest_url.to_string();
            // Handle links with or without fragments (e.g., "file.md" or "file.md#heading")
            let base_link = if let Some(pos) = link.find('#') {
                &link[..pos]
            } else {
                &link
            };
            if base_link.ends_with(".md") {
                // Store just the filename part for backlinks
                let filename = base_link.to_string();
                self.backlinks
                    .entry(filename)
                    .or_default()
                    .push(self.current_file.clone());
            }
        }
        event
    }

    /// Get the backlinks map
    pub fn get_backlinks(&self) -> &HashMap<String, Vec<String>> {
        &self.backlinks
    }
}

