use std::{borrow::Cow, rc::Rc};

use p1689::r5::{self};
use rustc_hash::FxHashMap;

use crate::BoxResult;

pub enum Graph<'i> {
    Awaiting { requires: Vec<Rc<r5::DepInfo<'i>>> },
    Finished,
}
impl Default for Graph<'_> {
    fn default() -> Self {
        let requires = vec![];
        Graph::Awaiting { requires }
    }
}

pub struct Order<'i, I> {
    infos: I,
    graph: FxHashMap<Cow<'i, str>, Graph<'i>>,
    order: Vec<Cow<'i, r5::Utf8Path>>,
    ended: bool,
    unresolved: usize,
}
impl<'i, I> Order<'i, I> {
    #[inline]
    pub fn new<T>(infos: T) -> Self
    where
        T: IntoIterator<Item = r5::DepInfo<'i>, IntoIter = I>,
    {
        let infos = infos.into_iter();
        let graph = FxHashMap::default();
        let order = Vec::new();
        let ended = false;
        let unresolved = 0;
        Self {
            infos,
            graph,
            order,
            ended,
            unresolved,
        }
    }
}

impl<'i, I> Iterator for Order<'i, I>
where
    I: Iterator<Item = r5::DepInfo<'i>>,
{
    type Item = BoxResult<Cow<'i, r5::Utf8Path>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while !self.ended && self.order.is_empty() {
            let Some(dep_info) = self.infos.next() else {
                self.ended = true;
                if self.unresolved > 0 {
                    return Some(Err("Cycle or incomplete build graph detected".into()));
                }
                break;
            };
            let dep_info = Rc::new(dep_info);
            for require in &dep_info.requires {
                let key = require.desc.logical_name();
                if let Graph::Awaiting { ref mut requires } = self.graph.entry(key).or_default() {
                    requires.push(dep_info.clone());
                }
            }
            if Rc::strong_count(&dep_info) != 1 {
                self.unresolved += 1;
                continue;
            }
            let outputs = dep_info.primary_output.iter().chain(&dep_info.outputs).cloned();
            self.order.extend(outputs);
            for provide in &dep_info.provides {
                let key = provide.desc.logical_name();
                if let Some(Graph::Awaiting { requires }) = self.graph.insert(key, Graph::Finished) {
                    for dep_info in requires.into_iter().filter_map(Rc::into_inner) {
                        let outputs = dep_info.primary_output.iter().chain(&dep_info.outputs).cloned();
                        self.order.extend(outputs);
                        self.unresolved -= 1;
                    }
                }
            }
        }
        self.order.pop().map(Ok)
    }
}

#[cfg(test)]
mod test {
    use r5::parsers::ParseStream;

    use super::*;

    const BAR: &str = r#"
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

    const FOO_PART1: &str = r#"
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

    const FOO_PART2: &str = r#"
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

    const FOO: &str = r#"
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

    const MAIN: &str = r#"
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

    const FOO_CYCLE: &str = r#"
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
        ]
        }
        ],
        "version": 0,
        "revision": 0
        }
    "#;

    const BAR_CYCLE: &str = r#"
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
        {
        "logical-name": "foo"
        }
        ]
        }
        ],
        "version": 0,
        "revision": 0
        }
    "#;

    #[test]
    fn test_channel() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
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
        let order = match Order::new(infos).collect::<BoxResult<Vec<_>>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        for item in order {
            std::println!("{}", item);
        }
    }

    #[test]
    fn test_vec() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("main.ddi".into(), MAIN),
            ("foo.ddi".into(), FOO),
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
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
        let order = match Order::new(infos).collect::<BoxResult<Vec<_>>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        for item in order {
            std::println!("{}", item);
        }
    }

    #[test]
    #[should_panic]
    fn test_vec_cycle() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 2] = [("foo.ddi".into(), FOO_CYCLE), ("bar.ddi".into(), BAR_CYCLE)];
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
        let order = match Order::new(infos).collect::<BoxResult<Vec<_>>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        for item in order {
            std::println!("{}", item);
        }
    }

    #[test]
    #[should_panic]
    fn test_vec_incomplete() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 2] = [("foo.ddi".into(), FOO_CYCLE), ("bar.ddi".into(), BAR_CYCLE)];
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
        let order = match Order::new(infos).collect::<BoxResult<Vec<_>>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        for item in order {
            std::println!("{}", item);
        }
    }
}
