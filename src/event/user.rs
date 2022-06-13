use alloc::format;
use cl_sys::{clCreateUserEvent, clSetUserEventStatus, CL_COMPLETE};
use crate::prelude::{Result, Error, Context};
use super::{BaseEvent, Event};

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UserEvent (BaseEvent);

impl UserEvent {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn new () -> Result<Self> {
        Self::with_context(Context::default())
    }

    #[inline]
    pub fn with_context (ctx: &Context) -> Result<Self> {
        let mut err = 0;
        let id = unsafe {
            clCreateUserEvent(ctx.0, &mut err)
        };

        if err == 0 {
            let inner = BaseEvent::new(id)?;
            return Ok(Self(inner));
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidContext => report.attach_printable(format!("'{:?}' is not a valid context", id)),
                    Error::OutOfResources => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the device"),
                    Error::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }

    #[inline(always)]
    pub fn set_status (&self, complete: Option<Error>) -> Result<()> {
        let status = match complete {
            None => CL_COMPLETE as i32,
            Some(e) => e.into(),
        };

        let err = unsafe {
            clSetUserEventStatus(self.0.0, status)
        };

        if err == 0 { return Ok(()) }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidEvent => report.attach_printable(format!("'{:?}' is not a valid event", self.0.0)),
                    Error::InvalidOperation => report.attach_printable("the execution status for event has already been changed by a previous call"),
                    Error::OutOfResources => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the device"),
                    Error::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
}

impl Event for UserEvent {
    type Result = ();

    #[inline(always)]
    fn wait (self) -> Result<Self::Result> {
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
    type Output = Result<()>;

    #[inline(always)]
    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        core::pin::Pin::new(&mut self.0).poll(cx)
    }
}