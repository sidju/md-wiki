use super::WikiTestFixture;


#[test]
fn test_heading_anchors() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("page1.md", "# Main Heading\n\nLink to [section](#sub-heading).\n\n## Sub Heading\n\nLink to [other page](page2.md#another-heading).")
        .add_markdown_file("page2.md", "# Another Heading\n\nContent here.");

    fixture.convert().expect("Conversion should succeed");

    // Check that headings have IDs and links work in correct order
    let page1_expected = r###"<h1 id="main-heading">Main Heading</h1>
<p>Link to <a href="#sub-heading">section</a>.</p>
<h2 id="sub-heading">Sub Heading</h2>
<p>Link to <a href="page2.html#another-heading">other page</a>.</p>"###;

    fixture.assert_output_contains("page1.html", page1_expected);
    fixture.assert_output_contains("page2.html", r###"<h1 id="another-heading">Another Heading</h1>"###);
}

#[test]
fn test_custom_heading_ids() {
    let fixture = WikiTestFixture::new();

    fixture.add_markdown_file("custom.md", "# Auto Generated\n\nThis gets auto ID.\n\n# Custom ID {#my-custom-id}\n\nThis has custom ID.\n\n## Another Auto {#also-custom}");

    fixture.convert().expect("Conversion should succeed");

    // Check that headings have correct IDs in order
    let content = fixture.get_output("custom.html").unwrap();

    // Verify all IDs are present
    assert!(content.contains(r###"id="auto-generated""###), "Should have auto-generated ID");
    assert!(content.contains(r###"id="my-custom-id""###), "Should have custom ID");
    assert!(content.contains(r###"id="also-custom""###), "Should have custom ID for h2");

    // Should NOT have auto-generated IDs for headings with custom IDs
    assert!(!content.contains(r###"id="custom-id""###), "Should not auto-generate when custom ID exists");
}
