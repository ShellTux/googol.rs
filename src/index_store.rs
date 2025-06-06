//! Manages storage, retrieval, and search of indexed web pages.
//!
//! Provides functions to store pages, search by keywords, and persist data to disk.
//!
//! # Example:
//!
//! ```rust
//! use googol::{index_store::IndexStore, page::Page};
//! use url::Url;
//!
//! let mut index_store = IndexStore::default();
//! // Sample pages and their data
//! let page1 = Page::create("https://example.com/page1").with_title("Page One");
//! let words1 = ["rust", "programming", "language"]
//!     .iter()
//!     .map(|w| w.to_string())
//!     .collect();
//! let outlinks_for_page1 = ["https://link1.com", "https://link2.com"]
//!     .iter()
//!     .map(|u| Url::parse(u).unwrap())
//!     .collect();
//! index_store.store(&page1, &words1, &outlinks_for_page1);
//! ```
//!
//! Supports loading existing index data from files.

use crate::page::Page;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use url::Url;

/// An index storage structure for managing web pages, their links, and search indices.
///
/// The `IndexStore` maintains collections of pages, their associated URLs, inverted indices for search,
/// backlink and outlink relationships, and handles persistence to disk.
///
/// # Examples
///
/// Creating a new `IndexStore` with a specified file path:
///
/// ```rust
/// use googol::index_store::IndexStore;
///
/// let index = IndexStore::new("index_data.json");
/// ```
///
/// Loading an existing index from disk:
///
/// ```rust
/// use googol::index_store::IndexStore;
///
/// let index_result = IndexStore::load("index_data.json");
/// ```
///
/// Storing a page in the index:
///
/// ```rust
/// use url::Url;
/// use googol::{index_store::IndexStore, page::Page};
///
/// let mut store = IndexStore::new("index_data.json");
/// let page = Page::create("https://example.com")
///     .with_title("Example Page");
/// let words = ["example", "page"].iter().map(|w| w.to_string()).collect();
/// let outlinks = ["https://linked.com"].iter().map(|u| Url::parse(u).unwrap()).collect();
/// store.store(&page, &words, &outlinks);
/// ```
///
/// Saving the index to disk:
///
/// ```rust
/// use googol::index_store::IndexStore;
/// use std::fs;
///
/// let mut store = IndexStore::new("index_data.json");
/// match store.save() {
///     Ok(bytes) => println!("Saved {} bytes", bytes),
///     Err(e) => println!("Error saving index: {}", e),
/// }
/// fs::remove_file("index_data.json").expect("Failed to delete temp file");
/// ```
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexStore {
    /// Set of all indexed pages.
    indexed_pages: HashSet<Page>,
    /// Map from URL to Page.
    url2pages: HashMap<Url, Page>,

    /// Forward index: word (lowercase) to set of URLs containing the word.
    index: HashMap<String, HashSet<Url>>,
    /// Inverse index: URL to set of words associated with the page.
    invert_index: HashMap<Url, HashSet<String>>,

    /// Map from URL to set of URLs linking **to** the page (backlinks).
    backlinks: HashMap<Url, HashSet<Url>>,
    /// Map from URL to set of URLs that the page links out to (outlinks).
    outlinks: HashMap<Url, HashSet<Url>>,

    /// Filesystem path for storing the index data.
    #[serde(skip)]
    filepath: PathBuf,
    /// Size of the serialized index in bytes.
    #[serde(skip)]
    size_bytes: usize,
}

impl IndexStore {
    /// Creates a new `IndexStore` with the specified file path.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path where index data will be stored.
    ///
    /// # Returns
    ///
    /// An `IndexStore` instance with the specified filepath.
    pub fn new<P>(filepath: P) -> Self
    where
        P: AsRef<Path>,
    {
        let mut index_store = Self::default();

        index_store.filepath = filepath.as_ref().to_path_buf();

        index_store
    }

