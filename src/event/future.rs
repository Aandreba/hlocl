use core::{task::Poll, pin::Pin};
use futures::Future;
use crate::prelude::{Context, ErrorCL};
use super::{UserEvent, BaseEvent};

pub struct FutureEvent<F> {
    fut: F,
    inner: UserEvent
}

impl<F: Future + Unpin> FutureEvent<F> {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new (fut: F) -> Result<Self, ErrorCL> {
        Self::with_context(Context::default(), fut)
    }

    #[inline(always)]
    pub fn with_context (ctx: &Context, fut: F) -> Result<Self, ErrorCL> {
        let inner = UserEvent::with_context(ctx)?;
        Ok(Self {
            fut,
            inner
        })
    }
}

impl<F: Future + Unpin> AsRef<BaseEvent> for FutureEvent<F> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self.inner.as_ref()
    }
}

impl<F: Future + Unpin> Future for FutureEvent<F> {
    type Output = Result<F::Output, ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let Poll::Ready(x) = Pin::new(&mut self.fut).poll(cx) {
            self.inner.set_status(Ok(()))?;
            return Poll::Ready(Ok(x));
        }

        Poll::Pending
    }
}