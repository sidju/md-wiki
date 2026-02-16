mod common;
use common::WikiTestFixture;

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
fn test_directory_structure_preserved() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_file("docs/guide.md", "# Guide\n\nContent here.")
        .add_file("notes/2024/january.md", "# January Notes\n\nNotes here.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that directory structure is preserved
    fixture.assert_output_exists("docs/guide.html");
    fixture.assert_output_exists("notes/2024/january.html");
}
