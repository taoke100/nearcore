#[macro_use]
extern crate bencher;

use bencher::Bencher;
use cached::{Cached, SizedCache};
use lru::LruCache;

fn bench_lru(bench: &mut Bencher) {
    let mut cache = LruCache::new(10000);
    bench.iter(|| {
        for x in 0..100000 {
            let a = rand::random::<u64>();
            let b = rand::random::<u64>();
            cache.put(a, b);
        }
    });
}

fn bench_sized_cache(bench: &mut Bencher) {
    let mut cache = SizedCache::with_size(10000);
    bench.iter(|| {
        for x in 0..100000 {
            let a = rand::random::<u64>();
            let b = rand::random::<u64>();
            cache.cache_set(a, b);
        }
    });
}

benchmark_group!(benches, bench_lru, bench_sized_cache);

benchmark_main!(benches);
