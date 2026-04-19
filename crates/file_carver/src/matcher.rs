/// Searches for a fixed byte pattern within a byte slice.
///
/// `PatternMatcher` is created once for a given pattern and can then search
/// for it across multiple byte slices without repeating the pattern as an
/// argument on every call.
pub(crate) struct PatternMatcher<'a> {
    pattern: &'a [u8],
}

impl<'a> PatternMatcher<'a> {
    /// Creates a new `PatternMatcher` that borrows `pattern`.
    pub fn new(pattern: &'a [u8]) -> Self {
        Self { pattern }
    }

    /// Returns the index of the first occurrence of the pattern in `buffer`
    /// starting the search at `from`.
    ///
    /// Returns `None` if the pattern is empty or not found.
    pub fn find_in(&self, buffer: &[u8], from: usize) -> Option<usize> {
        if self.pattern.is_empty() {
            return None;
        }

        let last_start = buffer.len().saturating_sub(self.pattern.len());

        if from > last_start {
            return None;
        }

        for offset in from..=last_start {
            if buffer[offset..offset + self.pattern.len()] == *self.pattern {
                return Some(offset);
            }
        }

        None
    }

    /// Returns the starting indices of **all** occurrences of the pattern in
    /// `buffer`, advancing one byte at a time so that overlapping matches
    /// are also reported.
    ///
    /// Returns an empty `Vec` if the pattern is empty or not found.
    pub fn find_all_in(&self, buffer: &[u8]) -> Vec<usize> {
        if self.pattern.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let mut search_from = 0;

        while let Some(found_at) = self.find_in(buffer, search_from) {
            matches.push(found_at);
            search_from = found_at + 1;
        }

        matches
    }
}
