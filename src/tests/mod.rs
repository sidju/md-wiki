use crate::filesystem::MockFileSystem;
use std::path::PathBuf;

pub mod conversion;
pub mod backlinks;
pub mod files;
pub mod search;
pub mod headings;
pub mod hashtags;
pub mod categories;
pub mod ignore;

/// Test fixture for setting up a mock filesystem with common test data
pub struct WikiTestFixture {
    pub fs: MockFileSystem,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl WikiTestFixture {
    /// Create a new test fixture with default paths
    pub fn new() -> Self {
        Self {
            fs: MockFileSystem::new(),
            input_dir: PathBuf::from("/input"),
            output_dir: PathBuf::from("/output"),
        }
    }

    /// Create a new test fixture with custom paths
    pub fn with_paths(input_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            fs: MockFileSystem::new(),
            input_dir: input_dir.into(),
            output_dir: output_dir.into(),
        }
    }

    /// Add a markdown file to the mock filesystem
    pub fn add_markdown_file(&self, name: &str, content: &str) -> &Self {
        let path = self.input_dir.join(name);
        self.fs.add_file(path, content);
        self
    }

    /// Add a non-markdown file to the mock filesystem
    pub fn add_file(&self, name: &str, content: &str) -> &Self {
        let path = self.input_dir.join(name);
        self.fs.add_file(path, content);
        self
    }

    /// Add a header.html template
    pub fn add_header(&self, content: &str) -> &Self {
        let path = self.input_dir.join("header.html");
        self.fs.add_file(path, content);
        self
    }

    /// Add a footer.html template
    pub fn add_footer(&self, content: &str) -> &Self {
        let path = self.input_dir.join("footer.html");
        self.fs.add_file(path, content);
        self
    }

    /// Run the wiki conversion
    pub fn convert(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::convert_wiki(
            &self.fs,
            self.input_dir.to_str().unwrap(),
            self.output_dir.to_str().unwrap(),
            None,
            &[], // No ignore patterns for existing tests
        )
    }

    /// Run the wiki conversion with search index
    pub fn convert_with_search(&self, index_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        crate::convert_wiki(
            &self.fs,
            self.input_dir.to_str().unwrap(),
            self.output_dir.to_str().unwrap(),
            Some(index_filename),
            &[], // No ignore patterns for existing tests
        )
    }

    /// Run the wiki conversion with custom ignore patterns
    pub fn convert_with_ignore(&self, ignore_patterns: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        crate::convert_wiki(
            &self.fs,
            self.input_dir.to_str().unwrap(),
            self.output_dir.to_str().unwrap(),
            None,
            ignore_patterns,
        )
    }

    /// Get the content of an output file
    pub fn get_output(&self, name: &str) -> Option<String> {
        let path = self.output_dir.join(name);
        self.fs.get_file(&path)
    }

    /// Assert that an output file exists and contains expected content
    pub fn assert_output_contains(&self, filename: &str, expected: &str) {
        let content = self.get_output(filename)
            .unwrap_or_else(|| panic!("Output file {} not found", filename));
        assert!(
            content.contains(expected),
            "Expected output file {} to contain '{}', but got:\n{}",
            filename,
            expected,
            content
        );
    }

    /// Assert that an output file exists
    pub fn assert_output_exists(&self, filename: &str) {
        assert!(
            self.get_output(filename).is_some(),
            "Expected output file {} to exist",
            filename
        );
    }

    /// Assert that an output file does not exist
    pub fn assert_output_not_exists(&self, filename: &str) {
        assert!(
            self.get_output(filename).is_none(),
            "Expected output file {} to not exist",
            filename
        );
    }

    /// Assert that content does NOT contain a substring
    pub fn assert_output_not_contains(&self, filename: &str, unexpected: &str) {
        let content = self.get_output(filename)
            .unwrap_or_else(|| panic!("Output file {} not found", filename));
        assert!(
            !content.contains(unexpected),
            "Expected output file {} to NOT contain '{}', but it did",
            filename,
            unexpected
        );
    }

    /// Count occurrences of a substring in an output file
    pub fn count_in_output(&self, filename: &str, substring: &str) -> usize {
        self.get_output(filename)
            .map(|content| content.matches(substring).count())
            .unwrap_or(0)
    }
}

impl Default for WikiTestFixture {
    fn default() -> Self {
        Self::new()
    }
}
