use core::marker::PhantomData;
use std::{borrow::Cow, rc::Rc};

// TODO:
// - detect duplicate nodes in input
// - bisimulation
use p1689::r5::{self};
use rustc_hash::FxHashMap;

pub enum Graph<'i> {
    Deps { deps: Vec<Rc<r5::DepInfo<'i>>> },
    Done,
}
impl Default for Graph<'_> {
    fn default() -> Self {
        let deps = vec![];
        Graph::Deps { deps }
    }
}
#[cfg(test)]
impl<'i> core::fmt::Debug for Graph<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deps { deps } => {
                let requires = deps
                    .iter()
                    .map(|elem| elem.primary_output.clone().unwrap_or_default())
                    .collect::<Vec<_>>();
                f.debug_tuple("Deps").field(&requires).finish()
            },
            Self::Done => write!(f, "Done"),
        }
    }
}

#[derive(Clone)]
pub struct OrderError<'i> {
    phantom: PhantomData<r5::DepInfo<'i>>,
}
impl<'i> OrderError<'i> {
    fn new() -> Self {
        Self { phantom: PhantomData }
    }
}

#[cfg(feature = "std")]
impl core::fmt::Debug for OrderError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderError").finish_non_exhaustive()
    }
}
#[cfg(feature = "std")]
impl core::fmt::Display for OrderError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}
#[cfg(feature = "std")]
impl std::error::Error for OrderError<'_> {}

pub struct Order<'i, I> {
    nodes: I,
    graph: &'i mut FxHashMap<Cow<'i, str>, Graph<'i>>,
    stack: Vec<r5::DepInfo<'i>>,
    solve: usize,
    order: Vec<Cow<'i, r5::Utf8Path>>,
    #[cfg(all(test, feature = "verify"))]
    check: bool,
    #[cfg(all(test, feature = "verify"))]
    other: Vec<Cow<'i, r5::Utf8Path>>,
}
impl<'i, I> Order<'i, I> {
    #[inline]
    pub fn new<T>(nodes: T, graph: &'i mut FxHashMap<Cow<'i, str>, Graph<'i>>) -> Self
    where
        T: IntoIterator<Item = r5::DepInfo<'i>, IntoIter = I>,
    {
        Self {
            nodes: nodes.into_iter(),
            graph,
            stack: Vec::new(),
            solve: 0,
            order: Vec::new(),
            #[cfg(all(test, feature = "verify"))]
            check: false,
            #[cfg(all(test, feature = "verify"))]
            other: Vec::new(),
        }
    }

    #[cfg(all(test, feature = "verify"))]
    fn trace(mut self, mut other: Vec<Cow<'i, r5::Utf8Path>>) -> Self {
        self.check = true;
        self.other = {
            other.reverse();
            other
        };
        self
    }

    #[inline(always)]
    fn verify(
        &mut self,
        output: Option<Cow<'i, r5::Utf8Path>>,
    ) -> Option<Result<Cow<'i, r5::Utf8Path>, OrderError<'i>>> {
        #[cfg(all(test, feature = "verify"))]
        if self.check {
            debug_assert_eq!(output, self.other.pop());
        }
        output.map(Ok)
    }

    fn resolve(&mut self, node: r5::DepInfo<'i>) -> Option<Result<Cow<'i, r5::Utf8Path>, OrderError<'i>>> {
        for provide in &node.provides {
            let key = provide.desc.logical_name();
            if let Some(Graph::Deps { deps }) = self.graph.insert(key, Graph::Done) {
                for node in deps.into_iter().filter_map(Rc::into_inner) {
                    self.stack.push(node);
                    self.solve -= 1;
                }
            }
        }
        self.order.extend(node.outputs);
        self.verify(node.primary_output)
    }

    #[cold]
    fn error(&self) -> Option<Result<Cow<'i, r5::Utf8Path>, OrderError<'i>>> {
        Some(Err(OrderError::new()))
    }
}

impl<'i, I> Iterator for Order<'i, I>
where
    I: Iterator<Item = r5::DepInfo<'i>>,
{
    type Item = Result<Cow<'i, r5::Utf8Path>, OrderError<'i>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(output) = self.order.pop() {
            return self.verify(Some(output));
        }

        if let Some(node) = self.stack.pop() {
            return self.resolve(node);
        }

        for node in self.nodes.by_ref() {
            let node = Rc::new(node);
            for require in node.requires.iter() {
                let key = require.desc.logical_name();
                if let Graph::Deps { ref mut deps } = self.graph.entry(key).or_default() {
                    deps.push(node.clone());
                }
            }
            if let Some(node) = Rc::into_inner(node) {
                return self.resolve(node);
            };
            self.solve += 1;
        }

        if self.solve > 0 {
            return self.error();
        }

        None
    }
}

