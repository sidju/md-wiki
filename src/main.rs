mod analyzer;
mod link_translator;
mod html_aggregator;
mod filesystem;

use clap::Parser as ClapParser;
use pulldown_cmark::{Options, Parser};
use std::path::{Path, PathBuf};

use analyzer::Analyzer;
use link_translator::translate_link;
use html_aggregator::HtmlAggregator;
use filesystem::{FileSystem, RealFileSystem};

/// A minimal static wiki generator using markdown files as input
#[derive(ClapParser, Debug)]
#[command(name = "md-wiki")]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory containing markdown files
    input_directory: String,

    /// Directory where HTML files will be created
    #[arg(default_value = ".")]
    output_directory: String,

    /// Optional path where search index will be written
    #[arg(long = "search-index")]
    search_index: Option<String>,
}

fn main() {
    let args = Args::parse();

    let fs = RealFileSystem;
    match convert_wiki(&fs, &args.input_directory, &args.output_directory, args.search_index.as_deref()) {
        Ok(_) => println!("Successfully converted markdown files to HTML"),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn convert_wiki<FS: FileSystem>(fs: &FS, input_dir: &str, output_dir: &str, search_index_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // Load header and footer
    let header_path = Path::new(input_dir).join("header.html");
    let footer_path = Path::new(input_dir).join("footer.html");
    
    let header = if fs.exists(&header_path) {
        fs.read_to_string(&header_path)?
    } else {
        String::from("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"UTF-8\">\n<title>Wiki</title>\n</head>\n<body>\n")
    };
    
    let footer = if fs.exists(&footer_path) {
        fs.read_to_string(&footer_path)?
    } else {
        String::from("</body>\n</html>\n")
    };

    // Find all files and separate them into markdown and other files
    let mut markdown_files: Vec<PathBuf> = Vec::new();
    let mut other_files: Vec<PathBuf> = Vec::new();
    
    let all_files = fs.walk_dir(Path::new(input_dir))?;
    for path in all_files {
        // Skip header.html and footer.html as they are templates
        // Note: Files with non-UTF8 names won't be skipped even if they're actually
        // named header.html or footer.html, but this is an extremely rare edge case
        // and such files would fail to process correctly anyway
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if file_name == "header.html" || file_name == "footer.html" {
            continue;
        }
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            markdown_files.push(path);
        } else {
            other_files.push(path);
        }
    }

    // Create output directory if it doesn't exist
    fs.create_dir_all(Path::new(output_dir))?;

    // Create analyzer to track backlinks across all files
    let mut analyzer = Analyzer::new();

    // Convert each markdown file to HTML using streaming design
    for md_path in &markdown_files {
        let content = fs.read_to_string(md_path)?;
        
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

        // Calculate the relative path from input_dir to preserve directory structure
        let relative_path = md_path.strip_prefix(input_dir)?;
        let output_path = Path::new(output_dir).join(relative_path).with_extension("html");
        
        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs.create_dir_all(parent)?;
        }
        
        fs.write(&output_path, &final_html)?;
        
        println!("Created: {}", output_path.display());
    }

    // Copy all non-.md files (preserving directory structure)
    for file_path in &other_files {
        let relative_path = file_path.strip_prefix(input_dir)?;
        let dest_path = Path::new(output_dir).join(relative_path);
        
        // Create parent directory if needed
        if let Some(parent) = dest_path.parent() {
            fs.create_dir_all(parent)?;
        }
        
        fs.copy(file_path, &dest_path)?;
        println!("Copied: {}", dest_path.display());
    }

    // Optionally generate search index if path is provided
    if let Some(index_path) = search_index_path {
        let search_index = analyzer.finalize();
        let index_json = serde_json::to_string_pretty(&search_index)?;
        
        // Create search-data.js with embedded index
        let search_data_content = format!("window.SEARCH_INDEX_DATA = {};", index_json);
        fs.write(Path::new(index_path), &search_data_content)?;
        println!("Created: {}", index_path);
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
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
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
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
        ).unwrap();

        // Check that page2.html has a backlink to page1
        let page2_content = fs::read_to_string(output_dir.join("page2.html")).unwrap();
        assert!(page2_content.contains("Linked from:"));
        assert!(page2_content.contains("page1.html"));

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_deduplicated_backlinks() {
        let test_dir = std::env::temp_dir().join("md-wiki-dedup-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create a page that links to another page multiple times
        fs::write(
            input_dir.join("page1.md"),
            "# Page 1\n\nThis links to [Page 2](page2.md) and also to [Page 2 again](page2.md).",
        )
        .unwrap();
        
        fs::write(
            input_dir.join("page2.md"),
            "# Page 2\n\nThis is the target page.",
        )
        .unwrap();

        // Convert
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
        ).unwrap();

        // Check that page2.html has a backlink to page1 only once
        let page2_content = fs::read_to_string(output_dir.join("page2.html")).unwrap();
        assert!(page2_content.contains("Linked from:"));
        
        // Count occurrences of page1.html in backlinks section
        let backlinks_start = page2_content.find("Linked from:").unwrap();
        let backlinks_section = &page2_content[backlinks_start..];
        let count = backlinks_section.matches("page1.html").count();
        assert_eq!(count, 1, "page1 should appear exactly once in backlinks, but appeared {} times", count);

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
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
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
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
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

        // Convert with search index
        let search_data_path = output_dir.join("search-data.js");
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            Some(search_data_path.to_str().unwrap())
        ).unwrap();

        // Check that search-data.js exists
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
            .expect("search-data.js should start with 'window.SEARCH_INDEX_DATA = '")
            .strip_suffix(";")
            .expect("search-data.js should end with ';'");
        let search_data_json: serde_json::Value = serde_json::from_str(data_match).unwrap();
        
        assert!(search_data_json.get("documents").is_some(), 
                "Parsed data should have documents field");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_no_search_index_by_default() {
        let test_dir = std::env::temp_dir().join("md-wiki-no-search-test");
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
            "# Test Page\n\nContent here.",
        )
        .unwrap();

        // Convert without search index
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
        ).unwrap();

        // Check that search-data.js does NOT exist
        let search_data_path = output_dir.join("search-data.js");
        assert!(!search_data_path.exists(), "search-data.js should not be created when path is None");

        // But the HTML file should exist
        assert!(output_dir.join("test.html").exists(), "HTML file should be created");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_copy_non_md_files() {
        let test_dir = std::env::temp_dir().join("md-wiki-copy-test");
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
            "# Test Page\n\nContent here.",
        )
        .unwrap();

        // Create non-markdown files
        fs::write(
            input_dir.join("style.css"),
            "body { color: red; }",
        )
        .unwrap();
        
        fs::write(
            input_dir.join("script.js"),
            "console.log('test');",
        )
        .unwrap();

        // Create a subdirectory with files
        fs::create_dir_all(&input_dir.join("assets")).unwrap();
        fs::write(
            input_dir.join("assets").join("image.txt"),
            "fake image data",
        )
        .unwrap();

        // Convert
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
        ).unwrap();

        // Check that HTML file was created
        assert!(output_dir.join("test.html").exists(), "HTML file should be created");

        // Check that non-markdown files were copied
        assert!(output_dir.join("style.css").exists(), "CSS file should be copied");
        assert!(output_dir.join("script.js").exists(), "JS file should be copied");
        assert!(output_dir.join("assets").join("image.txt").exists(), "Subdirectory file should be copied");

        // Verify content of copied files
        let css_content = fs::read_to_string(output_dir.join("style.css")).unwrap();
        assert_eq!(css_content, "body { color: red; }");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_directory_structure_preserved() {
        let test_dir = std::env::temp_dir().join("md-wiki-structure-test");
        let input_dir = test_dir.join("input");
        let output_dir = test_dir.join("output");

        // Clean up if exists
        let _ = fs::remove_dir_all(&test_dir);

        // Create test directories
        fs::create_dir_all(&input_dir).unwrap();
        fs::create_dir_all(&output_dir).unwrap();

        // Create markdown files in subdirectories
        fs::create_dir_all(&input_dir.join("docs")).unwrap();
        fs::write(
            input_dir.join("docs").join("guide.md"),
            "# Guide\n\nContent here.",
        )
        .unwrap();

        fs::create_dir_all(&input_dir.join("notes").join("2024")).unwrap();
        fs::write(
            input_dir.join("notes").join("2024").join("january.md"),
            "# January Notes\n\nNotes here.",
        )
        .unwrap();

        // Convert
        convert_wiki(&RealFileSystem, 
            input_dir.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            None
        ).unwrap();

        // Check that directory structure is preserved
        assert!(output_dir.join("docs").join("guide.html").exists(), 
                "docs/guide.html should exist");
        assert!(output_dir.join("notes").join("2024").join("january.html").exists(), 
                "notes/2024/january.html should exist");

        // Clean up
        fs::remove_dir_all(&test_dir).unwrap();
    }
}
