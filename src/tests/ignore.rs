use super::WikiTestFixture;

#[test]
fn test_ignore_dotfiles_by_default() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file(".hidden_file.txt", "hidden content")
        .add_file(".git/config", "git config")
        .add_file("visible.txt", "visible content");

    // Convert with default ignore patterns (should ignore dotfiles)
    fixture.convert_with_ignore(&[String::from(".*")]).expect("Conversion should succeed");

    // Check that HTML file was created
    fixture.assert_output_exists("index.html");

    // Check that visible file was copied
    fixture.assert_output_exists("visible.txt");

    // Check that dotfiles were NOT copied
    fixture.assert_output_not_exists(".hidden_file.txt");
    fixture.assert_output_not_exists(".git/config");
}

#[test]
fn test_ignore_custom_pattern() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("keep.txt", "keep this")
        .add_file("temp.tmp", "temporary file")
        .add_file("backup.bak", "backup file");

    // Ignore files ending with .tmp or .bak
    fixture.convert_with_ignore(&[String::from("*.tmp"), String::from("*.bak")])
        .expect("Conversion should succeed");

    // Check that HTML file was created
    fixture.assert_output_exists("index.html");

    // Check that .txt file was copied
    fixture.assert_output_exists("keep.txt");

    // Check that .tmp and .bak files were NOT copied
    fixture.assert_output_not_exists("temp.tmp");
    fixture.assert_output_not_exists("backup.bak");
}

#[test]
fn test_no_ignore_patterns() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file(".hidden_file.txt", "hidden content")
        .add_file("visible.txt", "visible content");

    // Convert with no ignore patterns (should include everything)
    fixture.convert_with_ignore(&[]).expect("Conversion should succeed");

    // Check that HTML file was created
    fixture.assert_output_exists("index.html");

    // Check that ALL files were copied (including dotfiles)
    fixture.assert_output_exists("visible.txt");
    fixture.assert_output_exists(".hidden_file.txt");
}

#[test]
fn test_ignore_dotfiles_in_subdirectories() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("assets/image.png", "image data")
        .add_file("assets/.hidden.png", "hidden image")
        .add_file(".git/config", "git config");

    // Ignore dotfiles
    fixture.convert_with_ignore(&[String::from(".*")]).expect("Conversion should succeed");

    // Check that visible subdirectory file was copied
    fixture.assert_output_exists("assets/image.png");

    // Check that dotfiles in subdirectories were NOT copied
    fixture.assert_output_not_exists("assets/.hidden.png");
    fixture.assert_output_not_exists(".git/config");
}

#[test]
fn test_edge_case_patterns() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("test.txt", "test")
        .add_file("a", "single char name")
        .add_file("ab", "two char name");

    // Test with single character pattern followed by wildcard
    fixture.convert_with_ignore(&[String::from("a*")]).expect("Should handle 'a*' pattern");
    fixture.assert_output_exists("test.txt");
    fixture.assert_output_not_exists("a");
    fixture.assert_output_not_exists("ab");
}

#[test]
fn test_wildcard_only_pattern() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("file1.txt", "file1")
        .add_file("file2.txt", "file2");

    // Test with wildcard only pattern (should match everything)
    fixture.convert_with_ignore(&[String::from("*")]).expect("Should handle '*' pattern");
    
    // All non-markdown files should be ignored
    fixture.assert_output_not_exists("file1.txt");
    fixture.assert_output_not_exists("file2.txt");
}

#[test]
fn test_suffix_pattern() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("file.log", "log file")
        .add_file("data.txt", "text file");

    // Test suffix pattern
    fixture.convert_with_ignore(&[String::from("*.log")]).expect("Should handle suffix pattern");
    
    fixture.assert_output_exists("data.txt");
    fixture.assert_output_not_exists("file.log");
}

#[test]
fn test_double_wildcard_pattern() {
    let fixture = WikiTestFixture::new();

    fixture
        .add_markdown_file("index.md", "# Index\n\nMain page.")
        .add_file("file1.txt", "file1")
        .add_file("file2.txt", "file2");

    // Test with double wildcard pattern (should match everything like single wildcard)
    fixture.convert_with_ignore(&[String::from("**")]).expect("Should handle '**' pattern");
    
    // All non-markdown files should be ignored
    fixture.assert_output_not_exists("file1.txt");
    fixture.assert_output_not_exists("file2.txt");
}

#[test]
fn test_current_directory_not_filtered() {
    // Test that "." is treated as a special case and not filtered by ".*" pattern
    
    // Use "." as input directory
    let fs = crate::filesystem::MockFileSystem::new();
    fs.add_file("./index.md", "# Index\n\nMain page.");
    fs.add_file("./visible.txt", "visible content");
    fs.add_file("./.hidden.txt", "hidden content");
    
    // Convert with default ignore patterns (should not filter "." itself)
    let result = crate::convert_wiki(
        &fs,
        ".",
        "/output",
        None,
        &[String::from(".*")],
    );
    
    assert!(result.is_ok(), "Conversion should succeed with '.' as input");
    
    // Check that files were processed
    assert!(fs.get_file(&std::path::PathBuf::from("/output/index.html")).is_some(),
        "index.html should be created");
    assert!(fs.get_file(&std::path::PathBuf::from("/output/visible.txt")).is_some(),
        "visible.txt should be copied");
    assert!(fs.get_file(&std::path::PathBuf::from("/output/.hidden.txt")).is_none(),
        ".hidden.txt should still be filtered");
}

#[test]
fn test_parent_directory_not_filtered() {
    // Test that ".." is treated as a special case and not filtered by ".*" pattern
    
    // Use ".." in path components
    let fs = crate::filesystem::MockFileSystem::new();
    fs.add_file("../parent/index.md", "# Index\n\nMain page.");
    fs.add_file("../parent/visible.txt", "visible content");
    
    // Convert with default ignore patterns (should not filter ".." itself)
    let result = crate::convert_wiki(
        &fs,
        "../parent",
        "/output",
        None,
        &[String::from(".*")],
    );
    
    assert!(result.is_ok(), "Conversion should succeed with '..' in path");
    
    // Check that files were processed
    assert!(fs.get_file(&std::path::PathBuf::from("/output/index.html")).is_some(),
        "index.html should be created");
    assert!(fs.get_file(&std::path::PathBuf::from("/output/visible.txt")).is_some(),
        "visible.txt should be copied");
}
