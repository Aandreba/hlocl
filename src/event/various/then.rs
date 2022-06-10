use crate::{event::{Event, BaseEvent}, prelude::ErrorCL};

pub struct Then<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> {
    pub(crate) inner: E,
    #[cfg(feature = "async")]
    pub(crate) f: Option<F>,
    #[cfg(not(feature = "async"))]
    pub(crate) f: F
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> Then<O, E, F> {
    #[cfg(feature = "async")]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f: Some(f) }
    }

    #[cfg(not(feature = "async"))]
    pub fn new (inner: E, f: F) -> Self {
        Self { inner, f }
    }
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> Event for Then<O, E, F> {
    type Result = O;

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, crate::prelude::ErrorCL> {
        let v = self.inner.wait()?;
        #[cfg(feature = "async")]
        return Ok(self.f.unwrap()(v));
        #[cfg(not(feature = "async"))]
        Ok((self.f)(v))
    }
}

#[cfg(feature = "async")]
impl<O, E: Event + Unpin, F: Unpin + FnOnce(E::Result) -> O> futures::Future for Then<O, E, F> {
    type Output = Result<O, ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let core::task::Poll::Ready(out) = core::pin::Pin::new(&mut self.inner).poll(cx)? {
            let f = self.f.take().unwrap();
            return core::task::Poll::Ready(Ok(f(out)))
        }

        core::task::Poll::Pending
    }
}

impl<O, E: Event, F: Unpin + FnOnce(E::Result) -> O> AsRef<BaseEvent> for Then<O, E, F> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self.inner.borrow_base()
    }
}