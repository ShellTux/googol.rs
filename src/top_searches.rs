//! Tracks and retrieves the most searched keywords.
//!
//! # Example:
//!
//! ```rust
//! use googol::top_searches::TopSearches;
//!
//! let mut ts = TopSearches::new();
//! ts.add_search("rust");
//! let top = ts.top_n(3);
//! ```

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

/// Maintains a collection of search keywords and provides functionality
/// to retrieve the most frequently searched terms.
///
/// # Overview
///
/// `TopSearches` tracks the number of times each search term has been added,
/// allowing you to query for the top `n` most searched keywords.
///
/// # Examples
///
/// ```rust
/// use googol::top_searches::TopSearches;
///
/// let mut searches = TopSearches::new();
/// searches.add_search("rust");
/// searches.add_search("rust");
/// searches.add_search("programming");
/// let top = searches.top_n(2);
/// assert_eq!(top, vec![("rust".to_string(), 2), ("programming".to_string(), 1)]);
/// ```
///
/// # Thread Safety
///
/// Not thread-safe. For concurrent use, consider wrapping in synchronization primitives.
#[derive(Debug, Default)]
pub struct TopSearches {
    /// Maps search keywords to their respective counts.
    counts: HashMap<String, usize>,
}

impl TopSearches {
    /// Creates a new, empty `TopSearches` instance.
    ///
    /// # Returns
    ///
    /// A new `TopSearches` with no recorded searches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use googol::top_searches::TopSearches;
    ///
    /// let searches = TopSearches::new();
    /// ```
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Records a new search for the given `word`.
    ///
    /// Increments the count for the specified search term.
    ///
    /// # Arguments
    ///
    /// * `word` - The search term to record.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use googol::top_searches::TopSearches;
    ///
    /// let mut searches = TopSearches::new();
    /// assert_eq!(searches.count("rust"), 0);
    /// searches.add_search("rust");
    /// assert_eq!(searches.count("rust"), 1);
    /// searches.add_search("rust");
    /// assert_eq!(searches.count("rust"), 2);
    /// ```
    pub fn add_search(&mut self, word: &str) {
        *self.counts.entry(word.to_string()).or_insert(0) += 1;
    }

    /// Returns the number of times the given `word` has been searched.
    ///
    /// # Arguments
    ///
    /// * `word` - The search term to query.
    ///
    /// # Returns
    ///
    /// The count of how many times `word` has been added.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use googol::top_searches::TopSearches;
    ///
    /// let mut searches = TopSearches::new();
    /// searches.add_search("rust");
    /// searches.add_search("rust");
    /// assert_eq!(searches.count("rust"), 2);
    /// assert_eq!(searches.count("programming"), 0);
    /// ```
    pub fn count(&self, word: &str) -> usize {
        self.counts.get(word).cloned().unwrap_or(0)
    }

    /// Retrieves the top `n` most searched keywords along with their counts.
    ///
    /// The results are sorted in descending order of count.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of top entries to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `(keyword, count)` tuples for the top `n` searches.
    ///
    /// # Example
    ///
    /// ```rust
    /// use googol::top_searches::TopSearches;
    ///
    /// let mut searches = TopSearches::new();
    /// searches.add_search("rust");
    /// searches.add_search("rust");
    /// searches.add_search("programming");
    /// let top = searches.top_n(2);
    /// assert_eq!(top, vec![("rust".to_string(), 2), ("programming".to_string(), 1)]);
    /// ```
    pub fn top_n(&self, n: usize) -> Vec<(String, usize)> {
        // Use a min-heap to keep track of top n counts
        let mut heap: BinaryHeap<Reverse<(usize, String)>> = BinaryHeap::new();

        for (keyword, &count) in &self.counts {
            if heap.len() < n {
                // Fill the heap initially
                heap.push(Reverse((count, keyword.clone())));
            } else if let Some(&Reverse((min_count, _))) = heap.peek() {
                if count > min_count {
                    // Replace the smallest in the heap if current count is higher
                    heap.pop();
                    heap.push(Reverse((count, keyword.clone())));
                }
            }
        }

        // Collect the heap into a vector
        let mut result: Vec<(usize, String)> = heap
            .into_iter()
            .map(|Reverse((count, keyword))| (count, keyword))
            .collect();

        // Sort in descending order of counts
        result.sort_by(|a, b| b.0.cmp(&a.0));

        // Convert to (keyword, count) tuples
        result
            .into_iter()
            .map(|(count, keyword)| (keyword, count))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_searches() {
        let mut searches = TopSearches::new();

        searches.add_search("rust");
        searches.add_search("rust");
        searches.add_search("programming");
        searches.add_search("language");
        searches.add_search("rust");
        searches.add_search("performance");
        searches.add_search("code");
        searches.add_search("code");
        searches.add_search("code");
        searches.add_search("performance");
        searches.add_search("performance");
        searches.add_search("performance");

        let top_searches = searches.top_n(3);
        let expected_top_searches: Vec<(String, usize)> =
            [("performance", 4), ("code", 3), ("rust", 3)]
                .iter()
                .map(|(word, count)| (word.to_string(), *count))
                .collect();

        assert_eq!(top_searches, expected_top_searches);
    }

    #[test]
    fn test_count() {
        let mut searches = TopSearches::new();

        assert_eq!(searches.count("rust"), 0);

        searches.add_search("rust");
        searches.add_search("rust");
        searches.add_search("programming");

        // Check counts
        assert_eq!(searches.count("rust"), 2);
        assert_eq!(searches.count("programming"), 1);
        assert_eq!(searches.count("language"), 0);
    }
}
