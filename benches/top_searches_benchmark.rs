use criterion::{Criterion, criterion_group, criterion_main};
use googol::top_searches::TopSearches;
use rand::{
    distr::{Alphanumeric, Distribution},
    rng,
    seq::IndexedRandom,
};
use std::hint;

fn generate_random_word(len: usize) -> String {
    let mut rng = rng();
    Alphanumeric
        .sample_iter(&mut rng)
        .take(len)
        .map(char::from)
        .collect()
}

fn benchmark_add_search(c: &mut Criterion) {
    let total_adds = 100_000;

    c.bench_function(
        &format!("top_searches add_search {} times", total_adds),
        |b| {
            b.iter(|| {
                // Clear the searches for each iteration if needed
                // or create a new instance outside the loop
                let mut local_searches = TopSearches::new();

                for _ in 0..total_adds {
                    let word = generate_random_word(5);
                    local_searches.add_search(&word);
                }

                // Use black_box to prevent optimization
                hint::black_box(local_searches);
            });
        },
    );
}

fn benchmark_count(c: &mut Criterion) {
    let mut searches = TopSearches::new();

    // Populate with some data
    let total_searches = 50_000;
    let mut words = Vec::new();
    for _ in 0..total_searches {
        let word = generate_random_word(5);
        words.push(word.clone());
        searches.add_search(&word);
    }

    // Pick some words to test count
    let total_words = 100;
    let test_words: Vec<&String> = words.choose_multiple(&mut rng(), total_words).collect();

    c.bench_function(&format!("top_searches count {} words", total_words), |b| {
        b.iter(|| {
            for &word in &test_words {
                let _count = searches.count(word);
                // Use black_box to prevent optimization
                hint::black_box(_count);
            }
        });
    });
}

fn benchmark_top_n(c: &mut Criterion) {
    // Initialize TopSearches with a large number of random searches
    let mut searches = TopSearches::new();

    // Generate and add a large number of searches
    let total_searches = 100_000;
    let mut counts_map = std::collections::HashMap::new();

    // For reproducibility, you might want to seed your RNG
    for _ in 0..total_searches {
        let word = generate_random_word(5);
        *counts_map.entry(word.clone()).or_insert(0) += 1;
        searches.add_search(&word);
    }

    // Determine how many top searches to retrieve
    let top_n = 10;

    // Benchmark the top_n method
    c.bench_function(
        &format!("top_searches top_n {} large dataset", top_n),
        |b| {
            b.iter(|| {
                let result = searches.top_n(top_n);
                // Optionally, do something with result to prevent compiler optimizations
                hint::black_box(result);
            });
        },
    );
}

criterion_group!(
    benches,
    benchmark_add_search,
    benchmark_count,
    benchmark_top_n
);
criterion_main!(benches);
