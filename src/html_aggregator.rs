use pulldown_cmark::{html, Event, Tag, CowStr, TagEnd};

/// Convert an event to a 'static lifetime by cloning borrowed data
fn event_to_static(event: Event) -> Event<'static> {
    match event {
        Event::Start(tag) => Event::Start(tag_to_static(tag)),
        Event::End(tag_end) => Event::End(tag_end),
        Event::Text(text) => Event::Text(text.into_string().into()),
        Event::Code(code) => Event::Code(code.into_string().into()),
        Event::Html(html) => Event::Html(html.into_string().into()),
        Event::InlineHtml(html) => Event::InlineHtml(html.into_string().into()),
        Event::FootnoteReference(label) => Event::FootnoteReference(label.into_string().into()),
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(checked) => Event::TaskListMarker(checked),
        Event::InlineMath(math) => Event::InlineMath(math.into_string().into()),
        Event::DisplayMath(math) => Event::DisplayMath(math.into_string().into()),
    }
}

fn tag_to_static(tag: Tag) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading { level, id, classes, attrs } => Tag::Heading {
            level,
            id: id.map(|s| s.into_string().into()),
            classes: classes.into_iter().map(|s| s.into_string().into()).collect(),
            attrs: attrs.into_iter().map(|(k, v)| (k.into_string().into(), v.map(|s| s.into_string().into()))).collect(),
        },
        Tag::BlockQuote(kind) => Tag::BlockQuote(kind),
        Tag::CodeBlock(kind) => Tag::CodeBlock(match kind {
            pulldown_cmark::CodeBlockKind::Indented => pulldown_cmark::CodeBlockKind::Indented,
            pulldown_cmark::CodeBlockKind::Fenced(lang) => pulldown_cmark::CodeBlockKind::Fenced(lang.into_string().into()),
        }),
        Tag::HtmlBlock => Tag::HtmlBlock,
        Tag::List(start) => Tag::List(start),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(label) => Tag::FootnoteDefinition(label.into_string().into()),
        Tag::Table(alignment) => Tag::Table(alignment),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link { link_type, dest_url, title, id } => Tag::Link {
            link_type,
            dest_url: dest_url.into_string().into(),
            title: title.into_string().into(),
            id: id.into_string().into(),
        },
        Tag::Image { link_type, dest_url, title, id } => Tag::Image {
            link_type,
            dest_url: dest_url.into_string().into(),
            title: title.into_string().into(),
            id: id.into_string().into(),
        },
        Tag::MetadataBlock(kind) => Tag::MetadataBlock(kind),
    }
}

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
    pub fn ingest<'a>(mut self, event: Event<'a>) -> Self {
        self.events.push(event_to_static(event));
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
