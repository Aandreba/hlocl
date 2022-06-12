use cl_sys::{clCreateUserEvent, clSetUserEventStatus, CL_COMPLETE};
use crate::prelude::{ErrorCL, Context};
use super::{BaseEvent, Event};

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UserEvent (BaseEvent);

impl UserEvent {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new () -> Result<Self, ErrorCL> {
        Self::with_context(Context::default())
    }

    #[inline(always)]
    pub fn with_context (ctx: &Context) -> Result<Self, ErrorCL> {
        let mut err = 0;
        let id = unsafe {
            clCreateUserEvent(ctx.0, &mut err)
        };

        if err == 0 {
            return Err(ErrorCL::from(err));
        }

        let id = BaseEvent::new(id)?;
        Ok(Self(id))
    }

    #[inline(always)]
    pub fn set_status (&self, complete: Result<(), ErrorCL>) -> Result<(), ErrorCL> {
        let status = match complete {
            Ok(_) => CL_COMPLETE as i32,
            Err(e) => e.ty().into(),
        };

        let err = unsafe {
            clSetUserEventStatus(self.0.0, status)
        };

        if err == 0 {
            return Ok(())
        }

        Err(ErrorCL::from(err))
    }
}

impl Event for UserEvent {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result, ErrorCL> {
        self.0.wait()
    }
}

impl AsRef<BaseEvent> for UserEvent {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        &self.0
    }
}

#[cfg(feature = "async")]
impl futures::Future for UserEvent {
    type Output = Result<(), ErrorCL>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.0).poll(cx)
    }
}