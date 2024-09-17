use core::marker::PhantomData;
use std::{borrow::Cow, collections::HashMap, rc::Rc};

use p1689::r5::{
    self,
    yoke::{DepInfoCart, DepInfoNameYoke, DepInfoYoke, DepInfoYokeExt},
};
use yoke::{Yoke, Yokeable};

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
        for key in node.provides() {
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

#[cfg(test)]
impl<E, I> Order<E, I>
where
    I: Iterator<Item = Result<DepInfoYoke, E>>,
{
    pub fn validate(self) -> Option<()> {
        use std::collections::BTreeSet;
        let mut valid = BTreeSet::new();
        for result in self {
            let node = result.ok()?;
            for provide in node.provides() {
                valid.insert(provide);
            }
            for require in node.requires() {
                if !valid.contains(&require) {
                    return None;
                }
            }
        }
        Some(())
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
                    for key in node.requires() {
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

    use r5::{parsers::ParseStream, yoke::DepInfoCart};

    use super::*;
    use crate::CppDepsBuilder;

    #[test]
    fn channel() {
        let nodes = {
            let (nodes_tx, nodes_rx) = std::sync::mpsc::channel();
            for entry in [
                crate::testing::corpus::entry::bar(),
                crate::testing::corpus::entry::foo_part1(),
                crate::testing::corpus::entry::foo_part2(),
                crate::testing::corpus::entry::foo(),
                crate::testing::corpus::entry::main(),
            ] {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(entry.json) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes_tx.send(Ok(node)).unwrap();
                }
            }
            nodes_rx
        };
        let order = Order::<Infallible, _>::new(nodes);
        assert!(order.validate().is_some());
    }

    #[test]
    fn vec() {
        let mut nodes = vec![];
        for entry in [
            crate::testing::corpus::entry::bar(),
            crate::testing::corpus::entry::foo_part1(),
            crate::testing::corpus::entry::foo_part2(),
            crate::testing::corpus::entry::foo(),
            crate::testing::corpus::entry::main(),
        ] {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(entry.json) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let order = Order::<Infallible, _>::new(nodes);
        assert!(order.validate().is_some());
    }

    #[test]
    fn vec_out_of_order() {
        let mut nodes = vec![];
        for entry in [
            crate::testing::corpus::entry::bar(),
            crate::testing::corpus::entry::foo_part1(),
            crate::testing::corpus::entry::foo_part2(),
            crate::testing::corpus::entry::foo(),
            crate::testing::corpus::entry::main(),
        ] {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(entry.json) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let order = Order::<Infallible, _>::new(nodes);
        assert!(order.validate().is_some());
    }

    #[test]
    #[should_panic]
    fn vec_cycle() {
        let mut nodes = vec![];
        for entry in [
            crate::testing::corpus::entry::foo_cycle(),
            crate::testing::corpus::entry::bar_cycle(),
        ] {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(entry.json) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let order = Order::<Infallible, _>::new(nodes);
        assert!(order.validate().is_some());
    }

    #[test]
    #[should_panic]
    fn vec_incomplete() {
        let mut nodes = vec![];
        for entry in [
            crate::testing::corpus::entry::foo_cycle(),
            crate::testing::corpus::entry::bar_cycle(),
        ] {
            let state = r5::parsers::State::default();
            let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
            let file = r5::parsers::dep_file(&mut stream).unwrap();
            for info in file.rules {
                let cart = Arc::new(entry.json) as DepInfoCart;
                let node = Yoke::attach_to_cart(cart, |_| info);
                nodes.push(Ok(node));
            }
        }
        let order = Order::<Infallible, _>::new(nodes);
        assert!(order.validate().is_some());
    }

    #[test]
    fn trace() {
        let nodes = {
            let mut nodes = vec![];
            for entry in [
                crate::testing::corpus::entry::bar(),
                crate::testing::corpus::entry::foo_part1(),
                crate::testing::corpus::entry::foo_part2(),
                crate::testing::corpus::entry::foo(),
                crate::testing::corpus::entry::main(),
            ] {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(entry.json) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes.push(Ok(node));
                }
            }
            nodes
        };
        let other = Order::<Infallible, _>::new(nodes);
        let nodes = {
            let mut nodes = vec![];
            for entry in [
                crate::testing::corpus::entry::main(),
                crate::testing::corpus::entry::foo(),
                crate::testing::corpus::entry::bar(),
                crate::testing::corpus::entry::foo_part1(),
                crate::testing::corpus::entry::foo_part2(),
            ] {
                let state = r5::parsers::State::default();
                let mut stream = ParseStream::new(entry.path, entry.json.as_ref(), state);
                let file = r5::parsers::dep_file(&mut stream).unwrap();
                for info in file.rules {
                    let cart = Arc::new(entry.json) as DepInfoCart;
                    let node = Yoke::attach_to_cart(cart, |_| info);
                    nodes.push(Ok(node));
                }
            }
            nodes
        };
        let other = other.collect::<Vec<_>>();
        let order = Order::new(nodes).trace(other);
        assert!(order.validate().is_some());
    }

    #[test]
    fn cpp_deps() {
        let out_dir = tempdir::TempDir::new("cpp_deps::order::test::cpp_deps").unwrap();
        let out_dir = out_dir.path();
        crate::testing::setup::build_script_env(out_dir).unwrap();
        let entries = [
            crate::testing::corpus::entry::bar(),
            crate::testing::corpus::entry::foo_part1(),
            crate::testing::corpus::entry::foo_part2(),
            crate::testing::corpus::entry::foo(),
            crate::testing::corpus::entry::main(),
        ]
        .into_iter()
        .map(|entry| (entry.path, entry.json));
        let cpp_deps = CppDepsBuilder::new().unwrap();
        let cpp_deps = cpp_deps.dep_bytes(entries);
        let cpp_deps = cpp_deps.build();
        let order = Order::new(cpp_deps);
        assert!(order.validate().is_some());
    }

    #[cfg(feature = "async")]
    #[test]
    fn cpp_deps_with_sink_async() {
        use futures_util::sink::SinkExt;
        let out_dir = tempdir::TempDir::new("cpp_deps::order::test::cpp_deps_with_sink_async").unwrap();
        let out_dir = out_dir.path();
        crate::testing::setup::build_script_env(out_dir).unwrap();
        let entries = [crate::testing::corpus::entry::main()]
            .into_iter()
            .map(|entry| (entry.path, entry.json));
        let cpp_deps = CppDepsBuilder::new().unwrap();
        let cpp_deps = cpp_deps.dep_bytes(entries);
        let cpp_deps = cpp_deps.build();
        let handle0 = std::thread::spawn({
            let mut sink = cpp_deps.sink();
            move || {
                futures_executor::block_on(async move {
                    for entry in [
                        crate::testing::corpus::entry::foo_part2(),
                        crate::testing::corpus::entry::foo(),
                    ]
                    .into_iter()
                    .map(|entry| (entry.path, entry.json))
                    {
                        let item = crate::CppDepsItem::DepData(entry);
                        sink.send(item).await.unwrap();
                    }
                    sink.flush().await.unwrap()
                })
            }
        });
        let handle1 = std::thread::spawn({
            let mut sink = cpp_deps.sink();
            move || {
                futures_executor::block_on(async move {
                    for entry in [
                        crate::testing::corpus::entry::bar(),
                        crate::testing::corpus::entry::foo_part1(),
                    ]
                    .into_iter()
                    .map(|entry| (entry.path, entry.json))
                    {
                        let item = crate::CppDepsItem::DepData(entry);
                        sink.send(item).await.unwrap();
                    }
                    sink.flush().await.unwrap()
                })
            }
        });
        let order = Order::new(cpp_deps);
        assert!(order.validate().is_some());
        handle0.join().unwrap();
        handle1.join().unwrap();
    }

    #[test]
    fn cpp_deps_with_sink_sync() {
        let out_dir = tempdir::TempDir::new("cpp_deps::order::test::cpp_deps_with_sink_sync").unwrap();
        let out_dir = out_dir.path();
        crate::testing::setup::build_script_env(out_dir).unwrap();
        let entries = [crate::testing::corpus::entry::main()]
            .into_iter()
            .map(|entry| (entry.path, entry.json));
        let cpp_deps = CppDepsBuilder::new().unwrap();
        let cpp_deps = cpp_deps.dep_bytes(entries);
        let cpp_deps = cpp_deps.build();
        let handle0 = std::thread::spawn({
            let sink = cpp_deps.sink();
            move || {
                for entry in [
                    crate::testing::corpus::entry::foo_part2(),
                    crate::testing::corpus::entry::foo(),
                ]
                .into_iter()
                .map(|entry| (entry.path, entry.json))
                {
                    let item = crate::CppDepsItem::DepData(entry);
                    sink.send_sync(item).unwrap();
                }
            }
        });
        let handle1 = std::thread::spawn({
            let sink = cpp_deps.sink();
            move || {
                for entry in [
                    crate::testing::corpus::entry::bar(),
                    crate::testing::corpus::entry::foo_part1(),
                ]
                .into_iter()
                .map(|entry| (entry.path, entry.json))
                {
                    let item = crate::CppDepsItem::DepData(entry);
                    sink.send_sync(item).unwrap();
                }
            }
        });
        let order = Order::new(cpp_deps);
        assert!(order.validate().is_some());
        handle0.join().unwrap();
        handle1.join().unwrap();
    }
}
