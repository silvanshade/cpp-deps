use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_sink::Sink;

use crate::{CppDepsItem, Error, InnerError, InnerErrorKind};

#[derive(Clone)]
#[repr(transparent)]
pub struct CppDepsSink<P, B>
where
    P: 'static,
    B: 'static,
{
    pub(crate) sink: flume::r#async::SendSink<'static, CppDepsItem<P, B>>,
}
impl<P, B> CppDepsSink<P, B> {
    #[inline]
    pub fn send_sync(&self, item: CppDepsItem<P, B>) -> Result<(), Error> {
        self.sink
            .sender()
            .send(item)
            .map_err(|_| Error::from(InnerError::new(InnerErrorKind::SinkFailedSendingCppDepsItem)))
    }
}

#[cfg(feature = "async")]
impl<P, B> Sink<CppDepsItem<P, B>> for CppDepsSink<P, B> {
    type Error = Error;

    #[inline]
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.sink;
        Pin::new(this)
            .poll_ready(cx)
            .map_err(|_| Error::from(InnerError::new(InnerErrorKind::SinkFailedSendingCppDepsItem)))
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: CppDepsItem<P, B>) -> Result<(), Self::Error> {
        let this = &mut self.sink;
        Pin::new(this)
            .start_send(item)
            .map_err(|_| Error::from(InnerError::new(InnerErrorKind::SinkFailedSendingCppDepsItem)))
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.sink;
        Pin::new(this)
            .poll_flush(cx)
            .map_err(|_| Error::from(InnerError::new(InnerErrorKind::SinkFailedSendingCppDepsItem)))
    }

    #[inline]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = &mut self.sink;
        Pin::new(this)
            .poll_close(cx)
            .map_err(|_| Error::from(InnerError::new(InnerErrorKind::SinkFailedSendingCppDepsItem)))
    }
}

#[cfg(test)]
mod test {
    use crate::testing::{BoxError, BoxResult};

    #[cfg(feature = "async")]
    #[test]
    fn cpp_deps_with_sink_async() -> BoxResult<()> {
        use futures_util::{sink::SinkExt, FutureExt};
        let items = [crate::testing::corpus::dep_text::main()];
        let validate = crate::testing::corpus::dep_text::validate_order(items)?;
        let mut futures = vec![];
        futures.push(
            {
                let mut sink = validate.cpp_deps.sink();
                async move {
                    for item in [
                        crate::testing::corpus::dep_text::foo_part2(),
                        crate::testing::corpus::dep_text::foo(),
                    ] {
                        sink.send(item).await.ok();
                    }
                    sink.flush().await.ok();
                }
            }
            .boxed(),
        );
        futures.push(
            {
                let mut sink = validate.cpp_deps.sink();
                async move {
                    for item in [
                        crate::testing::corpus::dep_text::bar(),
                        crate::testing::corpus::dep_text::foo_part1(),
                    ] {
                        sink.send(item).await.ok();
                    }
                    sink.flush().await.ok();
                }
            }
            .boxed(),
        );
        futures_executor::block_on(async move {
            futures_util::future::join_all(futures).await;
        });
        validate.run()
    }

    #[cfg(feature = "async")]
    #[test]
    #[should_panic]
    fn cpp_deps_with_sink_async_incomplete() {
        fn inner() -> BoxResult<()> {
            use futures_util::{sink::SinkExt, FutureExt};
            let items = [crate::testing::corpus::dep_text::main()];
            let validate = crate::testing::corpus::dep_text::validate_order(items)?;
            let mut futures = vec![];
            futures.push(
                {
                    let mut sink = validate.cpp_deps.sink();
                    async move {
                        for item in [
                            crate::testing::corpus::dep_text::foo_part2(),
                            crate::testing::corpus::dep_text::foo(),
                        ] {
                            sink.send(item).await.ok();
                        }
                        sink.flush().await.ok();
                    }
                }
                .boxed(),
            );
            futures_executor::block_on(async move {
                futures_util::future::join_all(futures).await;
            });
            validate.run()
        }
        inner().unwrap()
    }

    #[test]
    fn cpp_deps_with_sink_sync() -> BoxResult<()> {
        let items = [crate::testing::corpus::dep_text::main()];
        let validate = crate::testing::corpus::dep_text::validate_order(items)?;
        let mut threads = vec![];
        threads.push({
            let sink = validate.cpp_deps.sink();
            std::thread::spawn(move || {
                for item in [
                    crate::testing::corpus::dep_text::foo_part2(),
                    crate::testing::corpus::dep_text::foo(),
                ] {
                    sink.send_sync(item).ok();
                }
            })
        });
        threads.push({
            let sink = validate.cpp_deps.sink();
            std::thread::spawn(move || {
                for item in [
                    crate::testing::corpus::dep_text::bar(),
                    crate::testing::corpus::dep_text::foo_part1(),
                ] {
                    sink.send_sync(item).ok();
                }
            })
        });
        threads
            .into_iter()
            .try_for_each(std::thread::JoinHandle::join)
            .map_err(|_| BoxError::from("std::thread::join failed"))?;
        validate.run()
    }

    #[test]
    #[should_panic]
    fn cpp_deps_with_sink_sync_incomplete() {
        fn inner() -> BoxResult<()> {
            let items = [crate::testing::corpus::dep_text::main()];
            let validate = crate::testing::corpus::dep_text::validate_order(items)?;
            let mut threads = vec![];
            threads.push({
                let sink = validate.cpp_deps.sink();
                std::thread::spawn(move || {
                    for item in [
                        crate::testing::corpus::dep_text::foo_part2(),
                        crate::testing::corpus::dep_text::foo(),
                    ] {
                        sink.send_sync(item).ok();
                    }
                })
            });
            threads
                .into_iter()
                .try_for_each(std::thread::JoinHandle::join)
                .map_err(|_| BoxError::from("std::thread::join failed"))?;
            validate.run()
        }
        inner().unwrap()
    }
}
