use pulldown_cmark::{Event, Tag};

/// Translate .md links to .html links in markdown events
pub fn translate_link<'a>(event: Event<'a>) -> Event<'a> {
    match event {
        Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
            // Handle .md links with or without fragments
            let new_url = if let Some(hash_pos) = dest_url.find('#') {
                let (path, fragment) = dest_url.split_at(hash_pos);
                if path.ends_with(".md") {
                    format!("{}{}", path.replace(".md", ".html"), fragment).into()
                } else {
                    dest_url
                }
            } else if dest_url.ends_with(".md") {
                dest_url.replace(".md", ".html").into()
            } else {
                dest_url
            };
            Event::Start(Tag::Link { link_type, dest_url: new_url, title, id })
        }
        _ => event,
    }
}
