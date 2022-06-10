use crate::prelude::{Event, BaseEvent, ErrorCL};

pub struct Swap<T, E> {
    inner: E,
    #[cfg(not(feature = "async"))]
    v: T,
    #[cfg(feature = "async")]
    v: Option<T>
}

impl<T: Unpin, E: Event> Swap<T, E> {
    #[cfg(feature = "async")]
    pub fn new (inner: E, v: T) -> Self {
        Self { inner, v: Some(v) }
    }

    #[cfg(not(feature = "async"))]
    pub fn new (inner: E, v: T) -> Self {
        Self { inner, v }
    }
}

impl<T: Unpin, E: Event> Event for Swap<T, E> {
    type Result = T;

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, ErrorCL> {
        self.inner.wait()?;
        #[cfg(feature = "async")]
        return Ok(self.v.unwrap());
        #[cfg(not(feature = "async"))]
        Ok(self.v)
    }
}

#[cfg(feature = "async")]
impl<T: Unpin, E: Event> futures::Future for Swap<T, E> {
    type Output = Result<T, ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        if let core::task::Poll::Ready(_) = core::pin::Pin::new(&mut self.inner).poll(cx)? {
            return core::task::Poll::Ready(Ok(self.v.take().unwrap()))
        }

        core::task::Poll::Pending
    }
}

impl<T, E: Event> AsRef<BaseEvent> for Swap<T, E> {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self.inner.borrow_base()
    }
}