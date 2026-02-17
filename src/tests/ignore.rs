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
