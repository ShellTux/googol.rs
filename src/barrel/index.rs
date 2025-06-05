use crate::{page::Page, proto};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
};
use url::Url;

trait IndexStore {
    fn into_vec(&self) -> Vec<String>;
    fn store(&mut self, page: &Page, words: &Vec<String>);
}

trait InvertedIndexStore {
    fn into_vec(&self) -> Vec<String>;
    fn store(&mut self, url: &Url, words: &Vec<String>);
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexPages(HashMap<String, HashSet<Page>>);

impl IndexPages {
    fn search(&self, words: &Vec<String>) -> Vec<String> {
        let mut result_set = HashSet::new();

        for word in words {
            let word = word.to_lowercase();

            if let Some(urls) = self.0.get(&word) {
                for url in urls {
                    result_set.insert(url.clone());
                }
            }
        }

        result_set
            .into_iter()
            .map(|page| page.url.as_str().to_string())
            .collect()
    }

    pub fn search_order_by_relevance(
        &self,
        words: &Vec<String>,
        backlinks_map: &HashMap<Url, HashSet<Url>>,
    ) -> Vec<proto::Page> {
        let mut page_counts: HashMap<Page, usize> = HashMap::new();

        for word in words {
            let word = word.to_lowercase();

            if let Some(pages) = self.0.get(&word) {
                for page in pages {
                    *page_counts.entry(page.clone()).or_insert(0) =
                        backlinks_map.get(&page.url).iter().len();
                }
            }
        }

        let mut page_count_vec: Vec<(Page, usize)> = page_counts.into_iter().collect();

        page_count_vec.sort_by(|a, b| b.1.cmp(&a.1));

        page_count_vec
            .into_iter()
            .map(|(page, _)| page.into())
            .collect()
    }
}

impl IndexStore for IndexPages {
    fn into_vec(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    fn store(&mut self, page: &Page, words: &Vec<String>) {
        for word in words {
            let word = word.to_lowercase();

            self.0
                .entry(word)
                .or_insert_with(HashSet::new)
                .insert(page.clone());
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
// WARN: Page cannot be serialized into json key
struct InvertedIndexPages(HashMap<Url, HashSet<String>>);

impl InvertedIndexStore for InvertedIndexPages {
    fn into_vec(&self) -> Vec<String> {
        self.0.keys().map(|url| url.to_string()).collect()
    }

    fn store(&mut self, url: &Url, words: &Vec<String>) {
        self.0
            .entry(url.clone())
            .or_insert_with(HashSet::new)
            .extend(words.iter().map(|word| word.to_lowercase()));
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub indexed_pages: IndexPages,
    pub inverted_indexed_pages: InvertedIndexPages,
    pub backlinks: HashMap<Url, HashSet<Url>>,
    pub outlinks: HashMap<Url, HashSet<Url>>,
    #[serde(skip)]
    pub filepath: String,
    #[serde(skip)]
    pub size_bytes: usize,
}

impl Index {
    fn indexed_pages(&self) -> Vec<String> {
        self.inverted_indexed_pages.into_vec()
    }

    fn indexed_words(&self) -> Vec<String> {
        self.indexed_pages.into_vec()
    }

    pub fn consult_backlinks(&self, url: &Url) -> Vec<String> {
        match self.backlinks.get(url) {
            Some(backlink_set) => backlink_set.iter().map(|url| url.to_string()).collect(),
            None => Vec::new(),
        }
    }

    pub fn consult_outlinks(&self, url: &Url) -> Vec<String> {
        match self.outlinks.get(url) {
            Some(outlink_set) => outlink_set.iter().map(|url| url.to_string()).collect(),
            None => Vec::new(),
        }
    }

    pub fn store(&mut self, page: &Page, words: &Vec<String>, outlinks: &Vec<Url>) {
        self.indexed_pages.store(&page, words);
        self.inverted_indexed_pages.store(&page.url, words);

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

        if let Ok(mut file) = File::create(&self.filepath) {
            match serde_json::to_string(&self) {
                Ok(json_str) => match file.write(json_str.as_bytes()) {
                    Ok(size) => {
                        self.size_bytes = size;

                        info!(
                            "Written index ({:?}B) to file succesfully: {}",
                            size, self.filepath
                        );
                    }
                    Err(e) => error!("Failed to write index to file: {}", e),
                },
                Err(e) => {
                    error!("Failed to serialize index: {:#?}: {}", &self, e);
                }
            }
        } else {
            error!("Error creating file: {}", self.filepath);
        }
    }

    pub fn load(filepath: &str) -> Self {
        let mut index = match fs::read_to_string(filepath) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(index) => {
                    info!("Success to load index file: {}", filepath);
                    index
                }
                Err(e) => {
                    error!("Failed to parse index file: {}", e);
                    Index::default()
                }
            },
            Err(e) => {
                error!("Failed to read index file: {}", e);
                Index::default()
            }
        };

        index.filepath = filepath.to_string();
        index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_sample_index() -> Index {
        let mut index = Index {
            filepath: "test_path.json".to_string(),
            size_bytes: 0,
            backlinks: HashMap::new(),
            outlinks: HashMap::new(),
            indexed_pages: IndexPages(HashMap::new()),
            inverted_indexed_pages: InvertedIndexPages(HashMap::new()),
        };

        // Sample page
        let page = Page::create("https://example.com").with_title("Example");

        let words = vec!["test".to_string(), "page".to_string()];

        // Store page and words into index
        index.store(
            &page,
            &words,
            &vec![Url::from_str("http://outlink.com").unwrap()],
        );

        index
    }

    #[test]
    fn test_serialize_deserialize_index() {
        let index = create_sample_index();

        let json_str = serde_json::to_string(&index).expect("Serialization failed");

        let deserialized_index: Index =
            serde_json::from_str(&json_str).expect("Deserialization failed");

        assert_eq!("".to_string(), deserialized_index.filepath);
        assert_eq!(
            index.indexed_pages.into_vec(),
            deserialized_index.indexed_pages.into_vec()
        );
        assert_eq!(
            index.inverted_indexed_pages.into_vec(),
            deserialized_index.inverted_indexed_pages.into_vec()
        );

        // Check that backlinks and outlinks are preserved
        assert_eq!(index.backlinks, deserialized_index.backlinks);
        assert_eq!(index.outlinks, deserialized_index.outlinks);
    }
}
