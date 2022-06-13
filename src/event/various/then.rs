use crate::{event::{Event, BaseEvent}};
use crate::prelude::Result;

pub struct Then<E: Event, F: Unpin + FnOnce(&mut E::Result)> {
    pub(crate) inner: E,
    #[cfg(feature = "async")]
    pub(crate) f: Option<F>,
    #[cfg(not(feature = "async"))]
    pub(crate) f: F
}

impl<E: Event, F: Unpin + FnOnce(&mut E::Result)> Then<E, F> {
    #[cfg(feature = "async")]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f: Some(f) }
    }

    #[cfg(not(feature = "async"))]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f }
    }
}

impl<E: Event, F: Unpin + FnOnce(&mut E::Result)> Event for Then<E, F> {
    type Result = E::Result;

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
        let mut v = self.inner.wait()?;
        #[cfg(feature = "async")]
        self.f.unwrap()(&mut v);
        #[cfg(not(feature = "async"))]
        (self.f)(&mut v);
        return Ok(v)
    }
}

#[cfg(feature = "async")]
impl<E: Event + Unpin, F: Unpin + FnOnce(&mut E::Result)> futures::Future for Then<E, F> {
    type Output = E::Output;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let core::task::Poll::Ready(mut out) = core::pin::Pin::new(&mut self.inner).poll(cx)? {
            let f = self.f.take().unwrap();
            f(&mut out);
            return core::task::Poll::Ready(Ok(out))
        }

        core::task::Poll::Pending
    }
}

impl<E: Event, F: Unpin + FnOnce(&mut E::Result)> AsRef<BaseEvent> for Then<E, F> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self.inner.borrow_base()
    }
}