/// Shared hashtag parsing utilities
///
/// This module provides common functionality for detecting and parsing hashtags
/// in markdown text, used by both the analyzer and hashtag linker.

/// Parse hashtags from text and call a callback for each valid hashtag found
///
/// A valid hashtag:
/// - Starts with # at the beginning of text or preceded by whitespace or '(', '[', '{'
/// - Followed by one or more alphanumeric characters, underscores, or hyphens
/// - Must have at least one character after the #
///
/// The callback receives (start_idx, end_idx, category_name) for each hashtag found
pub fn parse_hashtags<F>(text: &str, mut callback: F)
where
    F: FnMut(usize, usize, &str),
{
    for (idx, _) in text.match_indices('#') {
        // Check if # is at start or preceded by whitespace or one of '(', '[', '{'
        let valid_prefix = if idx == 0 {
            true
        } else {
            // Find the character before # (respecting UTF-8 boundaries)
            let mut prev_idx = idx - 1;
            while prev_idx > 0 && !text.is_char_boundary(prev_idx) {
                prev_idx -= 1;
            }
            if let Some(prev_char) = text[prev_idx..idx].chars().next() {
                prev_char.is_whitespace() || matches!(prev_char, '(' | '[' | '{')
            } else {
                false
            }
        };

        if valid_prefix {
            // Extract category name after #
            let after_hash = &text[idx + 1..];
            let category_end = after_hash
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
                .unwrap_or(after_hash.len());

            let category = &after_hash[..category_end];

            if !category.is_empty() {
                let hashtag_end = idx + 1 + category_end;
                callback(idx, hashtag_end, category);
            }
        }
    }
}
