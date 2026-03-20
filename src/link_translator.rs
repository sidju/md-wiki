use comrak::nodes::{AstNode, NodeValue};

/// Translate .md links to .html links in the AST
pub fn translate_links<'a>(root: &'a AstNode<'a>) {
    for node in root.descendants() {
        let url = match node.data().value {
            NodeValue::Link(ref link) => Some(link.url.clone()),
            NodeValue::Image(ref link) => Some(link.url.clone()),
            _ => None,
        };

        if let Some(url) = url {
            let new_url = translate_url(&url);
            match node.data_mut().value {
                NodeValue::Link(ref mut link) => link.url = new_url,
                NodeValue::Image(ref mut link) => link.url = new_url,
                _ => {}
            }
        }
    }
}

fn translate_url(url: &str) -> String {
    if let Some(hash_pos) = url.find('#') {
        let (path, fragment) = url.split_at(hash_pos);
        if path.ends_with(".md") {
            return format!("{}{}", &path[..path.len() - 3], ".html") + fragment;
        }
    } else if url.ends_with(".md") {
        return format!("{}.html", &url[..url.len() - 3]);
    }
    url.to_string()
}
