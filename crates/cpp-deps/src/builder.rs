use core::{marker::PhantomData, num::NonZeroUsize};
use std::sync::Arc;

use p1689::r5;

use crate::{Compiler, CppDeps, CppDepsError, CppDepsItem, DepInfoYoke};

pub struct CppDepsBuilder<P = r5::Utf8PathBuf, B = Vec<u8>, I = core::iter::Empty<CppDepsItem<P, B>>> {
    iter: I,
    size_hint: usize,
    #[cfg(feature = "cc")]
    compiler: Arc<Compiler>,
    parallelism: NonZeroUsize,
    p: PhantomData<P>,
    b: PhantomData<B>,
}
impl<P, B> CppDepsBuilder<P, B> {
    pub fn new() -> Result<CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>, CppDepsError> {
        let iter = core::iter::empty();
        let parallelism = std::thread::available_parallelism()
            .map_err(CppDepsError::IoError)
            .or(NonZeroUsize::try_from(1).map_err(CppDepsError::InvalidParallelism))?;
        #[cfg(feature = "cc")]
        let compiler = {
            let build = cc::Build::default();
            let compiler = Compiler::new(&build)?;
            Arc::from(compiler)
        };
        Ok(Self {
            iter,
            size_hint: 0,
            parallelism,
            #[cfg(feature = "cc")]
            compiler,
            p: PhantomData,
            b: PhantomData,
        })
    }
}
impl<P, B, I> CppDepsBuilder<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    pub fn build(self) -> CppDeps<P, B, impl Iterator<Item = CppDepsItem<P, B>>> {
        let parallelism = self.parallelism.get();
        let (item_tx, item_rx) = flume::bounded(parallelism);
        CppDeps {
            iter: self.iter,
            size_hint: self.size_hint,
            item_tx,
            item_rx,
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    #[cfg(feature = "cc")]
    pub fn compiler(mut self, build: &crate::vendor::cc::Build) -> Result<Self, CppDepsError> {
        let compiler = self::Compiler::new(build)?;
        self.compiler = Arc::from(compiler);
        Ok(self)
    }

    pub fn parallelism(
        mut self,
        parallelism: usize,
    ) -> Result<CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>, CppDepsError> {
        let parallelism = NonZeroUsize::try_from(parallelism).map_err(CppDepsError::InvalidParallelism)?;
        self.parallelism = parallelism;
        Ok(self)
    }

    #[cfg(feature = "cc")]
    pub fn cpp_paths<Ps>(self, paths: Ps) -> CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Ps: IntoIterator<Item = P>,
    {
        let cpp_paths = paths.into_iter().map(CppDepsItem::CppPath);
        let (size_min, size_max) = cpp_paths.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDepsBuilder {
            iter: self.iter.chain(cpp_paths),
            size_hint,
            parallelism: self.parallelism,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    pub fn dep_paths<Ps>(self, paths: Ps) -> CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Ps: IntoIterator<Item = P>,
    {
        let dep_paths = paths.into_iter().map(CppDepsItem::DepPath);
        let (size_min, size_max) = dep_paths.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDepsBuilder {
            iter: self.iter.chain(dep_paths),
            size_hint,
            parallelism: self.parallelism,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    pub fn dep_bytes<Bs>(self, bytes: Bs) -> CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Bs: IntoIterator<Item = (P, B)>,
    {
        let dep_bytes = bytes.into_iter().map(CppDepsItem::DepData);
        // FIXME: remove size
        let (size_min, size_max) = dep_bytes.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDepsBuilder {
            iter: self.iter.chain(dep_bytes),
            size_hint,
            parallelism: self.parallelism,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }

    pub fn dep_nodes<Is>(self, infos: Is) -> CppDepsBuilder<P, B, impl Iterator<Item = CppDepsItem<P, B>>>
    where
        Is: IntoIterator<Item = DepInfoYoke>,
    {
        let dep_infos = infos.into_iter().map(CppDepsItem::DepInfo);
        // FIXME: remove size
        let (size_min, size_max) = dep_infos.size_hint();
        let size_hint = size_max.unwrap_or(size_min);
        CppDepsBuilder {
            iter: self.iter.chain(dep_infos),
            size_hint,
            parallelism: self.parallelism,
            #[cfg(feature = "cc")]
            compiler: self.compiler,
            p: PhantomData,
            b: PhantomData,
        }
    }
}
