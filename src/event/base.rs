use core::{mem::MaybeUninit, ptr::addr_of, hash::Hash};
use alloc::{format, vec::Vec};
use cl_sys::{cl_event, cl_event_info, clReleaseEvent, clGetEventInfo, CL_EVENT_COMMAND_QUEUE, CL_EVENT_COMMAND_TYPE, CL_EVENT_COMMAND_EXECUTION_STATUS, clWaitForEvents, clRetainEvent};
use crate::prelude::{Result, Error};
use super::Event;

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use alloc::sync::Arc;
        use futures::task::AtomicWaker;
        use cl_sys::clSetEventCallback;

        pub struct BaseEvent (pub(crate) cl_event, Arc<AtomicWaker>);
    } else {
        #[derive(PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct BaseEvent (pub(crate) cl_event);
    }
}

pub const EMPTY : [BaseEvent;0] = [];

impl BaseEvent {
    #[cfg(feature = "async")]
    pub fn new (id: cl_event) -> Result<Self> {
        let waker = AtomicWaker::new();
        let mut data = Arc::new(waker);
        let ptr = Arc::into_raw(data);

        unsafe {
            Arc::increment_strong_count(ptr);
            data = Arc::from_raw(ptr);

            let err = clSetEventCallback(id, cl_sys::CL_COMPLETE, Some(notify), ptr as *mut _);
            if err != 0 {
                Arc::decrement_strong_count(ptr);

                cfg_if::cfg_if! {
                    if #[cfg(feature = "error-stack")] {
                        let err = Error::from(err);
                        let report = error_stack::Report::new(err);

                        let report = match err {
                            Error::InvalidEvent => report.attach_printable(format!("'{:?}' is not a valid event", id)),
                            Error::OutOfResources => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the device"),
                            Error::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                            _ => report
                        };

                        return Err(report)
                    } else {
                        return Err(Error::from(err))
                    }
                }
            }
        }

        Ok(Self(id, data))
    }

    #[cfg(not(feature = "async"))]
    #[inline(always)]
    pub fn new (id: cl_event) -> Result<Self> {
        Ok(Self(id))
    }

    #[inline]
    fn get_info<T> (&self, id: cl_event_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetEventInfo(self.0, id, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, id, core::mem::size_of::<T>())?;
            Ok(result.assume_init())
        }
    }

    fn parse_error (&self, err: i32, ty: cl_event_info, size: usize) -> Result<()> {
        if err == 0 { return Ok(()); }
        
        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidEvent => report.attach_printable(format!("'{:?}' is not a valid event", self.0)),
                    Error::InvalidValue => report.attach_printable(format!("'{ty}' is not one of the supported values or size in bytes specified by {size} is < size of return type as specified in the table above and '{ty}' is not a NULL value")),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
}

impl Event for BaseEvent {
    type Result = ();
    
    #[inline(always)]
    fn command_queue (&self) -> Result<crate::prelude::CommandQueue> {
        self.get_info(CL_EVENT_COMMAND_QUEUE)
    }

    #[inline(always)]
    fn ty (&self) -> Result<super::CommandType> {
        self.get_info(CL_EVENT_COMMAND_TYPE)
    }

    #[inline(always)]
    fn status (&self) -> Result<super::EventStatus> {
        self.get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)
    }

    #[inline(always)]
    fn wait (self) -> Result<()> {
        let err = unsafe {
            clWaitForEvents(1, addr_of!(self.0))
        };

        if err == 0 { return Ok(()) }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidEvent => report.attach_printable(format!("'{:?}' is not a valid event", self.0)),
                    Error::InvalidValue => report.attach_printable("number of events is zero"),
                    Error::InvalidContext => report.attach_printable("events specified in event list do not belong to the same context"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
    
    #[inline(always)]
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<Vec<()>> {
        let events = iter.into_iter().map(|x| x.0).collect::<Vec<_>>();
        let len = u32::try_from(events.len()).expect("Too many events");

        let err = unsafe {
            clWaitForEvents(len, events.as_ptr())
        };

        if err == 0 { return Ok(alloc::vec![(); events.len()]) }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidEvent => report.attach_printable("invalid event found"),
                    Error::InvalidValue => report.attach_printable("number of events is zero"),
                    Error::InvalidContext => report.attach_printable("events specified in event list do not belong to the same context"),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }
}

impl AsRef<BaseEvent> for BaseEvent {
    #[inline(always)]
    fn as_ref(&self) -> &BaseEvent {
        self
    }
}

#[cfg(not(feature = "async"))]
impl Clone for BaseEvent {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainEvent(self.0));
        }
        
        Self(self.0)
    }
}

#[cfg(feature = "async")]
impl Clone for BaseEvent {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainEvent(self.0));
        }
        
        Self(self.0, self.1.clone())
    }
}

#[cfg(feature = "async")]
impl futures::Future for BaseEvent {
    type Output = Result<()>;

    #[inline(always)]
    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        self.1.register(cx.waker());

        if self.status()? == super::EventStatus::Complete {
            return core::task::Poll::Ready(Ok(()))
        }

        core::task::Poll::Pending
    }
}

#[cfg(feature = "async")]
impl PartialEq for BaseEvent {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(feature = "async")]
impl Eq for BaseEvent {}

#[cfg(feature = "async")]
impl Hash for BaseEvent {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Drop for BaseEvent {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseEvent(self.0))
        }
    }
}

unsafe impl Send for BaseEvent {}
unsafe impl Sync for BaseEvent {}

#[cfg(feature = "async")]
#[no_mangle]
extern "C" fn notify (_event: cl_event, _status: cl_sys::cl_int, data: *mut cl_sys::c_void) {
    let data = unsafe { Arc::from_raw(data as *const AtomicWaker) };
    data.wake()
}