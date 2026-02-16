use pulldown_cmark::{Event, Tag, LinkType, CowStr};
use crate::hashtag_parser;

/// Convert hashtags in text to clickable links
/// Returns a vector of events that should replace the input event
pub fn linkify_hashtags<'a>(event: Event<'a>) -> Vec<Event<'a>> {
    match event {
        Event::Text(text) => {
            let text_str = text.as_ref();
            let mut events = Vec::new();
            let mut last_end = 0;

            hashtag_parser::parse_hashtags(text_str, |idx, hashtag_end, category| {
                // Add text before the hashtag if any
                if idx > last_end {
                    events.push(Event::Text(CowStr::from(text_str[last_end..idx].to_string())));
                }

                // Add link for the hashtag
                let link_text = format!("#{}", category);
                let link_url = format!("{}.html", category);

                events.push(Event::Start(Tag::Link {
                    link_type: LinkType::Inline,
                    dest_url: CowStr::from(link_url),
                    title: CowStr::from(""),
                    id: CowStr::from(""),
                }));
                events.push(Event::Text(CowStr::from(link_text)));
                events.push(Event::End(pulldown_cmark::TagEnd::Link));

                last_end = hashtag_end;
            });

            // Add any remaining text after the last hashtag
            if last_end < text_str.len() {
                events.push(Event::Text(CowStr::from(text_str[last_end..].to_string())));
            }

            events
        }
        // For all other events, just pass through as-is
        other => vec![other],
    }
}
