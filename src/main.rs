use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

    // Build link graph: for each file, track which files link to it
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^\)]+\.md)\)")?;

    for md_path in &markdown_files {
        let content = fs::read_to_string(md_path)?;
        let file_name = md_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        for cap in link_regex.captures_iter(&content) {
            if let Some(linked_file) = cap.get(2) {
                let linked = linked_file.as_str().to_string();
                backlinks
                    .entry(linked)
                    .or_insert_with(Vec::new)
                    .push(file_name.clone());
            }
        }
    }

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)?;

    // Convert each markdown file to HTML
    for md_path in &markdown_files {
        let content = fs::read_to_string(md_path)?;
        
        // Replace .md links with .html links in markdown content
        let content_with_html_links = link_regex.replace_all(&content, |caps: &regex::Captures| {
            let text = &caps[1];
            let link = &caps[2];
            let html_link = link.replace(".md", ".html");
            format!("[{}]({})", text, html_link)
        });
        
        // Convert markdown to HTML
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&content_with_html_links, options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);

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
        if let Some(links) = backlinks.get(&md_file_name) {
            if !links.is_empty() {
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
    use std::path::Path;

    #[test]
    fn test_convert_simple_markdown() {
        let test_dir = "/tmp/md-wiki-test";
        let input_dir = format!("{}/input", test_dir);
        let output_dir = format!("{}/output", test_dir);

        // Clean up if exists
        let _ = fs::remove_dir_all(test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown file
        fs::write(
            format!("{}/test.md", input_dir),
            "# Hello World\n\nThis is a test.",
        )
        .unwrap();

        // Create header and footer
        fs::write(
            format!("{}/header.html", input_dir),
            "<html><body>",
        )
        .unwrap();
        fs::write(
            format!("{}/footer.html", input_dir),
            "</body></html>",
        )
        .unwrap();

        // Convert
        convert_wiki(&input_dir, &output_dir).unwrap();

        // Check output exists
        let output_path = format!("{}/test.html", output_dir);
        assert!(Path::new(&output_path).exists());

        // Check content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<html><body>"));
        assert!(content.contains("Hello World"));
        assert!(content.contains("</body></html>"));

        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }

    #[test]
    fn test_backlinks() {
        let test_dir = "/tmp/md-wiki-backlinks-test";
        let input_dir = format!("{}/input", test_dir);
        let output_dir = format!("{}/output", test_dir);

        // Clean up if exists
        let _ = fs::remove_dir_all(test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown files
        fs::write(
            format!("{}/page1.md", input_dir),
            "# Page 1\n\nThis links to [Page 2](page2.md).",
        )
        .unwrap();
        
        fs::write(
            format!("{}/page2.md", input_dir),
            "# Page 2\n\nThis is the target page.",
        )
        .unwrap();

        // Convert
        convert_wiki(&input_dir, &output_dir).unwrap();

        // Check that page2.html has a backlink to page1
        let page2_content = fs::read_to_string(format!("{}/page2.html", output_dir)).unwrap();
        assert!(page2_content.contains("Linked from:"));
        assert!(page2_content.contains("page1.html"));

        // Clean up
        fs::remove_dir_all(test_dir).unwrap();
    }
}
