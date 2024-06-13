use std::{collections::BTreeSet, fs::File, sync::Arc};

use memmap2::Mmap;
use p1689::r5::{
    self,
    yoke::{DepFileCart, DepFileYokeExt, DepInfoYoke},
};
use yoke::Yoke;

use crate::{
    analyzer::{AnalyzeNode, AnalyzerItem, WorkerItem},
    CppDepsItem,
    CppDepsSrc,
    InnerError,
    InnerErrorKind,
};
#[cfg(feature = "cc")]
use crate::{
    analyzer::{CompileNode, ResolveNode},
    compiler::Compiler,
};

pub struct Worker<P, B> {
    failure_tx: flume::Sender<InnerError>,
    analyze_tx: flume::Sender<AnalyzerItem<P>>,
    compile_rx: flume::Receiver<WorkerItem<P, B>>,
    #[cfg(feature = "cc")]
    compiler: Arc<Compiler>,
}

impl<P, B> Worker<P, B>
where
    P: AsRef<r5::Utf8Path> + Send + Sync + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
{
    pub(crate) fn new(
        failure_tx: flume::Sender<InnerError>,
        analyze_tx: flume::Sender<AnalyzerItem<P>>,
        compile_rx: flume::Receiver<WorkerItem<P, B>>,
        #[cfg(feature = "cc")] compiler: Arc<Compiler>,
    ) -> Self {
        Self {
            failure_tx,
            analyze_tx,
            compile_rx,
            #[cfg(feature = "cc")]
            compiler,
        }
    }

    pub(crate) fn run(mut self) -> impl FnOnce() {
        move || {
            while let Ok(item) = self.compile_rx.recv() {
                if let Err(err) = self.step(item) {
                    self.failure_tx.send(err).ok();
                }
            }
        }
    }

    fn step(&mut self, item: WorkerItem<P, B>) -> Result<(), InnerError> {
        match item {
            WorkerItem::Analyze(item) => self.analyze(item),
            #[cfg(feature = "cc")]
            WorkerItem::Compile(node) => self.compile(node),
            WorkerItem::Expects(count) => self.expects(count),
        }
    }

    fn analyze(&mut self, item: CppDepsItem<P, B>) -> Result<(), InnerError> {
        match item {
            #[cfg(feature = "cc")]
            CppDepsItem::SrcFile { src_file } => self.analyze_src_file(src_file)?,
            CppDepsItem::DepFile { src_file, dep_path } => self.analyze_dep_file(src_file, dep_path)?,
            CppDepsItem::DepText {
                src_file,
                dep_path,
                dep_text,
            } => self.analyze_dep_text(src_file, dep_path, dep_text)?,
            CppDepsItem::DepInfo { src_file, dep_info } => self.analyze_dep_info(src_file, dep_info)?,
        }
        Ok(())
    }

    #[cfg(feature = "cc")]
    fn analyze_src_file(&mut self, src_file: CppDepsSrc<P>) -> Result<(), InnerError> {
        let src_base = src_file.src_base.as_ref();
        let src_path = src_file.src_path.as_ref();
        let dep_path = self.compiler.compile_dep_file(src_base, src_path)?;
        let file = File::open(&dep_path).map_err(|err| InnerError::new(InnerErrorKind::FileOpen { err }))?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|err| InnerError::new(InnerErrorKind::MmapMap { err }))?;
        let cart = Arc::new(mmap) as DepFileCart;
        self.parse_dep_file(Some(src_file), dep_path, cart)?;
        Ok(())
    }

    fn analyze_dep_file(&mut self, src_file: Option<CppDepsSrc<P>>, dep_path: P) -> Result<(), InnerError> {
        let file = {
            let path = AsRef::<r5::Utf8Path>::as_ref(&dep_path);
            let path = AsRef::<std::path::Path>::as_ref(&path);
            File::open(path).map_err(|err| InnerError::new(InnerErrorKind::FileOpen { err }))
        }?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|err| InnerError::new(InnerErrorKind::MmapMap { err }))?;
        let cart = Arc::new(mmap) as DepFileCart;
        self.parse_dep_file(src_file, dep_path, cart)?;
        Ok(())
    }

    fn analyze_dep_text(
        &mut self,
        src_file: Option<CppDepsSrc<P>>,
        dep_path: P,
        dep_text: B,
    ) -> Result<(), InnerError> {
        let cart = Arc::new(dep_text) as DepFileCart;
        self.parse_dep_file(src_file, dep_path, cart)?;
        Ok(())
    }

    fn analyze_dep_info(&mut self, src_file: Option<CppDepsSrc<P>>, dep_info: DepInfoYoke) -> Result<(), InnerError> {
        let src_file = src_file.map(Arc::new);
        let bmi_dirs = BTreeSet::default();
        let bmi_maps = Vec::default();
        let item = AnalyzerItem::Analyze(AnalyzeNode {
            src_file,
            dep_info,
            bmi_dirs,
            bmi_maps,
        });
        self.analyze_tx
            .send(item)
            .map_err(|_| InnerError::new(InnerErrorKind::WorkerFailedSendingAnalyzerItem))?;
        Ok(())
    }

    #[cfg(feature = "cc")]
    fn compile(&mut self, item: CompileNode<P>) -> Result<(), InnerError> {
        let CompileNode {
            src_file,
            dep_info,
            mut bmi_dirs,
            bmi_maps,
        } = item;
        let src_base = src_file.src_base.as_ref();
        let src_path = src_file.src_path.as_ref();
        let obj_path = self
            .compiler
            .compile_obj_file(dep_info.get(), src_base, src_path, &mut bmi_dirs, &bmi_maps)?;
        let bmi_path = self.compiler.bmi_path(obj_path, &dep_info.get().provides);
        let node = ResolveNode {
            bmi_path,
            dep_info,
            bmi_dirs,
            bmi_maps,
        };
        let item = AnalyzerItem::Resolve(node);
        self.analyze_tx
            .send(item)
            .map_err(|_| InnerError::new(InnerErrorKind::WorkerFailedSendingAnalyzerItem))?;
        Ok(())
    }

    fn expects(&self, count: usize) -> Result<(), InnerError> {
        let item = AnalyzerItem::Expects(count);
        self.analyze_tx
            .send(item)
            .map_err(|_| InnerError::new(InnerErrorKind::WorkerFailedSendingExpects))?;
        Ok(())
    }

    fn parse_dep_file<Path>(
        &self,
        src_file: Option<CppDepsSrc<P>>,
        dep_path: Path,
        dep_cart: DepFileCart,
    ) -> Result<(), InnerError>
    where
        Path: AsRef<r5::Utf8Path>,
    {
        let src_file = src_file.map(Arc::new);
        let dep_file =
            Yoke::<&'static _, DepFileCart>::attach_to_cart(dep_cart, |cart| cart).try_map_project(|dep_text, _| {
                let state = r5::parsers::State::default();
                let mut stream = r5::parsers::ParseStream::new(dep_path.as_ref(), dep_text.as_ref(), state);
                r5::parsers::dep_file(&mut stream).map_err(|_| InnerError::new(InnerErrorKind::DepFileParse))
            })?;
        for dep_info in dep_file.rules() {
            let src_file = src_file.clone();
            let bmi_dirs = BTreeSet::default();
            let bmi_maps = Vec::default();
            let item = AnalyzerItem::Analyze(AnalyzeNode {
                src_file,
                dep_info,
                bmi_dirs,
                bmi_maps,
            });
            self.analyze_tx
                .send(item)
                .map_err(|_| InnerError::new(InnerErrorKind::WorkerFailedSendingAnalyzerItem))?;
        }
        Ok(())
    }
}
