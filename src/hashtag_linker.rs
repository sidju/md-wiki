use pulldown_cmark::{Event, Tag, LinkType, CowStr};

/// Convert hashtags in text to clickable links
/// Returns a vector of events that should replace the input event
pub fn linkify_hashtags<'a>(event: Event<'a>) -> Vec<Event<'a>> {
    match event {
        Event::Text(text) => {
            let text_str = text.as_ref();
            let mut events = Vec::new();
            let mut last_end = 0;
            let mut found_hashtag = false;
            
            for (idx, _) in text_str.match_indices('#') {
                // Check if # is at start or preceded by whitespace or opening bracket/parenthesis
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
                        found_hashtag = true;
                        
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
                        
                        last_end = idx + 1 + category_end;
                    }
                }
            }
            
            // If we found hashtags, add any remaining text
            if found_hashtag {
                if last_end < text_str.len() {
                    events.push(Event::Text(CowStr::from(text_str[last_end..].to_string())));
                }
                events
            } else {
                // No hashtags found, return original event
                vec![Event::Text(text)]
            }
        }
        // For all other events, just pass through as-is
        other => vec![other],
    }
}
