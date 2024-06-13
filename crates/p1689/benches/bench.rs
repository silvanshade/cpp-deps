extern crate alloc;

use criterion::{criterion_group, criterion_main, Criterion};
use p1689::r5::{self, parsers::ParseStream};
use rand::prelude::*;

fn json_parsing(c: &mut Criterion) {
    let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(r5::datagen::CHACHA8RNG_SEED);
    let config = r5::datagen::graph::GraphGeneratorConfig::default().node_count(rng.gen_range(0u8 ..= 16u8));
    let mut dep_files = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, config)
        .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
    let dep_file = dep_files.next().unwrap();

    let mut group = c.benchmark_group("parsing");

    group.throughput(criterion::Throughput::Bytes(dep_file.len() as u64));

    #[cfg(feature = "memchr")]
    group.bench_function("winnow-with-memchr", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let path = "test.ddi";
                let input = dep_file.as_bytes();
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, input, state);
                let start = std::time::Instant::now();
                r5::parsers::dep_file(&mut stream).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });

    #[cfg(not(feature = "memchr"))]
    group.bench_function("winnow-sans-memchr", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let path = "test.ddi";
                let input = dep_file.as_bytes();
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, input, state);
                let start = std::time::Instant::now();
                r5::parsers::dep_file(&mut stream).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<r5::DepFile>(&dep_file).unwrap())
    });

    group.finish();
}

fn json_parsing_with_more_escapes(c: &mut Criterion) {
    let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(r5::datagen::CHACHA8RNG_SEED);
    let config = r5::datagen::graph::GraphGeneratorConfig::default()
        .node_count(rng.gen_range(0u8 ..= 16u8))
        .more_escapes(true);
    let mut dep_files = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, config)
        .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
    let dep_file = dep_files.next().unwrap();

    let mut group = c.benchmark_group("parsing-with-more-escapes");

    group.throughput(criterion::Throughput::Bytes(dep_file.len() as u64));

    #[cfg(feature = "memchr")]
    group.bench_function("winnow-with-memchr", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let path = "test.ddi";
                let input = dep_file.as_bytes();
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, input, state);
                let start = std::time::Instant::now();
                r5::parsers::dep_file(&mut stream).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });

    #[cfg(not(feature = "memchr"))]
    group.bench_function("winnow-sans-memchr", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let path = "test.ddi";
                let input = dep_file.as_bytes();
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, input, state);
                let start = std::time::Instant::now();
                r5::parsers::dep_file(&mut stream).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<r5::DepFile>(&dep_file).unwrap())
    });

    group.finish();
}

criterion_group!(benches, json_parsing, json_parsing_with_more_escapes);
criterion_main!(benches);
