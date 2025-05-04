#![feature(test)]

extern crate test;

use stupid_simple_kv::IntoKey;

#[cfg(test)]
mod bench_keys {
    use super::*;
    use test::{Bencher, black_box};

    // Benchmark encoding alone (to isolate encoding performance)
    #[bench]
    fn bench_key_encoding(b: &mut Bencher) {
        let items: Vec<_> = (0..10000u64).collect();
        b.iter(|| {
            for &i in &items {
                black_box(("foo", i, "bar", true).into_key());
            }
        });
    }
}
