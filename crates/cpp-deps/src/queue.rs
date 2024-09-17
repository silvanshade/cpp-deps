use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_core::Stream;
use futures_sink::Sink;
use p1689::r5::{self, yoke::DepInfoYoke};

use crate::{
    order::{Order, OrderError},
    worker::{CppDepsWorker, WorkerError},
    CppDeps,
    CppDepsItem,
    ThreadError,
};

impl<P, B, I> CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    #[inline]
    pub fn toposort(self) -> impl Iterator<Item = Result<DepInfoYoke, OrderError<CppDepsIterError>>> {
        Order::new(self)
    }
}

impl<P, B, I> IntoIterator for CppDeps<P, B, I>
where
    P: AsRef<r5::Utf8Path> + Send + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    I: Iterator<Item = CppDepsItem<P, B>> + Send + 'static,
{
    type IntoIter = CppDepsIter;
    type Item = <CppDepsIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        let (info_tx, info_rx) = flume::unbounded();
        let mut threads = Vec::with_capacity(self.capacity + 1);
        for _ in 0 .. self.capacity {
            threads.push(std::thread::spawn(
                CppDepsWorker::new(
                    &self.item_rx,
                    &info_tx,
                    #[cfg(feature = "cc")]
                    self.compiler.clone(),
                )
                .run(),
            ));
        }
        threads.push(std::thread::spawn(Self::fanout_items(self.iter, &self.item_tx)));
        CppDepsIter {
            size_hint: self.size_hint,
            info_rx,
            threads,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CppDepsIterError {
    ThreadJoinError,
    WorkerError(WorkerError),
}
impl core::fmt::Display for CppDepsIterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

pub struct CppDepsIter {
    size_hint: usize,
    info_rx: flume::Receiver<Result<DepInfoYoke, WorkerError>>,
    threads: Vec<std::thread::JoinHandle<Result<(), ThreadError>>>,
}
impl CppDepsIter {}
impl Iterator for CppDepsIter {
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self.size_hint;
        let max = self.info_rx.is_disconnected().then_some(min); // If there are no sinks, we know the exact size
        (min, max)
    }
}
impl CppDepsIter {
    pub fn into_stream(self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> {
        self.info_rx.into_stream()
    }

    pub fn stream(&self) -> impl Stream<Item = Result<DepInfoYoke, WorkerError>> + '_ {
        self.info_rx.stream()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct CppDepsIterSink<P, B>(pub(crate) flume::r#async::SendSink<'static, CppDepsItem<P, B>>)
where
    P: 'static,
    B: 'static;
impl<P, B> CppDepsIterSink<P, B> {
    pub fn send_sync(&self, item: CppDepsItem<P, B>) -> Result<(), CppDepsIterSinkError<P, B>> {
        self.0
            .sender()
            .send(item)
            .map_err(|err| CppDepsIterSinkError(Box::new(err)))
    }
}

impl<P, B> Sink<CppDepsItem<P, B>> for CppDepsIterSink<P, B> {
    type Error = CppDepsIterSinkError<P, B>;

    #[inline]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this)
            .poll_ready(cx)
            .map_err(|err| CppDepsIterSinkError(Box::new(err)))
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: CppDepsItem<P, B>) -> Result<(), Self::Error> {
        let this = &mut self.0;
        Pin::new(this)
            .start_send(item)
            .map_err(|err| CppDepsIterSinkError(Box::new(err)))
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this)
            .poll_flush(cx)
            .map_err(|err| CppDepsIterSinkError(Box::new(err)))
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.0;
        Pin::new(this)
            .poll_close(cx)
            .map_err(|err| CppDepsIterSinkError(Box::new(err)))
    }
}

pub struct CppDepsIterSinkError<P, B>(Box<flume::SendError<CppDepsItem<P, B>>>);

impl<P, B> core::fmt::Debug for CppDepsIterSinkError<P, B> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl<P, B> core::fmt::Display for CppDepsIterSinkError<P, B> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl<P, B> std::error::Error for CppDepsIterSinkError<P, B> {}
