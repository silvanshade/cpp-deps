extern crate alloc;

use cpp_deps::order::Order;
use criterion::{criterion_group, criterion_main, Criterion};
use p1689::r5::{self, parsers::ParseStream};

fn single(c: &mut Criterion) {
    use rustc_hash::FxHashMap;

    let mut group = c.benchmark_group("single");

    group.throughput(criterion::Throughput::Bytes(
        [
            data::BAR.len(),
            data::FOO.len(),
            data::FOO_PART1.len(),
            data::FOO_PART2.len(),
            data::MAIN.len(),
        ]
        .into_iter()
        .sum::<usize>() as u64,
    ));

    group.bench_function("channel", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                #[allow(clippy::useless_conversion)]
                let entries: [(&r5::Utf8Path, &str); 5] = [
                    ("bar.ddi".into(), data::BAR),
                    ("foo-part1.ddi".into(), data::FOO_PART1),
                    ("foo-part2.ddi".into(), data::FOO_PART2),
                    ("foo.ddi".into(), data::FOO),
                    ("main.ddi".into(), data::MAIN),
                ];
                let mut paths = vec![];
                let mut mmaps = FxHashMap::default();
                let infos = {
                    let (infos_tx, infos_rx) = std::sync::mpsc::channel();
                    for (key, val) in entries {
                        mmaps.insert(key, val);
                        paths.push(key);
                    }
                    for path in paths {
                        let mmap = mmaps.get(path).unwrap();
                        let state = r5::parsers::State::default();
                        let mut stream = ParseStream::new(path, mmap.as_ref(), state);
                        let dep_file = r5::parsers::dep_file(&mut stream).unwrap();
                        for dep_info in dep_file.rules {
                            infos_tx.send(dep_info).unwrap();
                        }
                    }
                    infos_rx
                };
                let mut graph = FxHashMap::default();
                let order = Order::new(infos, &mut graph);
                let start = std::time::Instant::now();
                for item in order {
                    let _ = item;
                }
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.bench_function("vec", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                #[allow(clippy::useless_conversion)]
                let entries: [(&r5::Utf8Path, &str); 5] = [
                    ("bar.ddi".into(), data::BAR),
                    ("foo-part1.ddi".into(), data::FOO_PART1),
                    ("foo-part2.ddi".into(), data::FOO_PART2),
                    ("foo.ddi".into(), data::FOO),
                    ("main.ddi".into(), data::MAIN),
                ];
                let mut paths = vec![];
                let mut mmaps = FxHashMap::default();
                let mut infos = vec![];
                for (key, val) in entries {
                    mmaps.insert(key, val);
                    paths.push(key);
                }
                for path in paths {
                    let mmap = mmaps.get(path).unwrap();
                    let state = r5::parsers::State::default();
                    let mut stream = ParseStream::new(path, mmap.as_ref(), state);
                    let dep_file = r5::parsers::dep_file(&mut stream).unwrap();
                    for dep_info in dep_file.rules {
                        infos.push(dep_info);
                    }
                }
                let mut graph = FxHashMap::default();
                let order = Order::new(infos, &mut graph);
                let start = std::time::Instant::now();
                for item in order {
                    let _ = item;
                }
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.bench_function("vec-out-of-order", |b| {
        b.iter_custom(|iters| {
            let mut total_time = std::time::Duration::default();
            for _ in 0 .. iters {
                #[allow(clippy::useless_conversion)]
                let entries: [(&r5::Utf8Path, &str); 5] = [
                    ("main.ddi".into(), data::MAIN),
                    ("foo.ddi".into(), data::FOO),
                    ("bar.ddi".into(), data::BAR),
                    ("foo-part1.ddi".into(), data::FOO_PART1),
                    ("foo-part2.ddi".into(), data::FOO_PART2),
                ];
                let mut paths = vec![];
                let mut mmaps = FxHashMap::default();
                let mut infos = vec![];
                for (key, val) in entries {
                    mmaps.insert(key, val);
                    paths.push(key);
                }
                for path in paths {
                    let mmap = mmaps.get(path).unwrap();
                    let state = r5::parsers::State::default();
                    let mut stream = ParseStream::new(path, mmap.as_ref(), state);
                    let dep_file = r5::parsers::dep_file(&mut stream).unwrap();
                    for dep_info in dep_file.rules {
                        infos.push(dep_info);
                    }
                }
                let mut graph = FxHashMap::default();
                let order = Order::new(infos.into_iter(), &mut graph);
                let start = std::time::Instant::now();
                for item in order {
                    let _ = item;
                }
                total_time += start.elapsed();
            }
            total_time
        })
    });

    group.finish();
}

criterion_group!(benches, single);
criterion_main!(benches);

mod data {
    pub const BAR: &str = r#"
{
"rules": [
{
"primary-output": "bar.o",
"provides": [
{
"logical-name": "bar",
"is-interface": true
}
],
"requires": [
]
}
],
"version": 0,
"revision": 0
}
"#;

    pub const FOO_PART1: &str = r#"
{
"rules": [
{
"primary-output": "foo-part1.o",
"provides": [
{
"logical-name": "foo:part1",
"is-interface": true
}
],
"requires": [
]
}
],
"version": 0,
"revision": 0
}
"#;

    pub const FOO_PART2: &str = r#"
{
"rules": [
{
"primary-output": "foo-part2.o",
"provides": [
{
"logical-name": "foo:part2",
"is-interface": true
}
],
"requires": [
]
}
],
"version": 0,
"revision": 0
}
"#;

    pub const FOO: &str = r#"
{
"rules": [
{
"primary-output": "foo.o",
"provides": [
{
"logical-name": "foo",
"is-interface": true
}
],
"requires": [
{
"logical-name": "bar"
}
,
{
"logical-name": "foo:part2"
}
,
{
"logical-name": "foo:part1"
}
]
}
],
"version": 0,
"revision": 0
}
"#;

    pub const MAIN: &str = r#"
{
"rules": [
{
"primary-output": "main.o",
"requires": [
{
"logical-name": "bar"
}
]
}
],
"version": 0,
"revision": 0
}
"#;
}
