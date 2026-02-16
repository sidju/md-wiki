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
