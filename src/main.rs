mod analyzer;
mod link_translator;
mod html_aggregator;

use pulldown_cmark::{Options, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use analyzer::Analyzer;
use link_translator::translate_link;
use html_aggregator::HtmlAggregator;

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

    // Create analyzer to track backlinks across all files
    let mut analyzer = Analyzer::new();

    // Convert each markdown file to HTML using streaming design
    for md_path in &markdown_files {
        let content = fs::read_to_string(md_path)?;
        
        let file_name = md_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        
        // Set current file in analyzer
        analyzer.set_current_file(file_name.clone());
        
        // Parse markdown with options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        let parser = Parser::new_ext(&content, options);
        
        // Stream events through the processing pipeline:
        // 1. Analyze (track backlinks)
        // 2. Translate links (.md -> .html)
        // 3. Ingest into HTML aggregator (converts to 'static and handles lookahead for heading IDs)
        let html_content = {
            let aggregator = parser
                .map(|event| analyzer.analyze(event))
                .map(translate_link)
                .fold(HtmlAggregator::new(), |aggregator, event| {
                    aggregator.ingest(event)
                });
            
            aggregator.to_html_string()
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
        if let Some(links) = analyzer.get_backlinks().get(&md_file_name)
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

    // Finalize analyzer and generate search index
    let search_index = analyzer.finalize();
    let index_json = serde_json::to_string_pretty(&search_index)?;
    
    // Create search-data.js with embedded index
    let search_data_content = format!("window.SEARCH_INDEX_DATA = {};", index_json);
    let search_data_path = Path::new(output_dir).join("search-data.js");
    fs::write(&search_data_path, search_data_content)?;
    println!("Created: {}", search_data_path.display());
    
    // Copy resources folder if it exists
    let resources_src = Path::new(input_dir).join("resources");
    if resources_src.exists() && resources_src.is_dir() {
        let resources_dst = Path::new(output_dir).join("resources");
        fs::create_dir_all(&resources_dst)?;
        
        for entry in WalkDir::new(&resources_src) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let relative_path = path.strip_prefix(&resources_src)?;
                let dest_path = resources_dst.join(relative_path);
                
                // Create parent directory if needed
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::copy(path, &dest_path)?;
                println!("Copied: {}", dest_path.display());
            }
        }
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

    #[test]
    fn test_custom_heading_ids() {
        let test_dir = std::env::temp_dir().join("md-wiki-custom-ids-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown file with custom heading IDs
        fs::write(
            input_dir.join("custom.md"),
            "# Auto Generated\n\nThis gets auto ID.\n\n# Custom ID {#my-custom-id}\n\nThis has custom ID.\n\n## Another Auto {#also-custom}",
        )
        .unwrap();

        // Convert
        convert_wiki(
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()
        ).unwrap();

        // Check that headings have correct IDs
        let content = fs::read_to_string(output_dir.join("custom.html")).unwrap();
        
        // Auto-generated ID from text
        assert!(content.contains(r#"id="auto-generated""#), "Should have auto-generated ID");
        
        // Custom IDs from markdown
        assert!(content.contains(r#"id="my-custom-id""#), "Should have custom ID");
        assert!(content.contains(r#"id="also-custom""#), "Should have custom ID for h2");
        
        // Should NOT have auto-generated IDs for headings with custom IDs
        assert!(!content.contains(r#"id="custom-id""#), "Should not auto-generate when custom ID exists");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_search_data_generation() {
        let test_dir = std::env::temp_dir().join("md-wiki-search-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create test markdown file with headings
        fs::write(
            input_dir.join("page1.md"),
            "# Main Heading\n\nContent here.\n\n## Sub Heading\n\nMore content.",
        )
        .unwrap();

        // Convert
        convert_wiki(
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap()
        ).unwrap();

        // Check that search-data.js exists
        let search_data_path = output_dir.join("search-data.js");
        assert!(search_data_path.exists(), "search-data.js should be created");

        // Verify search-data.js contains the expected structure
        let search_data_content = fs::read_to_string(&search_data_path).unwrap();
        assert!(search_data_content.starts_with("window.SEARCH_INDEX_DATA = {"), 
                "search-data.js should set window.SEARCH_INDEX_DATA");
        assert!(search_data_content.contains("\"documents\""), 
                "search-data.js should contain documents");
        assert!(search_data_content.contains("page1.html"), 
                "search-data.js should reference page1.html");
        assert!(search_data_content.contains("Main Heading"), 
                "search-data.js should contain heading text");
        assert!(search_data_content.contains("Sub Heading"), 
                "search-data.js should contain all headings");

        // Verify the data can be parsed as valid JSON
        let data_match = search_data_content
            .strip_prefix("window.SEARCH_INDEX_DATA = ")
            .and_then(|s| s.strip_suffix(";"))
            .unwrap();
        let search_data_json: serde_json::Value = serde_json::from_str(data_match).unwrap();
        
        assert!(search_data_json.get("documents").is_some(), 
                "Parsed data should have documents field");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }
}
