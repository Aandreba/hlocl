use alloc::vec::Vec;

use crate::{event::{Event, BaseEvent}};
use crate::prelude::Result;

pub struct Map<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> {
    pub(crate) inner: E,
    #[cfg(feature = "async")]
    pub(crate) f: Option<F>,
    #[cfg(not(feature = "async"))]
    pub(crate) f: F
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> Map<O, E, F> {
    #[cfg(feature = "async")]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f: Some(f) }
    }

    #[cfg(not(feature = "async"))]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f }
    }
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> Event for Map<O, E, F> {
    type Result = O;

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
        let v = self.inner.wait()?;
        #[cfg(feature = "async")]
        return Ok(self.f.unwrap()(v));
        #[cfg(not(feature = "async"))]
        Ok((self.f)(v))
    }

    #[inline(always)]
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<alloc::vec::Vec<Self::Result>> {
        let (inner, f) : (Vec<_>, Vec<_>) = iter.into_iter().map(|x| (x.inner, x.f)).unzip();
        let base = <E as Event>::wait_all(inner)?;

        #[cfg(feature = "async")]
        let result = f.into_iter().zip(base).map(|(f, x)| f.unwrap()(x));
        #[cfg(not(feature = "async"))]
        let result = f.into_iter().zip(base).map(|(f, x)| f(x));

        Ok(result.collect())
    }
}

#[cfg(feature = "async")]
impl<O, E: Event + Unpin, F: Unpin + FnOnce(E::Result) -> O> futures::Future for Map<O, E, F> {
    type Output = Result<O>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let core::task::Poll::Ready(out) = core::pin::Pin::new(&mut self.inner).poll(cx)? {
            let f = self.f.take().unwrap();
            return core::task::Poll::Ready(Ok(f(out)))
        }

        core::task::Poll::Pending
    }
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> AsRef<BaseEvent> for Map<O, E, F> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self.inner.borrow_base()
    }
}