extern crate alloc;

use criterion::{criterion_group, criterion_main, Criterion};
use p1689::r5;
use rand::{RngCore, SeedableRng};

fn json_parsing(c: &mut Criterion) {
    let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(r5::datagen::CHACHA8RNG_SEED);
    let mut bytes = alloc::vec![0u8; 8192];
    rng.fill_bytes(&mut bytes);
    let mut u = arbitrary::Unstructured::new(&bytes);
    let config = r5::datagen::graph::GraphGeneratorConfig::default().node_count(u.int_in_range(0u8 ..= 16u8).unwrap());

    let mut dep_files = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, config)
        .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
    let dep_file = dep_files.next().unwrap();

    let mut group = c.benchmark_group("parsing");

    group.throughput(criterion::Throughput::Bytes(dep_file.len() as u64));

    group.bench_function("winnow", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let input = winnow::BStr::new(dep_file.as_bytes());
                let state = r5::parsers::State::default();
                let mut stream = winnow::Stateful { input, state };
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

criterion_group!(benches, json_parsing);
criterion_main!(benches);
