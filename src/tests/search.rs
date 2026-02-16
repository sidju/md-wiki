use super::WikiTestFixture;


#[test]
fn test_search_data_generation() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("page1.md", "# Main Heading\n\nContent here.\n\n## Sub Heading\n\nMore content.");
    
    fixture.convert_with_search("/output/search-data.js").expect("Conversion should succeed");
    
    // Check that search-data.js exists
    fixture.assert_output_exists("search-data.js");
    
    // Verify search-data.js contains the expected structure
    let search_data_content = fixture.get_output("search-data.js").unwrap();
    assert!(search_data_content.starts_with("window.SEARCH_INDEX_DATA = {"), 
            "search-data.js should set window.SEARCH_INDEX_DATA");
    assert!(search_data_content.contains("\"documents\""), 
            "search-data.js should contain documents");
    assert!(search_data_content.contains("page1.html"), 
            "search-data.js should reference page1.html");
    assert!(search_data_content.contains("Main Heading"), 
            "search-data.js should contain heading text");
    assert!(search_data_content.contains("Sub Heading"), 
            "search-data.js should contain all headings");
    
    // Verify the data can be parsed as valid JSON
    let data_match = search_data_content
        .strip_prefix("window.SEARCH_INDEX_DATA = ")
        .expect("search-data.js should start with 'window.SEARCH_INDEX_DATA = '")
        .strip_suffix(";")
        .expect("search-data.js should end with ';'");
    let _: serde_json::Value = serde_json::from_str(data_match)
        .expect("search data should be valid JSON");
}

#[test]
fn test_no_search_index_by_default() {
    let fixture = WikiTestFixture::new();
    
    fixture.add_markdown_file("test.md", "# Test Page\n\nContent here.");
    
    fixture.convert().expect("Conversion should succeed");
    
    // Check that search-data.js does NOT exist
    fixture.assert_output_not_exists("search-data.js");
    
    // But the HTML file should exist
    fixture.assert_output_exists("test.html");
}
