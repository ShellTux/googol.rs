use std::collections::{HashSet, VecDeque};
use url::Url;

use crate::{GoogolStatus, settings::gateway::DomainsFilter};

#[derive(Debug, Default)]
pub struct Queue {
    queue: VecDeque<Url>,
    seen: HashSet<Url>,
    domains_filter: DomainsFilter,
}

impl Queue {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn with_domains_filter(mut self, domains_filter: &DomainsFilter) -> Self {
        self.domains_filter = domains_filter.clone();
        self
    }

    #[allow(private_interfaces)]
    pub fn enqueue(&mut self, url: Url) -> (GoogolStatus, Vec<String>) {
        if self.seen.contains(&url) {
            return (GoogolStatus::AlreadyIndexedUrl, self.into_vec());
        }

        self.queue.push_back(url.clone());
        self.seen.insert(url);

        (GoogolStatus::Success, self.into_vec())
    }

    pub fn dequeue(&mut self) -> Option<Url> {
        self.queue.pop_front()
    }

    pub fn into_vec(&self) -> Vec<String> {
        self.queue.iter().map(|url| url.to_string()).collect()
    }

    pub fn clear_seen(&mut self) {
        self.seen.clear();

        for url in &self.queue {
            self.seen.insert(url.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_enqueue_and_duplicate() {
        let mut queue = Queue::default();
        let url = Url::parse("https://example.com").unwrap();

        // Enqueue a new URL
        let (status, list) = queue.enqueue(url.clone());
        assert_eq!(status, GoogolStatus::Success);
        assert_eq!(list, vec![url.as_str()]);

        // Enqueue the same URL again should return AlreadyIndexedUrl
        let (status_dup, list_dup) = queue.enqueue(url.clone());
        assert_eq!(status_dup, GoogolStatus::AlreadyIndexedUrl);
        assert_eq!(list_dup, vec![url.as_str()]);

        // Queue should contain only one URL
        assert_eq!(queue.into_vec(), vec![url.as_str()]);
    }

    #[test]
    fn test_dequeue() {
        let mut queue = Queue::default();

        let url1 = Url::parse("https://example.com/1").unwrap();
        let url2 = Url::parse("https://example.com/2").unwrap();

        queue.enqueue(url1.clone());
        queue.enqueue(url2.clone());

        // Dequeue should return url1 first
        let dequeued = queue.dequeue();
        assert_eq!(dequeued, Some(url1));
        // Now only url2 remains
        assert_eq!(queue.into_vec(), vec![url2.to_string()]);

        // Dequeue remaining URL
        let dequeued2 = queue.dequeue();
        assert_eq!(dequeued2, Some(url2));
        // Queue should now be empty
        assert_eq!(queue.into_vec(), Vec::<String>::new());

        // Dequeue from empty queue should return None
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn test_into_vec() {
        let mut queue = Queue::default();

        let url1 = Url::parse("https://foo.com").unwrap();
        let url2 = Url::parse("https://bar.com").unwrap();

        queue.enqueue(url1.clone());
        queue.enqueue(url2.clone());

        let vec_representation = queue.into_vec();
        assert_eq!(vec_representation, vec![url1.to_string(), url2.to_string()]);
    }

    #[test]
    fn test_clear_seen() {
        let mut queue = Queue::default();

        let url = Url::parse("https://test.com").unwrap();

        // Enqueue a URL
        queue.enqueue(url.clone());

        // Seen should contain the URL
        assert!(queue.seen.contains(&url));

        // Clear seen
        queue.clear_seen();

        // Seen should be empty
        assert!(!queue.seen.is_empty());

        // Enqueue same URL again after clearing
        let (status, list) = queue.enqueue(url.clone());
        assert_eq!(status, GoogolStatus::AlreadyIndexedUrl);
        assert_eq!(list, vec![url.to_string()]);

        queue.dequeue();
        queue.clear_seen();

        assert!(queue.seen.is_empty());

        // Enqueue same URL again after clearing should succeed
        let (status, list) = queue.enqueue(url.clone());
        dbg!(queue);
        assert_eq!(status, GoogolStatus::Success);
        assert_eq!(list, vec![url.to_string()]);
    }
}
