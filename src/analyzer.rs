use pulldown_cmark::{Event, Tag};
use std::collections::HashMap;

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

    /// Analyze an event, tracking backlinks, and return the event as 'static
    pub fn analyze(&mut self, event: Event) -> Event<'static> {
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
        event_to_static(event)
    }

    /// Get the backlinks map
    pub fn get_backlinks(&self) -> &HashMap<String, Vec<String>> {
        &self.backlinks
    }
}

