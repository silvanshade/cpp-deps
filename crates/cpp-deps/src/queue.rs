use core::num::NonZeroUsize;

use futures_core::Stream;
use futures_sink::Sink;
use p1689::r5;

use crate::{
    order::{Order, OrderError},
    worker::{CppDepsWorker, WorkerError},
    CppDeps,
    CppDepsItem,
    DepInfoYoke,
    ThreadError,
};

impl<P, B, I> CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    pub fn items(self) -> CppDepsIter<P, B> {
        let par = std::thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
            .min(self.size_hint);
        let (item_tx, item_rx) = flume::bounded(par);
        let (info_tx, info_rx) = flume::unbounded();
        let threads = (0 .. par)
            .map(|_| {
                let worker = CppDepsWorker::new(
                    &item_rx,
                    &info_tx,
                    #[cfg(feature = "cc")]
                    self.compiler.clone(),
                );
                std::thread::spawn(worker.run())
            })
            .chain(core::iter::once(std::thread::spawn(Self::feed_loop(
                &item_tx, self.iter,
            ))))
            .collect();
        CppDepsIter {
            item_tx: item_tx.downgrade(),
            info_rx,
            threads,
        }
    }

    #[inline]
    pub fn toposort(self) -> impl Iterator<Item = Result<DepInfoYoke, OrderError<CppDepsIterError>>> {
        Order::new(self.items())
    }
}

#[non_exhaustive]
pub enum CppDepsIterError {
    ThreadJoinError,
    WorkerError(WorkerError),
}

pub struct CppDepsIter<P, B> {
    item_tx: flume::WeakSender<CppDepsItem<P, B>>,
    info_rx: flume::Receiver<Result<DepInfoYoke, WorkerError>>,
    threads: Vec<std::thread::JoinHandle<Result<(), ThreadError>>>,
}
impl<P, B> CppDepsIter<P, B> {}
impl<P, B> Iterator for CppDepsIter<P, B> {
    type Item = Result<DepInfoYoke, CppDepsIterError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.info_rx.recv().ok() {
            // Process item and forward errors
            Some(res) => match res {
                Err(err) => Some(Err(CppDepsIterError::WorkerError(err))),
                Ok(item) => Some(Ok(item)),
            },
            // All items processed so try joining thread handles
            None => {
                while let Some(thread) = self.threads.pop() {
                    if thread.join().is_err() {
                        return Some(Err(CppDepsIterError::ThreadJoinError));
                    }
                }
                None
            },
        }
    }
}
impl<P, B> CppDepsIter<P, B> {
    pub fn sink(&self) -> Option<impl Sink<CppDepsItem<P, B>>>
    where
        P: 'static,
        B: 'static,
    {
        self.item_tx.upgrade().map(flume::Sender::into_sink)
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> {
        self.info_rx.into_stream()
    }

    pub fn stream(&self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> + '_ {
        self.info_rx.stream()
    }
}
