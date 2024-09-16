use core::marker::PhantomData;
use std::{borrow::Cow, collections::HashMap, rc::Rc};

use p1689::r5;
use yoke::{Yoke, Yokeable};

use crate::{DepInfoCart, DepInfoYoke};

#[derive(Clone)]
#[repr(transparent)]
pub struct DepInfoNameYoke {
    yoke: Yoke<Cow<'static, str>, DepInfoCart>,
}
impl core::fmt::Debug for DepInfoNameYoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self.yoke.get(), f)
    }
}
impl PartialEq for DepInfoNameYoke {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.yoke.get().eq(other.yoke.get())
    }
}
impl Eq for DepInfoNameYoke {}
impl core::hash::Hash for DepInfoNameYoke {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.yoke.get().hash(state);
    }
}

#[derive(Clone)]
pub enum Graph {
    Deps { deps: Vec<Rc<DepInfoYoke>> },
    Done,
}
impl Default for Graph {
    fn default() -> Self {
        let deps = vec![];
        Graph::Deps { deps }
    }
}
impl core::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deps { deps } => {
                #[allow(clippy::useless_conversion)]
                let requires = deps
                    .iter()
                    .map(|elem| {
                        elem.get()
                            .primary_output
                            .clone()
                            .unwrap_or(Cow::Borrowed("<unknown>".into()))
                    })
                    .collect::<Vec<_>>();
                f.debug_tuple("Deps").field(&requires).finish()
            },
            Self::Done => write!(f, "Done"),
        }
    }
}

#[non_exhaustive]
#[derive(Clone)]
pub enum OrderError<E> {
    CycleOrIncomplete { graph: HashMap<DepInfoNameYoke, Graph> },
    External(E),
}

impl<E> core::fmt::Debug for OrderError<E>
where
    E: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::CycleOrIncomplete { graph } => {
                f.debug_struct("CycleOrIncomplete").field("graph", &graph).finish()
            },
            OrderError::External(error) => core::fmt::Debug::fmt(&error, f),
        }
    }
}
impl<E> core::fmt::Display for OrderError<E>
where
    E: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderError::CycleOrIncomplete { graph } => {
                f.debug_struct("CycleOrIncomplete").field("graph", &graph).finish()
            },
            OrderError::External(error) => core::fmt::Display::fmt(&error, f),
        }
    }
}
#[cfg(feature = "std")]
impl<E> std::error::Error for OrderError<E> where E: std::error::Error {}

pub struct Order<E, I> {
    nodes: I,
    graph: HashMap<DepInfoNameYoke, Graph>,
    stack: Vec<DepInfoYoke>,
    solve: usize,
    #[cfg(all(test, feature = "verify"))]
    check: bool,
    #[cfg(all(test, feature = "verify"))]
    other: Vec<Result<DepInfoYoke, OrderError<E>>>,
    e: PhantomData<E>,
}
impl<E, I> Order<E, I> {
    #[inline]
    pub fn new<T>(nodes: T) -> Self
    where
        T: IntoIterator<Item = Result<DepInfoYoke, E>, IntoIter = I>,
    {
        Self {
            nodes: nodes.into_iter(),
            graph: HashMap::default(),
            stack: Vec::new(),
            solve: 0,
            #[cfg(all(test, feature = "verify"))]
            check: false,
            #[cfg(all(test, feature = "verify"))]
            other: Vec::new(),
            e: PhantomData,
        }
    }

    #[cold]
    fn error(&self) -> Option<Result<DepInfoYoke, OrderError<E>>> {
        let graph = self.graph.clone();
        let error = OrderError::CycleOrIncomplete { graph };
        Some(Err(error))
    }

    pub fn outputs(self) -> OrderOutputs<E, I>
    where
        I: Iterator<Item = Result<DepInfoYoke, E>>,
    {
        OrderOutputs::<E, I> {
            iter: self,
            node: Option::default(),
        }
    }

    fn resolve(&mut self, node: DepInfoYoke) -> Option<Result<DepInfoYoke, OrderError<E>>> {
        for provide in &node.get().provides {
            let key = DepInfoNameYoke {
                yoke: Yoke::attach_to_cart(node.backing_cart().clone(), |_| unsafe {
                    Yokeable::make(provide.desc.logical_name())
                }),
            };
            if let Some(Graph::Deps { deps }) = self.graph.insert(key, Graph::Done) {
                for node in deps.into_iter().filter_map(Rc::into_inner) {
                    self.stack.push(node);
                    self.solve -= 1;
                }
            }
        }
        self.verify(node)
    }

    #[cfg(all(test, feature = "verify"))]
    pub fn trace<Os>(mut self, other: Os) -> Self
    where
        Os: IntoIterator<Item = Result<DepInfoYoke, OrderError<E>>>,
        Os::IntoIter: DoubleEndedIterator,
    {
        self.check = true;
        self.other = other.into_iter().rev().collect();
        self
    }

