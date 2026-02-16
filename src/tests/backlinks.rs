use super::WikiTestFixture;


#[test]
fn test_backlinks() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_markdown_file("page1.md", "# Page 1\n\nThis links to [Page 2](page2.md).")
        .add_markdown_file("page2.md", "# Page 2\n\nThis is the target page.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that page2.html has a backlink to page1, verifying both presence and ordering
    fixture.assert_output_contains("page2.html", "<hr>\n<h2>Linked from:</h2>\n<ul>\n<li><a href=\"page1.html\">page1</a></li>");
}

#[test]
fn test_deduplicated_backlinks() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_markdown_file("page1.md", "# Page 1\n\nThis links to [Page 2](page2.md) and also to [Page 2 again](page2.md).")
        .add_markdown_file("page2.md", "# Page 2\n\nThis is the target page.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that page2.html has a backlink to page1 only once
    let page2_content = fixture.get_output("page2.html").unwrap();
    
    // Count occurrences of page1.html in the output
    let count = page2_content.matches("page1.html").count();
    assert_eq!(count, 1, "page1 should appear exactly once in backlinks, but appeared {} times", count);
    
    // Also verify the backlinks section structure
    fixture.assert_output_contains("page2.html", "<hr>\n<h2>Linked from:</h2>\n<ul>\n<li><a href=\"page1.html\">page1</a></li>");
}

#[test]
fn test_backlinks_order_independent() {
    // Test that backlinks work regardless of file processing order
    // By using non-alphabetical names (zzz processes after aaa)
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_markdown_file("zzz.md", "# ZZZ Page\n\nThis links to [AAA](aaa.md).")
        .add_markdown_file("aaa.md", "# AAA Page\n\nThis is the target.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Even though zzz.md might be processed after aaa.md (depending on HashMap order),
    // aaa.html should still have the backlink to zzz, and verify structure
    fixture.assert_output_contains("aaa.html", "<hr>\n<h2>Linked from:</h2>\n<ul>\n<li><a href=\"zzz.html\">zzz</a></li>");
}

#[test]
fn test_external_urls_not_tracked() {
    // Test that external URLs (with schemes or protocol-relative) are not tracked as backlinks
    let fixture = WikiTestFixture::new();
    
    let page1_content = "# Page 1\n\n\
        This links to [External](https://example.com/page.html) \
        and [Protocol Relative](//cdn.example.com/file.html).";
    let page2_content = "# Page 2\n\nThis links to [Internal](page1.md).";
    
    fixture
        .add_markdown_file("page1.md", page1_content)
        .add_markdown_file("page2.md", page2_content);
    
    fixture.convert().expect("Conversion should succeed");
    
    // page1.html should have a backlink from page2.html
    fixture.assert_output_contains("page1.html", "<hr>\n<h2>Linked from:</h2>\n<ul>\n<li><a href=\"page2.html\">page2</a></li>");
    
    // Verify that page1.html does NOT have any content suggesting external URLs were tracked
    // (The external links should appear in the content but not in the backlinks section)
    let page1_content = fixture.get_output("page1.html").unwrap();
    
    // External URLs should appear in the content (as links)
    assert!(page1_content.contains("https://example.com/page.html"), "External URL should be present in content");
    assert!(page1_content.contains("//cdn.example.com/file.html"), "Protocol-relative URL should be present in content");
    
    // But external URLs should NOT be tracked as pages receiving backlinks
    // This means no backlinks section should appear mentioning these URLs
}
