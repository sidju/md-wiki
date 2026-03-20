use super::WikiTestFixture;


#[test]
fn test_heading_anchors() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("page1.md", "# Main Heading\n\nLink to [section](#sub-heading).\n\n## Sub Heading\n\nLink to [other page](page2.md#another-heading).")
        .add_markdown_file("page2.md", "# Another Heading\n\nContent here.");

    fixture.convert().expect("Conversion should succeed");

    // Check that headings have IDs and links work in correct order
    let page1_expected = concat!(
        "<h1><a href=\"#main-heading\" aria-hidden=\"true\" class=\"anchor\" id=\"main-heading\"></a>Main Heading</h1>\n",
        "<p>Link to <a href=\"#sub-heading\">section</a>.</p>\n",
        "<h2><a href=\"#sub-heading\" aria-hidden=\"true\" class=\"anchor\" id=\"sub-heading\"></a>Sub Heading</h2>\n",
        "<p>Link to <a href=\"page2.html#another-heading\">other page</a>.</p>",
    );

    fixture.assert_output_contains("page1.html", page1_expected);
    fixture.assert_output_contains("page2.html", "<h1><a href=\"#another-heading\" aria-hidden=\"true\" class=\"anchor\" id=\"another-heading\"></a>Another Heading</h1>");
}

#[test]
fn test_heading_ids_are_auto_generated() {
    let fixture = WikiTestFixture::new();

    fixture.add_markdown_file("custom.md", "# Auto Generated\n\nThis gets auto ID.\n\n# Another Heading\n\nMore content.");

    fixture.convert().expect("Conversion should succeed");

    let content = fixture.get_output("custom.html").unwrap();
    assert!(content.contains(r###"id="auto-generated""###), "Should have auto-generated ID");
    assert!(content.contains(r###"id="another-heading""###), "Should have auto-generated ID for second heading");
}
