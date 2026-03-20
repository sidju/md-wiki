use comrak::Anchorizer;
use comrak::html::collect_text;
use comrak::nodes::{AstNode, NodeValue};
use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};
use crate::hashtag_parser;

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

/// Analyzer that tracks backlinks and categories by analyzing the AST
pub struct Analyzer {
    /// Maps from target HTML filename to a set of source HTML filenames that link to it
    backlinks: BTreeMap<String, BTreeSet<String>>,
    /// Maps from category name to a set of HTML filenames that belong to that category
    categories: BTreeMap<String, BTreeSet<String>>,
    /// Current source file being processed (HTML filename)
    current_file: String,
    /// Tracks headings for the current file
    current_headings: Vec<Heading>,
    /// All documents with their headings
    documents: Vec<Document>,
}

impl Analyzer {
    /// Create a new Analyzer for tracking backlinks and categories
    pub fn new() -> Self {
        Self {
            backlinks: BTreeMap::new(),
            categories: BTreeMap::new(),
            current_file: String::new(),
            current_headings: Vec::new(),
            documents: Vec::new(),
        }
    }

    /// Set the current file being processed
    pub fn set_current_file(&mut self, filename: String) {
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

    /// Analyze an AST, tracking backlinks, categories, and headings.
    pub fn analyze_ast<'a>(&mut self, root: &'a AstNode<'a>) {
        let mut anchorizer = Anchorizer::new();

        for node in root.descendants() {
            match node.data().value {
                NodeValue::Link(ref link) => {
                    let url = &link.url;
                    let base_link = if let Some(pos) = url.find('#') {
                        &url[..pos]
                    } else {
                        url.as_str()
                    };
                    let is_relative =
                        !base_link.contains("://") && !base_link.starts_with("//");
                    if is_relative && base_link.ends_with(".html") {
                        self.backlinks
                            .entry(base_link.to_string())
                            .or_default()
                            .insert(self.current_file.clone());
                    }
                }
                NodeValue::Text(ref text) => {
                    let text_str = text.as_ref();
                    hashtag_parser::parse_hashtags(text_str, |_start, _end, category| {
                        self.categories
                            .entry(category.to_string())
                            .or_default()
                            .insert(self.current_file.clone());
                    });
                }
                NodeValue::Heading(_) => {
                    let text = collect_text(node);
                    let id = anchorizer.anchorize(&text);
                    self.current_headings.push(Heading { text, id });
                }
                _ => {}
            }
        }
    }

    /// Get the backlinks map
    pub fn get_backlinks(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.backlinks
    }

    /// Get the categories map
    pub fn get_categories(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.categories
    }

    /// Finalize analysis and return search index
    pub fn finalize(mut self) -> SearchIndex {
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