    #[inline(always)]
    fn verify(&mut self, yoke: DepInfoYoke) -> Option<Result<DepInfoYoke, OrderError<E>>> {
        #[cfg(all(test, feature = "verify"))]
        if self.check {
            debug_assert_eq!(
                Some(yoke.get()),
                self.other
                    .pop()
                    .as_ref()
                    .and_then(|res| res.as_ref().map(Yoke::get).ok())
            );
        }
        Some(Ok(yoke))
    }
}
impl<E, I> Iterator for Order<E, I>
where
    I: Iterator<Item = Result<DepInfoYoke, E>>,
{
    type Item = Result<DepInfoYoke, OrderError<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            return self.resolve(node);
        }

        for result in self.nodes.by_ref() {
            match result {
                Err(err) => return Some(Err(OrderError::External(err))),
                Ok(node) => {
                    let node = Rc::new(node);
                    for require in node.get().requires.iter() {
                        let key = DepInfoNameYoke {
                            yoke: Yoke::attach_to_cart(node.backing_cart().clone(), |_| unsafe {
                                Yokeable::make(require.desc.logical_name())
                            }),
                        };
                        if let Graph::Deps { ref mut deps } = self.graph.entry(key).or_default() {
                            deps.push(node.clone());
                        }
                    }
                    if let Some(node) = Rc::into_inner(node) {
                        return self.resolve(node);
                    };
                    self.solve += 1;
                },
            }
        }

        if self.solve > 0 {
            return self.error();
        }

        None
    }
}

