mod common;
use common::WikiTestFixture;

#[test]
fn test_convert_simple_markdown() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_header("<html><body>")
        .add_footer("</body></html>")
        .add_markdown_file("test.md", "# Hello World\n\nThis is a test.");
    
    fixture.convert().expect("Conversion should succeed");
    
    fixture.assert_output_exists("test.html");
    fixture.assert_output_contains("test.html", "<html><body>");
    fixture.assert_output_contains("test.html", "Hello World");
    fixture.assert_output_contains("test.html", "</body></html>");
}
