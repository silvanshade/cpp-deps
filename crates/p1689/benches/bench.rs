extern crate alloc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use p1689::r5;
use rand::{RngCore, SeedableRng};

fn atoi_parsing(c: &mut Criterion) {
    let char = '💯';
    let text = char.escape_unicode().to_string();
    let text = text.strip_prefix("\\u{").unwrap();
    let input = winnow::BStr::new(text);
    c.bench_function("hex_to_u32", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let state = crate::r5::parsers::State::default();
                let stream = &mut winnow::Stateful { input, state };
                let start = std::time::Instant::now();
                stream.state.hex_to_u32::<()>(stream).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });
}

fn json_parsing(c: &mut Criterion) {
    let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(2);
    let mut bytes = alloc::vec![0u8; 8192];
    rng.fill_bytes(&mut bytes);
    let mut u = arbitrary::Unstructured::new(&bytes);
    let node_count = u.int_in_range(0u8 ..= 16u8).unwrap();

    let mut dep_files = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, node_count)
        .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
    let dep_file = dep_files.next().unwrap();

    let mut group = c.benchmark_group("parsing");

    group.throughput(criterion::Throughput::Bytes(dep_file.len() as u64));

    group.bench_function("winnow", |b| {
        b.iter(|| {
            let input = winnow::BStr::new(dep_file.as_bytes());
            let state = r5::parsers::State::default();
            let mut state_stream = winnow::Stateful { input, state };
            r5::parsers::dep_file(black_box(&mut state_stream)).unwrap();
        })
    });

    group.bench_function("serde_json", |b| {
        b.iter(|| serde_json::from_str::<r5::DepFile>(&dep_file).unwrap())
    });

    group.bench_function("simd_json", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                let mut buf = dep_file.clone().into_bytes();
                let start = std::time::Instant::now();
                simd_json::from_slice::<r5::DepFile>(&mut buf).unwrap();
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.finish();
}

criterion_group!(benches, atoi_parsing, json_parsing);
criterion_main!(benches);
