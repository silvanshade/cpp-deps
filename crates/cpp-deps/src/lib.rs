// TODO:
// 1. move non-compilation out of threads?
// 2. optimize for single threaded case? (depending on workload size, process non-process spawning jobs on main non-spawned main thread, replacing `feed_loop`)
// 3. simplify exposed types (hide yoke, arcs, etc).
// 4. consolidate errors
// 5. test feature combinations: (w/o async, w/o cc)

mod compiler;
mod order;
mod queue;
mod vendor;
mod worker;

use std::sync::Arc;

use p1689::r5;
use yoke::Yoke;

pub use crate::{
    compiler::CompilerError,
    order::OrderError,
    queue::{CppDepsIter, CppDepsIterError},
    worker::WorkerError,
};

type DepInfoCart = Arc<dyn AsRef<[u8]> + Send + Sync + 'static>;
type DepInfoYoke = Yoke<r5::DepInfo<'static>, DepInfoCart>;

use core::marker::PhantomData;

use crate::compiler::Compiler;

pub enum ThreadError {
    SendError,
    WorkerError(WorkerError),
}

pub enum CppDepsError {
    Compiler(CompilerError),
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
            Self::VarError(error) => core::fmt::Debug::fmt(&error, f),
        }
    }
}
impl core::fmt::Display for CppDepsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compiler(error) => core::fmt::Display::fmt(&error, f),
            Self::VarError(error) => core::fmt::Display::fmt(&error, f),
        }
    }
}
impl std::error::Error for CppDepsError {}

pub struct CppDeps<P = r5::Utf8PathBuf, B = Vec<u8>, I = core::iter::Empty<CppDepsItem<P, B>>> {
    iter: I,
    size_hint: usize,
    #[cfg(feature = "cc")]
    compiler: Arc<Compiler>,
    p: PhantomData<P>,
    b: PhantomData<B>,
}
impl<P, B> CppDeps<P, B> {
    pub fn new(
        #[cfg(feature = "cc")] cc: &crate::vendor::cc::Build,
    ) -> Result<CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>>, CppDepsError> {
        #[cfg(feature = "cc")]
        let compiler = Arc::from(Compiler::new(cc)?);
        Ok(Self {
            iter: core::iter::empty(),
            size_hint: 0,
            #[cfg(feature = "cc")]
            compiler,
            p: PhantomData,
            b: PhantomData,
        })
    }
}
impl<P, B, I> CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    #[cfg(feature = "cc")]
    #[inline]
    pub fn add_cpp_paths<Ps>(self, paths: Ps) -> CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Ps: IntoIterator<Item = P>,
    {
        let cpp_paths = paths.into_iter().map(CppDepsItem::CppPath);
        let (size_min, size_max) = cpp_paths.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDeps {
            iter: self.iter.chain(cpp_paths),
            size_hint,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    #[inline]
    pub fn add_dep_paths<Ps>(self, paths: Ps) -> CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Ps: IntoIterator<Item = P>,
    {
        let dep_paths = paths.into_iter().map(CppDepsItem::DepPath);
        let (size_min, size_max) = dep_paths.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDeps {
            iter: self.iter.chain(dep_paths),
            size_hint,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    #[inline]
    pub fn add_dep_bytes<Bs>(self, bytes: Bs) -> CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Bs: IntoIterator<Item = (P, B)>,
    {
        let dep_bytes = bytes.into_iter().map(CppDepsItem::DepData);
        // FIXME: remove size
        let (size_min, size_max) = dep_bytes.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDeps {
            iter: self.iter.chain(dep_bytes),
            size_hint,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    #[inline]
    pub fn add_dep_nodes<Is>(self, infos: Is) -> CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Is: IntoIterator<Item = DepInfoYoke>,
    {
        let dep_infos = infos.into_iter().map(CppDepsItem::DepInfo);
        // FIXME: remove size
        let (size_min, size_max) = dep_infos.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDeps {
            iter: self.iter.chain(dep_infos),
            size_hint,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }
}
impl<P, B, I> CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    fn feed_loop(
        item_tx: &flume::Sender<CppDepsItem<P, B>>,
        iter: I,
    ) -> impl FnOnce() -> Result<(), ThreadError> + Send + 'static {
        let item_tx = item_tx.clone();
        move || {
            for item in iter {
                item_tx.send(item).map_err(|_| ThreadError::SendError)?;
            }
            panic!("oops");
            Ok(())
        }
    }
}

pub enum CppDepsItem<P = r5::Utf8PathBuf, B = Vec<u8>> {
    #[cfg(feature = "cc")]
    CppPath(P),
    DepPath(P),
    DepData((P, B)),
    DepInfo(DepInfoYoke),
}
