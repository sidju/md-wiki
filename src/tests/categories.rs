use super::WikiTestFixture;

#[test]
fn test_category_page_auto_generation() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("page1.md", "# Page 1\n\nThis page is about #testing.")
        .add_markdown_file("page2.md", "# Page 2\n\nThis also covers #testing.");

    fixture.convert().expect("Conversion should succeed");

    // Category page should be auto-generated
    fixture.assert_output_exists("testing.html");

    // Category page should have a title
    fixture.assert_output_contains("testing.html", "<h1>testing</h1>");

    // Category page should list pages in the category
    fixture.assert_output_contains("testing.html", "<h2>Pages in this category:</h2>");
    fixture.assert_output_contains("testing.html", r###"<a href="page1.html">page1</a>"###);
    fixture.assert_output_contains("testing.html", r###"<a href="page2.html">page2</a>"###);
}

#[test]
fn test_multiple_categories_per_page() {
    let fixture = WikiTestFixture::new();

    fixture.add_markdown_file("notes.md", "# Notes\n\nCategories: #knowledge-management #learning");

    fixture.convert().expect("Conversion should succeed");

    // Both category pages should be created
    fixture.assert_output_exists("knowledge-management.html");
    fixture.assert_output_exists("learning.html");

    // Each category page should list the notes page
    fixture.assert_output_contains("knowledge-management.html", r###"<a href="notes.html">notes</a>"###);
    fixture.assert_output_contains("learning.html", r###"<a href="notes.html">notes</a>"###);
}

#[test]
fn test_existing_page_as_category() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("tutorial.md", "# Tutorial\n\nThis is the tutorial category page.")
        .add_markdown_file("page1.md", "# Page 1\n\nTags: #tutorial")
        .add_markdown_file("page2.md", "# Page 2\n\nTags: #tutorial");

    fixture.convert().expect("Conversion should succeed");

    // Existing tutorial.html should be preserved and enhanced with category listing
    fixture.assert_output_contains("tutorial.html", "This is the tutorial category page.");

    // Category listing should be added to existing page
    fixture.assert_output_contains("tutorial.html", "<h2>Pages in this category:</h2>");
    fixture.assert_output_contains("tutorial.html", r###"<a href="page1.html">page1</a>"###);
    fixture.assert_output_contains("tutorial.html", r###"<a href="page2.html">page2</a>"###);
}

#[test]
fn test_category_page_not_created_for_unused_hashtag() {
    let fixture = WikiTestFixture::new();

    fixture.add_markdown_file("page.md", "# Page\n\nThis has no hashtags.");

    fixture.convert().expect("Conversion should succeed");

    // Only the regular page should exist
    fixture.assert_output_exists("page.html");

    // No category pages should be created - only page.html in output
    let output = fixture.get_output("page.html").unwrap();
    assert!(!output.is_empty(), "Page should have content");

    // Verify no category pages exist by checking common category names
    fixture.assert_output_not_exists("category.html");
    fixture.assert_output_not_exists("test.html");
}

#[test]
fn test_category_pages_are_sorted() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("zzz.md", "# ZZZ Page\n\n#category")
        .add_markdown_file("aaa.md", "# AAA Page\n\n#category")
        .add_markdown_file("mmm.md", "# MMM Page\n\n#category");

    fixture.convert().expect("Conversion should succeed");

    // Check that category page lists pages in sorted order
    let category_content = fixture.get_output("category.html").unwrap();

    // Find positions of each page link
    let aaa_pos = category_content.find("aaa.html").expect("Should contain aaa.html");
    let mmm_pos = category_content.find("mmm.html").expect("Should contain mmm.html");
    let zzz_pos = category_content.find("zzz.html").expect("Should contain zzz.html");

    // Verify they appear in alphabetical order
    assert!(aaa_pos < mmm_pos, "aaa should come before mmm");
    assert!(mmm_pos < zzz_pos, "mmm should come before zzz");
}

#[test]
fn test_category_backlinks() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("page.md", "# Page\n\nSee [category](category.html).")
        .add_markdown_file("another.md", "# Another\n\nTags: #category");

    fixture.convert().expect("Conversion should succeed");

    // Category page should have backlinks section from the page that linked to it
    fixture.assert_output_contains("category.html", "<h2>Linked from:</h2>");
    fixture.assert_output_contains("category.html", r###"<a href="page.html">page</a>"###);
}

#[test]
fn test_deduplication_in_categories() {
    let fixture = WikiTestFixture::new();

    fixture.add_markdown_file("page.md", "# Page\n\nTags: #test #test #test");

    fixture.convert().expect("Conversion should succeed");

    // Page should appear only once in category listing
    let category_content = fixture.get_output("test.html").unwrap();

    // Extract just the category section
    let category_section = category_content
        .split("Pages in this category:")
        .nth(1)
        .expect("Should have category section");

    let count = category_section.matches("page.html").count();
    assert_eq!(count, 1, "page.html should appear exactly once in category listing, but appeared {} times", count);
}

#[test]
fn test_category_and_backlinks_order() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("tutorial.md", "# Tutorial\n\nContent here.")
        .add_markdown_file("page.md", "# Page\n\nTags: #tutorial and link to [Tutorial](tutorial.html)");

    fixture.convert().expect("Conversion should succeed");

    // Tutorial page should have category listing before backlinks
    let tutorial_content = fixture.get_output("tutorial.html").unwrap();

    let category_pos = tutorial_content.find("Pages in this category:").expect("Should have category section");
    let backlinks_pos = tutorial_content.find("Linked from:").expect("Should have backlinks section");

    assert!(category_pos < backlinks_pos, "Category listing should appear before backlinks");
}
