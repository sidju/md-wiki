use super::WikiTestFixture;


#[test]
fn test_convert_simple_markdown() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_header("<html><body>")
        .add_footer("</body></html>")
        .add_markdown_file("test.md", "# Hello World\n\nThis is a test.");

    fixture.convert().expect("Conversion should succeed");

    fixture.assert_output_exists("test.html");
    // Check ordering of header, content, and footer in one assertion
    fixture.assert_output_contains("test.html", "<html><body><h1><a href=\"#hello-world\" aria-hidden=\"true\" class=\"anchor\" id=\"hello-world\"></a>Hello World</h1>\n<p>This is a test.</p>\n</body></html>");
}
