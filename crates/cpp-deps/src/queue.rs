use core::{
    num::NonZeroUsize,
    pin::Pin,
    task::{Context, Poll},
};

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
            item_tx: Some(item_tx),
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
    item_tx: Option<flume::Sender<CppDepsItem<P, B>>>,
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
                self.item_tx.take();
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
    pub fn sink(&self) -> Option<CppDepsIterSink<P, B>>
    where
        P: 'static,
        B: 'static,
    {
        self.item_tx
            .upgrade()
            .map(flume::Sender::into_sink)
            .map(CppDepsIterSink)
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> {
        self.info_rx.into_stream()
    }

    pub fn stream(&self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> + '_ {
        self.info_rx.stream()
    }
}

#[repr(transparent)]
pub struct CppDepsIterSink<P, B>(flume::r#async::SendSink<'static, CppDepsItem<P, B>>)
where
    P: 'static,
    B: 'static;
impl<P, B> Sink<CppDepsItem<P, B>> for CppDepsIterSink<P, B> {
    type Error = CppDepsIterSinkError<P, B>;

    #[inline]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this).poll_ready(cx).map_err(CppDepsIterSinkError)
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: CppDepsItem<P, B>) -> Result<(), Self::Error> {
        let this = &mut self.0;
        Pin::new(this).start_send(item).map_err(CppDepsIterSinkError)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this).poll_flush(cx).map_err(CppDepsIterSinkError)
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this).poll_close(cx).map_err(CppDepsIterSinkError)
    }
}

pub struct CppDepsIterSinkError<P, B>(flume::SendError<CppDepsItem<P, B>>);

impl<P, B> core::fmt::Debug for CppDepsIterSinkError<P, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl<P, B> core::fmt::Display for CppDepsIterSinkError<P, B> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl<P, B> std::error::Error for CppDepsIterSinkError<P, B> {}