    /// Loads an `IndexStore` from disk at the given path.
    ///
    /// If the file does not exist or cannot be read, it initializes a new `IndexStore`.
    ///
    /// # Arguments
    ///
    /// * `filepath` - Path to the JSON file containing serialized `IndexStore`.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if file operations or deserialization fail.
    pub fn load<P>(filepath: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        match File::open(&filepath) {
            Ok(mut file) => {
                let mut json_str = String::new();

                let size = file.read_to_string(&mut json_str)?;
                let mut index_store: IndexStore = serde_json::from_str(&json_str).map_err(|e| {
                    use io::{Error, ErrorKind};

                    Error::new(ErrorKind::Other, format!("Deserialization error: {}", e))
                })?;

                index_store.filepath = filepath.as_ref().to_path_buf();
                index_store.size_bytes = size;

                Ok(index_store)
            }
            Err(e) => {
                error!("Error opening file {:?}: {}", filepath.as_ref().to_str(), e);
                Ok(Self::new(&filepath))
            }
        }
    }

    /// Searches for pages containing all the specified words.
    ///
    /// The search is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `words` - A vector of words to search for.
    ///
    /// # Returns
    ///
    /// A set of `Page` instances matching all words. Empty if no matches or input is empty.
    pub fn search(&self, words: &Vec<String>) -> HashSet<Page> {
        if words.is_empty() {
            return HashSet::new();
        }

        // Collect URL sets for each word (case-insensitive)
        let sets_of_urls: Vec<&HashSet<Url>> = words
            .iter()
            .map(|w| w.to_lowercase())
            .filter_map(|word| self.index.get(&word))
            .collect();

        // If any word isn't found, no pages contain all words
        if sets_of_urls.len() < words.len() {
            return HashSet::new();
        }

        // Intersect all URL sets to find common pages
        let intersection_urls = sets_of_urls
            .iter()
            .skip(1)
            .fold(sets_of_urls[0].clone(), |acc, set| &acc & set);

        // Convert URLs to Pages
        intersection_urls
            .iter()
            .filter_map(|url| self.url2pages.get(url))
            .cloned()
            .collect()
    }

    /// Searches for pages matching all words and sorts them by their backlink count (descending).
    ///
    /// The most backlinks (popularity) pages appear first.
    ///
    /// # Arguments
    ///
    /// * `words` - A vector of words to search for.
    ///
    /// # Returns
    ///
    /// A vector of `Page` sorted by relevance (backlink count).
    pub fn search_by_relevance(&self, words: &Vec<String>) -> Vec<Page> {
        let pages = self.search(words);

        let mut pages_with_backlinks: Vec<(Page, usize)> = pages
            .into_iter()
            .map(|page| {
                let backlink_count = self.backlinks.get(&page.url).map_or(0, |s| s.len());
                (page, backlink_count)
            })
            .collect();

        // Sort descending by backlink count
        pages_with_backlinks.sort_by(|(_, a_size), (_, b_size)| b_size.cmp(&a_size));

        pages_with_backlinks
            .into_iter()
            .map(|(page, _)| page)
            .collect()
    }

    /// Stores a page and its associated data into the index.
    ///
    /// Updates the inverted index, backlink relationships, and outlinks.
    ///
    /// # Arguments
    ///
    /// * `page` - The `Page` to store.
    /// * `words` - Words associated with the page.
    /// * `outlinks` - Outgoing links from the page.
    pub fn store(&mut self, page: &Page, words: &Vec<String>, outlinks: &Vec<Url>) {
        self.indexed_pages.insert(page.clone());
        self.url2pages.insert(page.url.clone(), page.clone());

        for word in words.iter().map(|word| word.to_lowercase()) {
            self.index
                .entry(word.clone())
                .or_insert_with(HashSet::new)
                .insert(page.url.clone());

            self.invert_index
                .entry(page.url.clone())
                .or_insert_with(HashSet::new)
                .insert(word.clone());
        }

        self.outlinks
            .entry(page.url.clone())
            .or_insert_with(HashSet::new)
            .extend(outlinks.iter().cloned());

        for outlink in outlinks {
            self.backlinks
                .entry(outlink.clone())
                .or_insert_with(HashSet::new)
                .insert(page.url.clone());
        }
    }

    /// Retrieves all backlinks (pages linking to the given URL).
    ///
    /// # Arguments
    ///
    /// * `url` - The URL for which to retrieve backlinks.
    ///
    /// # Returns
    ///
    /// A set of URLs linking to the given URL. Empty if none.
    pub fn consult_backlinks(&self, url: &Url) -> HashSet<Url> {
        self.backlinks.get(url).cloned().unwrap_or_default()
    }