// TODO:
// - test permutations
// - benchmark permutations

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
        "logical-name": "foo"
        },
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
        let nodes = {
            let (nodes_tx, nodes_rx) = std::sync::mpsc::channel();
            for (key, val) in entries {
                mmaps.insert(key, val);
                paths.push(key);
            }
            for path in paths {
                let mmap = mmaps.get(path).unwrap();
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, mmap.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    nodes_tx.send(info).unwrap();
                }
            }
            nodes_rx
        };
        let mut graph = FxHashMap::default();
        let order = match Order::new(nodes, &mut graph).collect::<Result<Vec<_>, _>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        assert_eq!(
            order,
            ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"].map(Into::<&r5::Utf8Path>::into)
        );
    }

    #[cfg(feature = "verify")]
    #[test]
    fn test_vec() {
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
        let mut nodes = vec![];
        for (key, val) in entries {
            mmaps.insert(key, val);
            paths.push(key);
        }
        for path in paths {
            let mmap = mmaps.get(path).unwrap();
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, mmap.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                nodes.push(info);
            }
        }
        let mut graph = FxHashMap::default();
        let order = match Order::new(nodes.clone(), &mut graph).collect::<Result<Vec<_>, _>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        assert_eq!(
            order,
            ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"].map(Into::<&r5::Utf8Path>::into)
        );
        let mut graph = FxHashMap::default();
        match Order::new(nodes, &mut graph)
            .trace(order)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
    }

    #[cfg(feature = "verify")]
    #[test]
    fn test_vec_out_of_order() {
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
        let mut nodes = vec![];
        for (key, val) in entries {
            mmaps.insert(key, val);
            paths.push(key);
        }
        for path in paths {
            let mmap = mmaps.get(path).unwrap();
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, mmap.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                nodes.push(info);
            }
        }
        let mut graph = FxHashMap::default();
        let order = match Order::new(nodes.clone(), &mut graph).collect::<Result<Vec<_>, _>>() {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
        assert_eq!(
            order,
            ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"].map(Into::<&r5::Utf8Path>::into)
        );
        let mut graph = FxHashMap::default();
        match Order::new(nodes, &mut graph)
            .trace(order)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(order) => order,
            Err(err) => {
                panic!("{err}");
            },
        };
    }

    #[test]
    #[should_panic]
    fn test_vec_cycle() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 2] = [("foo.ddi".into(), FOO_CYCLE), ("bar.ddi".into(), BAR_CYCLE)];
        let mut paths = vec![];
        let mut mmaps = FxHashMap::default();
        let mut nodes = vec![];
        for (key, val) in entries {
            mmaps.insert(key, val);
            paths.push(key);
        }
        for path in paths {
            let mmap = mmaps.get(path).unwrap();
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, mmap.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                nodes.push(info);
            }
        }
        let mut graph = FxHashMap::default();
        let order = match Order::new(nodes, &mut graph).collect::<Result<Vec<_>, _>>() {
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
        let mut nodes = vec![];
        for (key, val) in entries {
            mmaps.insert(key, val);
            paths.push(key);
        }
        for path in paths {
            let mmap = mmaps.get(path).unwrap();
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, mmap.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                nodes.push(info);
            }
        }
        let mut graph = FxHashMap::default();
        let order = match Order::new(nodes, &mut graph).collect::<Result<Vec<_>, _>>() {
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
