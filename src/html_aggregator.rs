use pulldown_cmark::{html, Event, Tag, CowStr, TagEnd};

/// Convert text to a URL-friendly slug for heading IDs
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// HTML Aggregator that handles lookahead needs for anchor generation
pub struct HtmlAggregator {
    /// Collected events
    events: Vec<Event<'static>>,
}

impl HtmlAggregator {
    /// Create a new HTML aggregator
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    /// Ingest an event into the aggregator
    pub fn ingest(mut self, event: Event<'static>) -> Self {
        self.events.push(event);
        self
    }

    /// Convert the aggregated events to an HTML string
    /// This handles the lookahead needed for heading ID generation
    pub fn to_html_string(self) -> String {
        // Process events to add IDs to headings (requires lookahead)
        let mut processed_events = Vec::new();
        let mut i = 0;

        while i < self.events.len() {
            match &self.events[i] {
                Event::Start(Tag::Heading { level, id, classes, attrs }) => {
                    // Only generate ID if one wasn't already provided in markdown
                    let heading_id = if id.is_none() {
                        // Look ahead to collect heading text for ID generation
                        let mut current_heading_text = String::new();
                        let mut j = i + 1;
                        while j < self.events.len() {
                            if let Event::End(TagEnd::Heading(_)) = self.events[j] {
                                break;
                            }
                            if let Event::Text(ref text) = self.events[j] {
                                current_heading_text.push_str(text);
                            }
                            j += 1;
                        }

                        // Generate ID from heading text
                        if !current_heading_text.is_empty() {
                            Some(CowStr::from(slugify(&current_heading_text)))
                        } else {
                            None
                        }
                    } else {
                        id.clone()
                    };

                    processed_events.push(Event::Start(Tag::Heading {
                        level: *level,
                        id: heading_id,
                        classes: classes.clone(),
                        attrs: attrs.clone(),
                    }));
                }
                _ => {
                    processed_events.push(self.events[i].clone());
                }
            }
            i += 1;
        }

        let mut html_output = String::new();
        html::push_html(&mut html_output, processed_events.into_iter());
        html_output
    }
}