    /// Retrieves all outlinks (pages linked from the given URL).
    ///
    /// # Arguments
    ///
    /// * `url` - The URL for which to retrieve outlinks.
    ///
    /// # Returns
    ///
    /// A set of URLs that the page links to. Empty if none.
    pub fn consult_outlinks(&self, url: &Url) -> HashSet<Url> {
        self.outlinks.get(url).cloned().unwrap_or_default()
    }

    /// Saves the current index to disk.
    ///
    /// Serializes the index to JSON and writes it to the specified filepath.
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if serialization or file writing fails.
    pub fn save(&mut self) -> Result<usize, io::Error> {
        let json = serde_json::to_string(self).map_err(|e| {
            use io::{Error, ErrorKind};

            Error::new(ErrorKind::Other, format!("Serialization error: {}", e))
        })?;

        // WARN: filepath could be invalid
        // dbg!(&self.filepath);

        match File::create(&self.filepath)?.write(json.as_bytes()) {
            Ok(size) => {
                self.size_bytes = size;
                Ok(size)
            }
            Err(e) => {
                error!(
                    "Failed to write to file {}: {}",
                    &self.filepath.display(),
                    e
                );
                Err(e)
            }
        }
    }
}

/// Tests for `IndexStore` functionalities.
#[cfg(test)]
mod tests {
    use crate::url::parse_url_panic;

    use super::*;
    use std::{collections::HashSet, fs, path};
    use url::Url;

    /// Helper to create a Page with optional title.
    fn create_page(url: &str, title: Option<&str>) -> Page {
        let mut page = Page::create(url);

        if let Some(t) = title {
            page = page.with_title(t);
        }

        page
    }

