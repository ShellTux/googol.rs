use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

#[derive(Debug, Default)]
pub struct TopSearches {
    counts: HashMap<String, usize>,
}

impl TopSearches {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    pub fn add_search(&mut self, word: &str) {
        *self.counts.entry(word.to_string()).or_insert(0) += 1;
    }

    pub fn top_n(&self, n: usize) -> Vec<(String, usize)> {
        let mut heap: BinaryHeap<Reverse<(usize, String)>> = BinaryHeap::new();

        for (keyword, &count) in &self.counts {
            if heap.len() < n {
                heap.push(Reverse((count, keyword.clone())));
            } else if let Some(&Reverse((min_count, _))) = heap.peek() {
                if count > min_count {
                    heap.pop();
                    heap.push(Reverse((count, keyword.clone())));
                }
            }
        }

        let mut result: Vec<(usize, String)> = heap
            .into_iter()
            .map(|Reverse((count, keyword))| (count, keyword))
            .collect();

        result.sort_by(|a, b| b.0.cmp(&a.0));

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
}
