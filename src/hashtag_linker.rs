use std::borrow::Cow;
use comrak::Arena;
use comrak::nodes::{AstNode, NodeValue, NodeLink};
use comrak::nodes::AstNode as Node;
use crate::hashtag_parser;

/// Convert hashtags in text nodes to clickable link nodes in the AST
pub fn linkify_hashtags<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>) {
    // Collect text nodes that contain '#' (avoid mutating while iterating)
    let text_nodes: Vec<&'a AstNode<'a>> = root
        .descendants()
        .filter(|node| matches!(&node.data().value, NodeValue::Text(t) if t.contains('#')))
        .collect();

    for node in text_nodes {
        let text = match node.data().value {
            NodeValue::Text(ref t) => t.to_string(),
            _ => continue,
        };

        let mut hashtags: Vec<(usize, usize, String)> = Vec::new();
        hashtag_parser::parse_hashtags(&text, |start, end, tag| {
            hashtags.push((start, end, tag.to_string()));
        });

        if hashtags.is_empty() {
            continue;
        }

        let mut last_end = 0;
        for (start, end, tag) in &hashtags {
            if *start > last_end {
                let before = arena.alloc(Node::from(NodeValue::Text(
                    Cow::Owned(text[last_end..*start].to_string()),
                )));
                node.insert_before(before);
            }

            let link_node = arena.alloc(Node::from(NodeValue::Link(Box::new(NodeLink {
                url: format!("{}.html", tag),
                title: String::new(),
            }))));
            let link_text = arena.alloc(Node::from(NodeValue::Text(
                Cow::Owned(format!("#{}", tag)),
            )));
            link_node.append(link_text);
            node.insert_before(link_node);

            last_end = *end;
        }

        if last_end < text.len() {
            node.data_mut().value =
                NodeValue::Text(Cow::Owned(text[last_end..].to_string()));
        } else {
            node.detach();
        }
    }
}