    /// Helper to initialize an index with sample data.
    fn create_index_store() -> IndexStore {
        let mut index_store = IndexStore::default();

        // Sample pages and their data
        let page1 = Page::create("https://example.com/page1").with_title("Page One");
        let words1 = ["rust", "programming", "language"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let outlinks_for_page1 = ["https://link1.com", "https://link2.com"]
            .iter()
            .map(|u| Url::parse(u).unwrap())
            .collect();
        index_store.store(&page1, &words1, &outlinks_for_page1);

        let page2 = Page::create("https://example.com/page2").with_title("Page Two");
        let words2 = ["rust", "web"].iter().map(|w| w.to_string()).collect();
        let outlinks_for_page2 = ["https://link3.com"].iter().map(parse_url_panic).collect();
        index_store.store(&page2, &words2, &outlinks_for_page2);

        let page3 = Page::create("https://example.com/page3").with_title("Page Three");
        let words3 = ["programming", "tutorial"]
            .iter()
            .map(|w| w.to_string())
            .collect();
        let outlinks_for_page3 = [
            "https://link4.com",
            "https://link5.com",
            "https://link6.com",
        ]
        .iter()
        .map(parse_url_panic)
        .collect();
        index_store.store(&page3, &words3, &outlinks_for_page3);

        // Add backlinks for testing search_by_relevance
        index_store.backlinks.insert(
            page1.url,
            ["https://link1.com", "https://link2.com"]
                .iter()
                .map(parse_url_panic)
                .collect(),
        );
        index_store.backlinks.insert(
            page2.url,
            ["https://link3.com"].iter().map(parse_url_panic).collect(),
        );
        index_store.backlinks.insert(
            page3.url,
            [
                "https://link4.com",
                "https://link5.com",
                "https://link6.com",
            ]
            .iter()
            .map(parse_url_panic)
            .collect(),
        );

        index_store
    }

    #[test]
    fn test_search_single_word() {
        let index_store = create_index_store();

        let results = index_store.search(&vec!["rust".to_string()]);
        let urls: HashSet<Url> = results.iter().map(|p| p.url.clone()).collect();

        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&Url::parse("https://example.com/page1").unwrap()));
        assert!(urls.contains(&Url::parse("https://example.com/page2").unwrap()));
    }

    #[test]
    fn test_search_multiple_words() {
        let index_store = create_index_store();

        // Search for pages containing both "rust" and "programming"
        let results = index_store.search(
            &["rust", "programming"]
                .iter()
                .map(|w| w.to_string())
                .collect(),
        );
        let urls: HashSet<_> = results.iter().map(|p| p.url.clone()).collect();

        assert_eq!(urls.len(), 1);
        assert!(urls.contains(&Url::parse("https://example.com/page1").unwrap()));
    }

    #[test]
    fn test_search_no_match() {
        let index_store = create_index_store();

        let results = index_store.search(&vec!["nonexistent".to_string()]);
        assert!(results.is_empty());

        let results2 = index_store.search(&vec!["rust".to_string(), "nonexistent".to_string()]);
        assert!(results2.is_empty());
    }

    #[test]
    fn test_search_empty_input() {
        let index_store = create_index_store();

        let results = index_store.search(&vec![]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_case_insensitivity() {
        let index_store = create_index_store();

        let results_lower = index_store.search(&vec!["rust".to_string()]);
        let results_upper = index_store.search(&vec!["RUST".to_string()]);

        let urls_lower: HashSet<_> = results_lower.iter().map(|p| p.url.clone()).collect();
        let urls_upper: HashSet<_> = results_upper.iter().map(|p| p.url.clone()).collect();

        assert_eq!(urls_lower, urls_upper);
    }

    #[test]
    fn test_search_by_relevance() {
        let index_store = create_index_store();

        // Search for pages containing "rust"
        let sorted_pages = index_store.search_by_relevance(&vec!["rust".to_string()]);

        // Expect pages sorted by backlinks: page3 (3), page1 (2), page2 (1)
        assert_eq!(sorted_pages.len(), 2);
        let urls: Vec<_> = sorted_pages.iter().map(|p| p.url.clone()).collect();

        assert_eq!(urls[0], Url::parse("https://example.com/page1").unwrap());
        assert_eq!(urls[1], Url::parse("https://example.com/page2").unwrap());

        // Check ordering by backlink count
        let backlink_counts: Vec<_> = sorted_pages
            .iter()
            .map(|p| index_store.backlinks.get(&p.url).map_or(0, |s| s.len()))
            .collect();

        assert!(backlink_counts.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn test_search_by_relevance_empty_results() {
        let index_store = create_index_store();

        // Search for non-existent words
        let results = index_store.search_by_relevance(&vec!["nonexistent".to_string()]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_relevance_no_backlinks() {
        let mut index_store = create_index_store();

        // Create a page with no backlinks
        let page_no_backlinks = create_page("https://example.com/page4", Some("Page Four"));
        let words = vec!["tutorial"].iter().map(|w| w.to_string()).collect();
        index_store.store(&page_no_backlinks, &words, &vec![]);

        // Now search for "tutorial", which matches page3 and page4
        let results = index_store.search_by_relevance(&vec!["tutorial".to_string()]);
        // Page3 has backlinks, page4 has none
        assert_eq!(results.len(), 2);
        assert_eq!(
            results[0].url,
            Url::parse("https://example.com/page3").unwrap()
        );
    }

    #[test]
    fn test_save_and_load() {
        let mut store = create_index_store();
        let temp_path = ".test_index_store.json";
        store.filepath = path::absolute(temp_path).unwrap();
        dbg!(&store);

        // Save the store
        let save_result = store.save();
        dbg!(&save_result);
        assert!(save_result.is_ok());
        let saved_size = save_result.unwrap();
        assert!(saved_size > 0);

        // Read the file content directly
        let file_content = fs::read_to_string(temp_path).expect("Failed to read temp file");
        assert!(!file_content.is_empty());

        // Now load into a new IndexStore
        let loaded_store = IndexStore::load(temp_path);
        assert!(loaded_store.is_ok());

        let loaded_store = loaded_store.unwrap();

        // Check that loaded data contains the same pages
        assert_eq!(loaded_store.indexed_pages.len(), store.indexed_pages.len());

        // Check that a known page exists
        let url = Url::parse("https://example.com/page1").unwrap();
        assert!(loaded_store.url2pages.contains_key(&url));
        let page = &loaded_store.url2pages[&url];
        assert_eq!(page.title.as_deref(), Some("Page One"));

        // Cleanup the temp file
        fs::remove_file(temp_path).expect("Failed to delete temp file");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let nonexistent_path = "nonexistent_file.json";
        let result = IndexStore::load(nonexistent_path);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, IndexStore::new(nonexistent_path));
    }

    #[test]
    fn test_save_error_handling() {
        // Create a store with an invalid path to trigger write error
        let mut store = create_index_store();
        store.filepath = path::absolute("/invalid_path/test.json").unwrap();

        // Save should fail
        let result = store.save();
        assert!(result.is_err());
    }
}
