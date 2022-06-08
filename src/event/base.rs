use core::{mem::MaybeUninit, ptr::addr_of, hash::Hash};
use cl_sys::{cl_event, cl_event_info, clReleaseEvent, clGetEventInfo, CL_EVENT_COMMAND_QUEUE, CL_EVENT_COMMAND_TYPE, CL_EVENT_COMMAND_EXECUTION_STATUS, clWaitForEvents, clRetainEvent};
use futures::Future;
use crate::prelude::ErrorCL;
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

impl BaseEvent {
    #[cfg(feature = "async")]
    pub fn new (id: cl_event) -> Result<Self, ErrorCL> {
        let waker = AtomicWaker::new();
        let mut data = Arc::new(waker);
        let ptr = Arc::into_raw(data);

        unsafe {
            Arc::increment_strong_count(ptr);
            data = Arc::from_raw(ptr);

            let err = clSetEventCallback(id, cl_sys::CL_COMPLETE, Some(notify), ptr as *mut _);
            if err != 0 {
                return Err(ErrorCL::from(err));
            }
        }

        Ok(Self(id, data))
    }

    #[cfg(not(feature = "async"))]
    #[inline(always)]
    pub fn new (id: cl_event) -> Result<Self, ErrorCL> {
        Ok(Self(id))
    }

    #[inline]
    fn get_info<T> (&self, id: cl_event_info) -> Result<T, ErrorCL> {
        let mut result = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetEventInfo(self.0, id, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(result.assume_init());
            }

            Err(ErrorCL::from(err))
        }
    }
}

impl Event for BaseEvent {
    type Result = ();
    
    #[inline(always)]
    fn command_queue (&self) -> Result<crate::prelude::CommandQueue, ErrorCL> {
        self.get_info(CL_EVENT_COMMAND_QUEUE)
    }

    #[inline(always)]
    fn ty (&self) -> Result<super::CommandType, ErrorCL> {
        self.get_info(CL_EVENT_COMMAND_TYPE)
    }

    #[inline(always)]
    fn status (&self) -> Result<super::EventStatus, ErrorCL> {
        self.get_info(CL_EVENT_COMMAND_EXECUTION_STATUS)
    }

    #[inline(always)]
    fn wait (self) -> Result<(), ErrorCL> {
        let err = unsafe {
            clWaitForEvents(1, addr_of!(self.0))
        };

        if err == 0 {
            return Ok(())
        }

        Err(ErrorCL::from(err))
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
impl Future for BaseEvent {
    type Output = Result<(), ErrorCL>;

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
extern "C" fn notify (_event: cl_event, _status: cl_sys::cl_int, data: *mut cl_sys::c_void) {
    let data = unsafe { Arc::from_raw(data as *const AtomicWaker) };
    data.wake()
}