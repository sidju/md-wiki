mod common;
use common::WikiTestFixture;

#[test]
fn test_backlinks() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_markdown_file("page1.md", "# Page 1\n\nThis links to [Page 2](page2.md).")
        .add_markdown_file("page2.md", "# Page 2\n\nThis is the target page.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that page2.html has a backlink to page1
    fixture.assert_output_contains("page2.html", "Linked from:");
    fixture.assert_output_contains("page2.html", "page1.html");
}

#[test]
fn test_deduplicated_backlinks() {
    let fixture = WikiTestFixture::new();
    
    fixture
        .add_markdown_file("page1.md", "# Page 1\n\nThis links to [Page 2](page2.md) and also to [Page 2 again](page2.md).")
        .add_markdown_file("page2.md", "# Page 2\n\nThis is the target page.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that page2.html has a backlink to page1 only once
    fixture.assert_output_contains("page2.html", "Linked from:");
    
    // Count occurrences of page1.html in the output
    let count = fixture.count_in_output("page2.html", "page1.html");
    assert_eq!(count, 1, "page1 should appear exactly once in backlinks, but appeared {} times", count);
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
    // aaa.html should still have the backlink to zzz
    fixture.assert_output_contains("aaa.html", "Linked from:");
    fixture.assert_output_contains("aaa.html", "zzz.html");
}
