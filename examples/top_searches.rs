use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// A helper struct to store keywords and their counts, and retrieve top N keywords.
pub struct KeywordCounter {
    counts: HashMap<String, usize>,
}

impl KeywordCounter {
    /// Create a new empty KeywordCounter
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Increment the count for a keyword
    pub fn add_keyword(&mut self, keyword: &str) {
        *self.counts.entry(keyword.to_string()).or_insert(0) += 1;
    }

    /// Get the top n keywords with the biggest counts
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

        // Extract and sort in descending order
        let mut result: Vec<(usize, String)> = heap
            .into_iter()
            .map(|Reverse((count, keyword))| (count, keyword))
            .collect();

        result.sort_by(|a, b| b.0.cmp(&a.0));

        // Convert to (keyword, count) tuple for output
        result
            .into_iter()
            .map(|(count, keyword)| (keyword, count))
            .collect()
    }
}

// Example usage
fn main() {
    let mut counter = KeywordCounter::new();

    // Add some keywords
    counter.add_keyword("rust");
    counter.add_keyword("rust");
    counter.add_keyword("programming");
    counter.add_keyword("language");
    counter.add_keyword("rust");
    counter.add_keyword("performance");
    counter.add_keyword("code");
    counter.add_keyword("code");
    counter.add_keyword("code");
    counter.add_keyword("performance");
    counter.add_keyword("performance");
    counter.add_keyword("performance");

    let top_keywords = counter.top_n(3);
    for (keyword, count) in top_keywords {
        println!("{}: {}", keyword, count);
    }
}
