use core::ops::Deref;
use std::{fs::File, sync::Arc};

use memmap2::Mmap;
use p1689::r5;
use yoke::{Yoke, Yokeable};

use crate::{compiler::Compiler, CppDepsItem, DepInfoCart, DepInfoYoke, ThreadError};

type ParseErrorYoke = Yoke<p1689::r5::parsers::Error<'static, p1689::r5::parsers::ErrorKind>, DepInfoCart>;

pub struct CompileArtifacts {
    obj: r5::Utf8PathBuf,
    dep: r5::Utf8PathBuf,
}

#[non_exhaustive]
pub enum WorkerError {
    #[non_exhaustive]
    CompileError {
        path: r5::Utf8PathBuf,
        error: std::io::Error,
    },
    ParseError(ParseErrorYoke),
}

pub struct CppDepsWorker<P, B> {
    item_rx: flume::Receiver<CppDepsItem<P, B>>,
    info_tx: flume::Sender<Result<DepInfoYoke, WorkerError>>,
    #[cfg(feature = "cc")]
    pub compiler: Arc<Compiler>,
}

impl<P, B> CppDepsWorker<P, B>
where
    P: AsRef<r5::Utf8Path> + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
{
    pub fn new(
        item_rx: &flume::Receiver<CppDepsItem<P, B>>,
        info_tx: &flume::Sender<Result<DepInfoYoke, WorkerError>>,
        #[cfg(feature = "cc")] compiler: Arc<Compiler>,
    ) -> Self {
        let item_rx = item_rx.clone();
        let info_tx = info_tx.clone();
        Self {
            item_rx,
            info_tx,
            #[cfg(feature = "cc")]
            compiler,
        }
    }

    pub fn run(mut self) -> impl FnOnce() -> Result<(), ThreadError> {
        move || {
            while let Ok(item) = self.item_rx.recv() {
                self.step(item)?;
            }
            Ok(())
        }
    }

    fn step(&mut self, item: CppDepsItem<P, B>) -> Result<(), ThreadError> {
        match item {
            #[cfg(feature = "cc")]
            CppDepsItem::CppPath(path) => {
                let artifacts = match self.compile(path) {
                    Err(err) => return self.send(Err(err)),
                    Ok(outputs) => outputs,
                };
                let dep_file = &artifacts.dep;
                let file = File::open(dep_file).unwrap();
                let cart = Arc::new(unsafe { Mmap::map(&file) }.unwrap()) as DepInfoCart;
                self.parse(dep_file, cart)?;
            },
            CppDepsItem::DepPath(path) => {
                let file = {
                    let path = AsRef::<r5::Utf8Path>::as_ref(&path);
                    let path = AsRef::<std::path::Path>::as_ref(&path);
                    File::open(path).unwrap()
                };
                let cart = Arc::new(unsafe { Mmap::map(&file) }.unwrap()) as DepInfoCart;
                self.parse(path, cart)?;
            },
            CppDepsItem::DepData((path, data)) => {
                let cart = Arc::new(data) as DepInfoCart;
                self.parse(path, cart)?;
            },
            CppDepsItem::DepInfo(yoke) => {
                self.send(Ok(yoke))?;
            },
        }
        Ok(())
    }

    #[cfg(feature = "cc")]
    fn compile(&mut self, path: P) -> Result<CompileArtifacts, WorkerError> {
        let path = path.as_ref();
        self.compiler
            .tool
            .to_command()
            .arg(path.as_std_path())
            .status()
            .map_err(|error| WorkerError::CompileError {
                path: path.to_path_buf(),
                error,
            })?;
        let obj = path.with_extension(self.compiler.obj_file_ext());
        let dep = path.with_extension(self.compiler.dep_file_ext());
        Ok(CompileArtifacts { obj, dep })
    }

    fn parse<Path>(&self, path: Path, cart: DepInfoCart) -> Result<(), ThreadError>
    where
        Path: AsRef<r5::Utf8Path>,
    {
        let dep_file = {
            let path = path.as_ref();
            let state = r5::parsers::State::default();
            let input = cart.deref().as_ref();
            let mut stream = r5::parsers::ParseStream::new(path, input, state);
            r5::parsers::dep_file(&mut stream)
        }
        .map_err(|err| {
            let yoke = Yoke::attach_to_cart(cart.clone(), |_| unsafe { Yokeable::make(err) });
            ThreadError::WorkerError(WorkerError::ParseError(yoke))
        })?;
        for info in dep_file.rules {
            let cart = cart.clone();
            let yoke = Yoke::attach_to_cart(cart, |_| unsafe { Yokeable::make(info) });
            self.send(Ok(yoke))?;
        }
        Ok(())
    }

    #[inline]
    fn send(&self, node: Result<DepInfoYoke, WorkerError>) -> Result<(), ThreadError> {
        self.info_tx.send(node).map_err(|_| ThreadError::SendError)
    }
}
