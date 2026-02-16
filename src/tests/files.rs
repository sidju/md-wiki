use super::WikiTestFixture;


#[test]
fn test_copy_non_md_files() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("test.md", "# Test Page\n\nContent here.")
        .add_file("style.css", "body { color: red; }")
        .add_file("script.js", "console.log('test');")
        .add_file("assets/image.txt", "fake image data");

    fixture.convert().expect("Conversion should succeed");

    // Check that HTML file was created
    fixture.assert_output_exists("test.html");

    // Check that non-markdown files were copied
    fixture.assert_output_exists("style.css");
    fixture.assert_output_exists("script.js");
    fixture.assert_output_exists("assets/image.txt");

    // Verify content of copied files
    let css_content = fixture.get_output("style.css").unwrap();
    assert_eq!(css_content, "body { color: red; }");
}

#[test]
fn test_markdown_files_in_subdirectories_copied_not_processed() {
    // Markdown files in subdirectories should be copied as-is, not processed
    let fixture = WikiTestFixture::new();

    fixture
        .add_file("docs/guide.md", "# Guide\n\nContent here.")
        .add_markdown_file("index.md", "# Index\n\nRoot file.");

    // Should succeed (with warning to stderr)
    fixture.convert().expect("Should succeed with warning");

    // Root markdown file should be processed to HTML
    fixture.assert_output_exists("index.html");
    let index_html = fixture.get_output("index.html").unwrap();
    assert!(index_html.contains("<h1"), "Root file should be processed to HTML");

    // Subdirectory markdown file should be copied as-is (not processed)
    fixture.assert_output_exists("docs/guide.md");
    let guide_md = fixture.get_output("docs/guide.md").unwrap();
    assert_eq!(guide_md, "# Guide\n\nContent here.", "Nested .md should be copied as-is");

    // Should NOT create guide.html
    assert!(fixture.get_output("docs/guide.html").is_none(),
            "Nested .md files should not be processed to HTML");
}
