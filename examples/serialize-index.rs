use std::collections::{HashMap, HashSet};

use googol::page::Page;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexPages(HashMap<String, HashSet<Page>>);

#[derive(Debug, Default, Serialize, Deserialize)]
// WARN: Page cannot be serialized into json key
pub struct InvertedIndexPages(HashMap<Url, HashSet<String>>);

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

fn main() {
    let mut index = Index {
        backlinks: HashMap::new(),
        outlinks: HashMap::new(),
        filepath: "file.json".to_string(),
        size_bytes: 0,
        indexed_pages: IndexPages::default(),
        inverted_indexed_pages: InvertedIndexPages::default(),
    };

    let url = Url::parse("https://google.com").unwrap();
    let word = "foo".to_string();

    let page = Page::create(url.as_str()).with_title("Google");

    index
        .outlinks
        .entry(url.clone())
        .or_default()
        .insert(url.clone());

    index
        .backlinks
        .entry(url.clone())
        .or_default()
        .insert(url.clone());

    index
        .indexed_pages
        .0
        .entry(word.clone())
        .or_default()
        .insert(page.clone());

    index
        .inverted_indexed_pages
        .0
        .entry(url.clone())
        .or_default()
        .insert(word.clone());

    dbg!(&index);

    let index_json = serde_json::to_string(&index).expect("Failed serializing");

    dbg!(&index_json);
}
