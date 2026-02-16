use super::WikiTestFixture;

#[test]
fn test_hashtag_linkification() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Page with hashtags\n\nThis page has #test and #example hashtags.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that hashtags are converted to links
    fixture.assert_output_contains("page.html", r###"<a href="test.html">#test</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="example.html">#example</a>"###);
}

#[test]
fn test_hashtag_with_hyphens_and_underscores() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nCategories: #knowledge-management #test_case");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that hashtags with hyphens and underscores are properly linked
    fixture.assert_output_contains("page.html", r###"<a href="knowledge-management.html">#knowledge-management</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="test_case.html">#test_case</a>"###);
}

#[test]
fn test_hashtag_at_start_of_line() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\n#start-of-line is a hashtag.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Hashtag at start of line should be linkified
    fixture.assert_output_contains("page.html", r###"<a href="start-of-line.html">#start-of-line</a>"###);
}

#[test]
fn test_hashtag_after_punctuation() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nCategories: (#inparentheses) and [#inbrackets] and {#inbraces}");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Hashtags after opening brackets/parentheses should be linkified
    fixture.assert_output_contains("page.html", r###"<a href="inparentheses.html">#inparentheses</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="inbrackets.html">#inbrackets</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="inbraces.html">#inbraces</a>"###);
}

#[test]
fn test_hashtag_not_in_word() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nThis has word#nothashtag in it.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Hashtag in middle of word should NOT be linkified
    fixture.assert_output_not_contains("page.html", r###"<a href="nothashtag.html">"###);
    // Original text should remain
    fixture.assert_output_contains("page.html", "word#nothashtag");
}

#[test]
fn test_multiple_hashtags_in_same_line() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nTags: #first #second #third");
    
    fixture.convert().expect("Conversion should succeed");
    
    // All hashtags should be linkified
    fixture.assert_output_contains("page.html", r###"<a href="first.html">#first</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="second.html">#second</a>"###);
    fixture.assert_output_contains("page.html", r###"<a href="third.html">#third</a>"###);
}

#[test]
fn test_empty_hashtag_not_linkified() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nThis has a bare # symbol.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Empty hashtag should not be converted to a link
    fixture.assert_output_not_contains("page.html", r###"<a href=".html">"###);
}

#[test]
fn test_hashtag_stops_at_non_alphanumeric() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page.md", "# Test\n\nThis has #tag, #tag. #tag! #tag?");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Hashtags should stop at punctuation
    let count = fixture.count_in_output("page.html", r###"<a href="tag.html">#tag</a>"###);
    assert_eq!(count, 4, "Should have exactly 4 instances of #tag linked");
}
