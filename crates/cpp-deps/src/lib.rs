// TODO:
// 1. move non-compilation out of threads?
// 2. optimize for single threaded case? (depending on workload size, process non-process spawning jobs on main non-spawned main thread, replacing `feed_loop`)
// 3. simplify exposed types (hide yoke, arcs, etc).
// 4. consolidate errors
// 5. test feature combinations: (w/o async, w/o cc)
// 6. re-distribute tests across modules (don't put everything under `crate::order`)

mod builder;
mod compiler;
mod order;
mod queue;
mod vendor;
mod worker;

use std::sync::Arc;

use p1689::r5;
use yoke::Yoke;

type DepInfoCart = Arc<dyn AsRef<[u8]> + Send + Sync + 'static>;
type DepInfoYoke = Yoke<r5::DepInfo<'static>, DepInfoCart>;

use core::marker::PhantomData;

use crate::compiler::Compiler;
pub use crate::{
    builder::CppDepsBuilder,
    compiler::CompilerError,
    order::OrderError,
    queue::{CppDepsIter, CppDepsIterError, CppDepsIterSink},
    worker::WorkerError,
};

pub enum ThreadError {
    SendError,
    WorkerError(WorkerError),
}

pub enum CppDepsError {
    Compiler(CompilerError),
    IoError(std::io::Error),
    InvalidParallelism(core::num::TryFromIntError),
    VarError(std::env::VarError),
}
impl From<CompilerError> for CppDepsError {
    fn from(error: CompilerError) -> Self {
        CppDepsError::Compiler(error)
    }
}
impl From<std::env::VarError> for CppDepsError {
    fn from(error: std::env::VarError) -> Self {
        CppDepsError::VarError(error)
    }
}
impl core::fmt::Debug for CppDepsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => core::fmt::Debug::fmt(&error, f),
            Self::IoError(error) => core::fmt::Debug::fmt(&error, f),
            Self::InvalidParallelism(error) => core::fmt::Debug::fmt(&error, f),
            Self::VarError(error) => core::fmt::Debug::fmt(&error, f),
        }
    }
}
impl core::fmt::Display for CppDepsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => core::fmt::Display::fmt(&error, f),
            Self::IoError(error) => core::fmt::Display::fmt(&error, f),
            Self::InvalidParallelism(error) => core::fmt::Display::fmt(&error, f),
            Self::VarError(error) => core::fmt::Display::fmt(&error, f),
        }
    }
}
impl std::error::Error for CppDepsError {}

pub enum CppDepsItem<P = r5::Utf8PathBuf, B = Vec<u8>> {
    #[cfg(feature = "cc")]
    CppPath(P),
    DepPath(P),
    DepData((P, B)),
    DepInfo(DepInfoYoke),
}

pub struct CppDeps<P = r5::Utf8PathBuf, B = Vec<u8>, I = core::iter::Empty<CppDepsItem<P, B>>> {
    iter: I,
    size_hint: usize,
    item_tx: flume::Sender<CppDepsItem<P, B>>,
    item_rx: flume::Receiver<CppDepsItem<P, B>>,
    capacity: usize,
    #[cfg(feature = "cc")]
    compiler: Arc<Compiler>,
    p: PhantomData<P>,
    b: PhantomData<B>,
}
impl<P, B, I> CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    fn fanout_items(
        iter: I,
        item_tx: &flume::Sender<CppDepsItem<P, B>>,
    ) -> impl FnOnce() -> Result<(), ThreadError> + Send + 'static {
        let item_tx = item_tx.clone();
        move || {
            for item in iter {
                item_tx.send(item).map_err(|_| ThreadError::SendError)?;
            }
            Ok(())
        }
    }

    pub fn sink(&self) -> CppDepsIterSink<P, B>
    where
        P: 'static,
        B: 'static,
    {
        let sender = self.item_tx.clone();
        let sink = flume::Sender::into_sink(sender);
        CppDepsIterSink(sink)
    }
}
