mod analyzer;
mod link_translator;
mod hashtag_linker;
mod html_aggregator;
pub mod filesystem;

use clap::Parser as ClapParser;
use pulldown_cmark::{Options, Parser};
use std::path::{Path, PathBuf};

use analyzer::Analyzer;
use link_translator::translate_link;
use hashtag_linker::linkify_hashtags;
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

/// Convert a directory of markdown files to HTML
fn convert_wiki<FS: FileSystem>(
    fs: &FS,
    input_dir: &str,
    output_dir: &str,
    search_index_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
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
            // Check if markdown file is in a subdirectory
            let relative_path = path.strip_prefix(input_dir)?;
            if relative_path.parent().is_some_and(|p| p.as_os_str() != "") {
                // Warn and treat as regular file (copy without processing)
                eprintln!("Warning: Markdown files should be in the source directory root.");
                eprintln!("  Found: {} - copying without processing", relative_path.display());
                other_files.push(path);
            } else {
                // Markdown file in root - process it
                markdown_files.push(path);
            }
        } else {
            other_files.push(path);
        }
    }

    // Create output directory if it doesn't exist
    fs.create_dir_all(Path::new(output_dir))?;

    // Create analyzer to track backlinks across all files
    let mut analyzer = Analyzer::new();

    // FIRST PASS: Analyze all markdown files and generate HTML (but don't write yet)
    let mut generated_html: Vec<(PathBuf, String, String)> = Vec::new();
    
    for md_path in &markdown_files {
        let content = fs.read_to_string(md_path)?;
        
        // Since all markdown files are in root, we can use just the filename
        let file_name = md_path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Non-UTF8 filename")?
            .to_string();
        
        // Set current file in analyzer (using HTML filename)
        let html_filename = file_name.replace(".md", ".html");
        analyzer.set_current_file(html_filename.clone());
        
        // Parse markdown with options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        let parser = Parser::new_ext(&content, options);
        
        // Stream events through the processing pipeline:
        // 1. Translate links (.md -> .html)
        // 2. Analyze (track backlinks, categories, and headings) - must see original hashtags
        // 3. Linkify hashtags (expand text events into link events)
        // 4. Aggregate to HTML
        let html_content = {
            let aggregator = parser
                .map(translate_link)
                .map(|event| analyzer.analyze(event))
                .flat_map(linkify_hashtags)
                .fold(HtmlAggregator::new(), |aggregator, event| {
                    aggregator.ingest(event)
                });
            
            aggregator.to_html_string()
        };
        
        // Store the generated HTML and HTML filename for second pass
        generated_html.push((md_path.clone(), html_filename, html_content));
    }

    // SECOND PASS: Add backlinks and write files (now all backlinks are complete)
    let mut existing_pages = std::collections::HashSet::new();
    
    for (md_path, html_file_name, html_content) in generated_html {
        // Build category pages list if this page is a category
        let mut category_pages_html = String::new();
        let category_name = html_file_name.trim_end_matches(".html");
        if let Some(pages) = analyzer.get_categories().get(category_name) {
            if !pages.is_empty() {
                category_pages_html.push_str("<hr>\n<h2>Pages in this category:</h2>\n<ul>\n");
                for page in pages {
                    let page_stem = page.trim_end_matches(".html");
                    category_pages_html.push_str(&format!(
                        "<li><a href=\"{}\">{}</a></li>\n",
                        page, page_stem
                    ));
                }
                category_pages_html.push_str("</ul>\n");
            }
        }
        
        // Build backlinks section (now all backlinks are available)
        let mut backlinks_html = String::new();
        if let Some(links) = analyzer.get_backlinks().get(&html_file_name)
            && !links.is_empty() {
                backlinks_html.push_str("<hr>\n<h2>Linked from:</h2>\n<ul>\n");
                for link in links {
                    let link_stem = link.trim_end_matches(".html");
                    backlinks_html.push_str(&format!(
                        "<li><a href=\"{}\">{}</a></li>\n",
                        link, link_stem
                    ));
                }
                backlinks_html.push_str("</ul>\n");
            }

        // Combine header, content, category pages, backlinks, and footer
        let mut final_html = String::new();
        final_html.push_str(&header);
        final_html.push_str(&html_content);
        final_html.push_str(&category_pages_html);
        final_html.push_str(&backlinks_html);
        final_html.push_str(&footer);

        // Calculate the relative path from input_dir to preserve directory structure
        let relative_path = md_path.strip_prefix(input_dir)?;
        let output_path = Path::new(output_dir).join(relative_path).with_extension("html");
        
        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs.create_dir_all(parent)?;
        }
        
        fs.write(&output_path, &final_html)?;
        
        // Track this page as existing
        existing_pages.insert(html_file_name);
        
        println!("Created: {}", output_path.display());
    }

    // THIRD PASS: Create category pages that don't already exist
    for (category_name, pages) in analyzer.get_categories() {
        let category_html_name = format!("{}.html", category_name);
        
        // Only create if this category page doesn't exist
        if !existing_pages.contains(&category_html_name) && !pages.is_empty() {
            // Build category page content
            let mut category_content = String::new();
            category_content.push_str(&format!("<h1>{}</h1>\n", category_name));
            
            // Add pages in this category
            category_content.push_str("<hr>\n<h2>Pages in this category:</h2>\n<ul>\n");
            for page in pages {
                let page_stem = page.trim_end_matches(".html");
                category_content.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    page, page_stem
                ));
            }
            category_content.push_str("</ul>\n");
            
            // Add backlinks section if any
            let mut backlinks_html = String::new();
            if let Some(links) = analyzer.get_backlinks().get(&category_html_name) {
                if !links.is_empty() {
                    backlinks_html.push_str("<hr>\n<h2>Linked from:</h2>\n<ul>\n");
                    for link in links {
                        let link_stem = link.trim_end_matches(".html");
                        backlinks_html.push_str(&format!(
                            "<li><a href=\"{}\">{}</a></li>\n",
                            link, link_stem
                        ));
                    }
                    backlinks_html.push_str("</ul>\n");
                }
            }
            
            // Combine header, content, backlinks, and footer
            let mut final_html = String::new();
            final_html.push_str(&header);
            final_html.push_str(&category_content);
            final_html.push_str(&backlinks_html);
            final_html.push_str(&footer);
            
            // Write the category page
            let output_path = Path::new(output_dir).join(&category_html_name);
            fs.write(&output_path, &final_html)?;
            println!("Created category page: {}", output_path.display());
        }
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
mod tests;