pub struct OrderOutputs<E, I> {
    iter: Order<E, I>,
    node: Option<(DepInfoCart, <Vec<Cow<'static, r5::Utf8Path>> as IntoIterator>::IntoIter)>,
}
impl<E, I> Iterator for OrderOutputs<E, I>
where
    I: Iterator<Item = Result<DepInfoYoke, E>>,
{
    type Item = Result<Yoke<Cow<'static, r5::Utf8Path>, DepInfoCart>, OrderError<E>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((cart, outputs)) = &mut self.node {
            if let Some(output) = outputs.next() {
                let yoke = Yoke::attach_to_cart(cart.clone(), |_| output);
                return Some(Ok(yoke));
            }
        }
        match self.iter.next() {
            Some(Ok(yoke)) => {
                let cart = yoke.backing_cart().clone();
                let info = unsafe { yoke.replace_cart(|_| ()) }.into_yokeable().transform_owned();
                self.node = Some((cart.clone(), info.outputs.into_iter()));
                info.primary_output
                    .map(|output| Yoke::attach_to_cart(cart, |_| output))
                    .map(Ok)
            },
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use core::convert::Infallible;
    use std::sync::Arc;

    use r5::parsers::ParseStream;

    use super::*;
    use crate::CppDeps;

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
    fn channel() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
        ];
        let nodes = {
            let (nodes_tx, nodes_rx) = std::sync::mpsc::channel();
            for (path, data) in entries {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, data.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(data) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes_tx.send(Ok(node)).unwrap();
                }
            }
            nodes_rx
        };
        let mut order = Order::<Infallible, _>::new(nodes).outputs();
        for expect in ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"] {
            let Some(result) = order.next() else {
                panic!("Output ended unexpectedly");
            };
            match result {
                Err(err) => panic!("{err}"),
                Ok(yoke) => assert_eq!(yoke.get().as_str(), expect),
            }
        }
    }

    #[test]
    fn vec() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
        ];
        let mut nodes = vec![];
        for (path, data) in entries {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, data.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(data) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let mut order = Order::<Infallible, _>::new(nodes).outputs();
        for expect in ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"] {
            let Some(result) = order.next() else {
                panic!("Output ended unexpectedly");
            };
            match result {
                Err(err) => panic!("{err}"),
                Ok(yoke) => assert_eq!(yoke.get().as_str(), expect),
            }
        }
    }

    #[test]
    fn vec_out_of_order() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("main.ddi".into(), MAIN),
            ("foo.ddi".into(), FOO),
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
        ];
        let mut nodes = vec![];
        for (path, data) in entries {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, data.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(data) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let mut order = Order::<Infallible, _>::new(nodes).outputs();
        for expect in ["bar.o", "foo-part1.o", "foo-part2.o", "foo.o", "main.o"] {
            let Some(result) = order.next() else {
                panic!("Output ended unexpectedly");
            };
            match result {
                Err(err) => panic!("{err}"),
                Ok(yoke) => assert_eq!(yoke.get().as_str(), expect),
            }
        }
    }

    #[test]
    #[should_panic]
    fn vec_cycle() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 2] = [("foo.ddi".into(), FOO_CYCLE), ("bar.ddi".into(), BAR_CYCLE)];
        let mut nodes = vec![];
        for (path, data) in entries {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, data.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(data) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        for result in Order::<Infallible, _>::new(nodes) {
            match result {
                Err(err) => {
                    panic!("{err}");
                },
                Ok(yoke) => {
                    let Some(output) = yoke.get().primary_output.as_ref() else {
                        continue;
                    };
                    std::println!("{output}");
                },
            }
        }
    }

    #[test]
    #[should_panic]
    fn vec_incomplete() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 2] = [("foo.ddi".into(), FOO_CYCLE), ("bar.ddi".into(), BAR_CYCLE)];
        let mut nodes = vec![];
        for (path, data) in entries {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(path, data.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(data) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        for result in Order::<Infallible, _>::new(nodes) {
            match result {
                Err(err) => {
                    panic!("{err}");
                },
                Ok(yoke) => {
                    let Some(output) = yoke.get().primary_output.as_ref() else {
                        continue;
                    };
                    std::println!("{output}");
                },
            }
        }
    }

    #[test]
    fn trace() {
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
        ];
        let nodes = {
            let mut nodes = vec![];
            for (path, data) in entries {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, data.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(data) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes.push(Ok(node));
                }
            }
            nodes
        };
        let other = Order::<Infallible, _>::new(nodes);
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("main.ddi".into(), MAIN),
            ("foo.ddi".into(), FOO),
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
        ];
        let nodes = {
            let mut nodes = vec![];
            for (path, data) in entries {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(path, data.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(data) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes.push(Ok(node));
                }
            }
            nodes
        };
        let other = other.collect::<Vec<_>>();
        let order = Order::new(nodes).trace(other);
        for result in order {
            match result {
                Err(err) => {
                    panic!("{err}");
                },
                Ok(yoke) => {
                    let Some(output) = yoke.get().primary_output.as_ref() else {
                        continue;
                    };
                    std::println!("{output}");
                },
            }
        }
    }

    #[test]
    fn cpp_deps() {
        let out_dir = tempdir::TempDir::new("cpp_deps::order::test::cpp_deps").unwrap();
        let out_dir = out_dir.path().as_os_str().to_str().unwrap();
        std::env::set_var("OPT_LEVEL", "3");
        std::env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        std::env::set_var("HOST", "x86_64-unknown-linux-gnu");
        std::env::set_var("OUT_DIR", out_dir);
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
        ];
        let cc = cc::Build::default();
        let cpp_deps = CppDeps::new(&cc).unwrap();
        let cpp_deps = cpp_deps.add_dep_bytes(entries);
        for result in Order::new(cpp_deps.items()) {
            match result {
                Err(_err) => {
                    panic!();
                },
                Ok(yoke) => {
                    let Some(output) = yoke.get().primary_output.as_ref() else {
                        continue;
                    };
                    std::println!("{output}");
                },
            }
        }
    }

    #[cfg(feature = "async")]
    #[test]
    fn cpp_deps_with_sink() {
        use futures_util::sink::SinkExt;
        let out_dir = tempdir::TempDir::new("cpp_deps::order::test::cpp_deps_with_sink").unwrap();
        let out_dir = out_dir.path().as_os_str().to_str().unwrap();
        std::env::set_var("OPT_LEVEL", "3");
        std::env::set_var("TARGET", "x86_64-unknown-linux-gnu");
        std::env::set_var("HOST", "x86_64-unknown-linux-gnu");
        std::env::set_var("OUT_DIR", out_dir);
        #[allow(clippy::useless_conversion)]
        let entries: [(&r5::Utf8Path, &str); 5] = [
            ("bar.ddi".into(), BAR),
            ("foo-part1.ddi".into(), FOO_PART1),
            ("foo-part2.ddi".into(), FOO_PART2),
            ("foo.ddi".into(), FOO),
            ("main.ddi".into(), MAIN),
        ];
        let cc = cc::Build::default();
        let cpp_deps = CppDeps::new(&cc).unwrap();
        let cpp_deps = cpp_deps.add_dep_bytes(entries);
        let cpp_deps = cpp_deps.items();
        let mut sink = cpp_deps.sink().unwrap();
        let handle = std::thread::spawn(|| {
            futures_executor::block_on(async move {
                let item = crate::CppDepsItem::DepData(("main.ddi".into(), MAIN));
                sink.feed(item).await
            })
            .unwrap()
        });
        for result in Order::new(cpp_deps) {
            match result {
                Err(_err) => {
                    panic!();
                },
                Ok(yoke) => {
                    let Some(output) = yoke.get().primary_output.as_ref() else {
                        continue;
                    };
                    std::println!("{output}");
                },
            }
        }
        handle.join().unwrap()
    }
}
