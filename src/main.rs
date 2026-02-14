use pulldown_cmark::{html, Event, Options, Parser, Tag, CowStr};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: md-wiki <input_directory> [output_directory]");
        eprintln!("  input_directory:  Directory containing markdown files");
        eprintln!("  output_directory: Directory where HTML files will be created (default: current directory)");
        std::process::exit(1);
    }

    let input_dir = &args[1];
    let output_dir = if args.len() > 2 {
        &args[2]
    } else {
        "."
    };

    match convert_wiki(input_dir, output_dir) {
        Ok(_) => println!("Successfully converted markdown files to HTML"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn convert_wiki(input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Load header and footer
    let header_path = Path::new(input_dir).join("header.html");
    let footer_path = Path::new(input_dir).join("footer.html");
    
    let header = if header_path.exists() {
        fs::read_to_string(&header_path)?
    } else {
        String::from("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>Wiki</title>\n</head>\n<body>\n")
    };
    
    let footer = if footer_path.exists() {
        fs::read_to_string(&footer_path)?
    } else {
        String::from("</body>\n</html>\n")
    };

    // Find all markdown files
    let mut markdown_files: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(input_dir) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
            markdown_files.push(path.to_path_buf());
        }
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Build link graph: for each file, track which files link to it
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();

    // Convert each markdown file to HTML
    for md_path in &markdown_files {
        let content = fs::read_to_string(md_path)?;
        
        let file_name = md_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        
        // Parse markdown with options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&content, options);
        
        // Track links and convert to HTML in a single pass
        let html_content = {
            // First pass: track links and transform link URLs
            let mut events = Vec::new();
            let mut heading_stack: Vec<String> = Vec::new();
            
            for event in parser {
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
                        backlinks
                            .entry(filename)
                            .or_default()
                            .push(file_name.clone());
                    }
                }
                
                // Process event for HTML generation
                match event {
                    Event::Start(Tag::Heading { .. }) => {
                        heading_stack.push(String::new());
                        events.push(event);
                    }
                    Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                        heading_stack.pop();
                        events.push(event);
                    }
                    Event::Text(ref text) => {
                        if let Some(heading_text) = heading_stack.last_mut() {
                            heading_text.push_str(text);
                        }
                        events.push(event);
                    }
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
                        events.push(Event::Start(Tag::Link { link_type, dest_url: new_url, title, id }));
                    }
                    _ => {
                        events.push(event);
                    }
                }
            }
            
            // Second pass: add IDs to headings
            let mut processed_events = Vec::new();
            let mut i = 0;
            
            while i < events.len() {
                match &events[i] {
                    Event::Start(Tag::Heading { level, id, classes, attrs }) => {
                        // Look ahead to collect heading text
                        let mut current_heading_text = String::new();
                        let mut j = i + 1;
                        while j < events.len() {
                            if let Event::End(pulldown_cmark::TagEnd::Heading(_)) = events[j] {
                                break;
                            }
                            if let Event::Text(ref text) = events[j] {
                                current_heading_text.push_str(text);
                            }
                            j += 1;
                        }
                        
                        // Generate ID from heading text
                        let heading_id = if !current_heading_text.is_empty() {
                            Some(CowStr::from(slugify(&current_heading_text)))
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
                        processed_events.push(events[i].clone());
                    }
                }
                i += 1;
            }
            
            let mut html_output = String::new();
            html::push_html(&mut html_output, processed_events.into_iter());
            html_output
        };

        // Get the file name without extension
        let file_stem = md_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        
        let md_file_name = md_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Build backlinks section
        let mut backlinks_html = String::new();
        if let Some(links) = backlinks.get(&md_file_name)
            && !links.is_empty() {
                backlinks_html.push_str("<hr>\n<h2>Linked from:</h2>\n<ul>\n");
                for link in links {
                    let link_stem = link.trim_end_matches(".md");
                    backlinks_html.push_str(&format!(
                        "<li><a href=\"{}.html\">{}</a></li>\n",
                        link_stem, link_stem
                    ));
                }
                backlinks_html.push_str("</ul>\n");
            }

        // Combine header, content, backlinks, and footer
        let final_html = format!("{}{}{}{}", header, html_content, backlinks_html, footer);

        // Write to output file
        let output_path = Path::new(output_dir).join(format!("{}.html", file_stem));
        fs::write(&output_path, final_html)?;
        
        println!("Created: {}", output_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_convert_simple_markdown() {
        let test_dir = std::env::temp_dir().join("md-wiki-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown file
        fs::write(
            input_dir.join("test.md"),
            "# Hello World\n\nThis is a test.",
        )
        .unwrap();

        // Create header and footer
        fs::write(
            input_dir.join("header.html"),
            "<html><body>",
        )
        .unwrap();
        fs::write(
            input_dir.join("footer.html"),
            "</body></html>",
        )
        .unwrap();

        // Convert
        convert_wiki(
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()
        ).unwrap();

        // Check output exists
        let output_path = output_dir.join("test.html");
        assert!(output_path.exists());

        // Check content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<html><body>"));
        assert!(content.contains("Hello World"));
        assert!(content.contains("</body></html>"));

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_backlinks() {
        let test_dir = std::env::temp_dir().join("md-wiki-backlinks-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown files
        fs::write(
            input_dir.join("page1.md"),
            "# Page 1\n\nThis links to [Page 2](page2.md).",
        )
        .unwrap();
        
        fs::write(
            input_dir.join("page2.md"),
            "# Page 2\n\nThis is the target page.",
        )
        .unwrap();

        // Convert
        convert_wiki(
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()
        ).unwrap();

        // Check that page2.html has a backlink to page1
        let page2_content = fs::read_to_string(output_dir.join("page2.html")).unwrap();
        assert!(page2_content.contains("Linked from:"));
        assert!(page2_content.contains("page1.html"));

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_heading_anchors() {
        let test_dir = std::env::temp_dir().join("md-wiki-anchors-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown file with headings and anchor links
        fs::write(
            input_dir.join("page1.md"),
            "# Main Heading\n\nLink to [section](#sub-heading).\n\n## Sub Heading\n\nLink to [other page](page2.md#another-heading).",
        )
        .unwrap();
        
        fs::write(
            input_dir.join("page2.md"),
            "# Another Heading\n\nContent here.",
        )
        .unwrap();

        // Convert
        convert_wiki(
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()
        ).unwrap();

        // Check that headings have IDs
        let page1_content = fs::read_to_string(output_dir.join("page1.html")).unwrap();
        assert!(page1_content.contains(r#"id="main-heading""#));
        assert!(page1_content.contains(r#"id="sub-heading""#));
        
        // Check that internal anchor links work
        assert!(page1_content.contains(r##"href="#sub-heading""##));
        
        // Check that cross-file anchor links work
        assert!(page1_content.contains(r##"href="page2.html#another-heading""##));
        
        let page2_content = fs::read_to_string(output_dir.join("page2.html")).unwrap();
        assert!(page2_content.contains(r#"id="another-heading""#));

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }
}
