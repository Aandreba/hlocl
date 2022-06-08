use core::borrow::Borrow;
use cl_sys::{CL_COMMAND_NDRANGE_KERNEL, CL_COMMAND_TASK, CL_COMMAND_NATIVE_KERNEL, CL_COMMAND_READ_BUFFER, CL_COMMAND_WRITE_BUFFER, CL_COMMAND_COPY_BUFFER, CL_COMMAND_READ_IMAGE, CL_COMMAND_WRITE_IMAGE, CL_COMMAND_COPY_IMAGE, CL_COMMAND_COPY_IMAGE_TO_BUFFER, CL_COMMAND_COPY_BUFFER_TO_IMAGE, CL_COMMAND_MAP_BUFFER, CL_COMMAND_MAP_IMAGE, CL_COMMAND_UNMAP_MEM_OBJECT, CL_COMMAND_MARKER, CL_COMMAND_ACQUIRE_GL_OBJECTS, CL_COMMAND_RELEASE_GL_OBJECTS, CL_COMPLETE, CL_RUNNING, CL_SUBMITTED, CL_QUEUED};
use crate::prelude::{CommandQueue, ErrorCL};
use self::various::Then;

flat_mod!(base, user, buffer);
pub mod various;

#[cfg(feature = "async")]
pub trait Event: Sized + Borrow<BaseEvent> + futures::Future<Output = Result<Self::Result, ErrorCL>> {
    type Result;

    fn wait (self) -> Result<Self::Result, ErrorCL>;

    #[inline(always)]
    fn command_queue (&self) -> Result<CommandQueue, ErrorCL> {
        BaseEvent::command_queue(self.borrow())
    }

    #[inline(always)]
    fn ty (&self) -> Result<CommandType, ErrorCL> {
        BaseEvent::ty(self.borrow())
    }

    #[inline(always)]
    fn status (&self) -> Result<EventStatus, ErrorCL> {
        BaseEvent::status(self.borrow())
    }

    #[inline(always)]
    fn then<O, F: Unpin + FnOnce(Self::Result) -> O> (self, f: F) -> Then<O, Self, F> {
        Then::new(self, f)
    }

    #[inline(always)]
    fn borrow_base (&self) -> &BaseEvent {
        self.borrow().borrow()
    }
}

#[cfg(not(feature = "async"))]
pub trait Event: Sized + Borrow<BaseEvent> {
    type Result;

    fn wait (self) -> Result<Self::Result, ErrorCL>;

    #[inline(always)]
    fn command_queue (&self) -> Result<CommandQueue, ErrorCL> {
        BaseEvent::command_queue(self.borrow())
    }

    #[inline(always)]
    fn ty (&self) -> Result<CommandType, ErrorCL> {
        BaseEvent::ty(self.borrow())
    }

    #[inline(always)]
    fn status (&self) -> Result<EventStatus, ErrorCL> {
        BaseEvent::status(self.borrow())
    }

    #[inline(always)]
    fn then<O, F: FnOnce(Self::Result) -> O> (self, f: F) -> Then<O, Self, F> {
        Then {
            inner: self,
            f
        }
    }

    #[inline(always)]
    fn borrow_base (&self) -> &BaseEvent {
        self.borrow().borrow()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum CommandType {
    NdRangeKernel = CL_COMMAND_NDRANGE_KERNEL,
    Task = CL_COMMAND_TASK,
    NativeKernel = CL_COMMAND_NATIVE_KERNEL,
    ReadBuffer = CL_COMMAND_READ_BUFFER,
    WriteBuffer = CL_COMMAND_WRITE_BUFFER,
    CopyBuffer = CL_COMMAND_COPY_BUFFER,
    ReadImage = CL_COMMAND_READ_IMAGE,
    WriteImage = CL_COMMAND_WRITE_IMAGE,
    CopyImage = CL_COMMAND_COPY_IMAGE,
    CopyImageToBuffer = CL_COMMAND_COPY_IMAGE_TO_BUFFER,
    CopyBufferToImage = CL_COMMAND_COPY_BUFFER_TO_IMAGE,
    MapBuffer = CL_COMMAND_MAP_BUFFER,
    MapImage = CL_COMMAND_MAP_IMAGE,
    UnmapMemObject = CL_COMMAND_UNMAP_MEM_OBJECT,
    Marker = CL_COMMAND_MARKER,
    AcquireGLObjects = CL_COMMAND_ACQUIRE_GL_OBJECTS,
    ReleaseGLObjects = CL_COMMAND_RELEASE_GL_OBJECTS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum EventStatus {
    Complete = CL_COMPLETE,
    Running = CL_RUNNING,
    Submitted = CL_SUBMITTED,
    Queued = CL_QUEUED
}