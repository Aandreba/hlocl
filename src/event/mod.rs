use alloc::vec::Vec;
use opencl_sys::{CL_COMMAND_NDRANGE_KERNEL, CL_COMMAND_TASK, CL_COMMAND_NATIVE_KERNEL, CL_COMMAND_READ_BUFFER, CL_COMMAND_WRITE_BUFFER, CL_COMMAND_COPY_BUFFER, CL_COMMAND_READ_IMAGE, CL_COMMAND_WRITE_IMAGE, CL_COMMAND_COPY_IMAGE, CL_COMMAND_COPY_IMAGE_TO_BUFFER, CL_COMMAND_COPY_BUFFER_TO_IMAGE, CL_COMMAND_MAP_BUFFER, CL_COMMAND_MAP_IMAGE, CL_COMMAND_UNMAP_MEM_OBJECT, CL_COMMAND_MARKER, CL_COMMAND_ACQUIRE_GL_OBJECTS, CL_COMMAND_RELEASE_GL_OBJECTS, CL_COMPLETE, CL_RUNNING, CL_SUBMITTED, CL_QUEUED};
use crate::{prelude::{Result, CommandQueue}};
use self::various::{Map, Swap, Then};

flat_mod!(base, user, buffer);
#[cfg(feature = "async")]
flat_mod!(future);
pub mod various;

#[cfg(feature = "async")]
pub trait Event: Sized + Unpin + AsRef<BaseEvent> + futures::Future<Output = crate::prelude::Result<Self::Result>> {
    type Result;

    fn wait (self) -> Result<Self::Result>;
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<Vec<Self::Result>>;

    /// # Panic
    /// This method panics if ```wait_all``` doesn't return a vector of the same size as the input.
    #[inline(always)]
    fn wait_all_array<const N: usize> (iter: [Self; N]) -> Result<[Self::Result; N]> {
        let all = Self::wait_all(iter)?;
        let all = match TryInto::<[Self::Result; N]>::try_into(all) {
            Ok(x) => x,
            Err(e) => panic!("Returned vector is not the same size as the input: expected {N}, got {}", e.len())
        };

        Ok(all)
    }

    #[inline(always)]
    fn command_queue (&self) -> Result<CommandQueue> {
        BaseEvent::command_queue(self.as_ref())
    }

    #[inline(always)]
    fn ty (&self) -> Result<CommandType> {
        BaseEvent::ty(self.as_ref())
    }

    #[inline(always)]
    fn status (&self) -> Result<EventStatus> {
        BaseEvent::status(self.as_ref())
    }

    #[inline(always)]
    fn map<O, F: Unpin + FnOnce(Self::Result) -> O> (self, f: F) -> Map<O, Self, F> {
        Map::new(self, f)
    }

    #[inline(always)]
    fn then<F: Unpin + FnOnce(&mut Self::Result)> (self, f: F) -> Then<Self, F> {
        Then::new(self, f)
    }

    #[inline(always)]
    fn swap<O: Unpin> (self, v: O) -> Swap<O, Self> {
        Swap::new(self, v)
    }

    #[inline(always)]
    fn borrow_base (&self) -> &BaseEvent {
        <Self as AsRef<BaseEvent>>::as_ref(self)
    }
}

#[cfg(not(feature = "async"))]
pub trait Event: Sized + AsRef<BaseEvent> {
    type Result;

    fn wait (self) -> Result<Self::Result>;
    fn wait_all (iter: impl IntoIterator<Item = Self>) -> Result<Vec<Self::Result>>;

    /// # Panic
    /// This method panics if ```wait_all``` doesn't return a vector of the same size as the input.
    #[inline(always)]
    fn wait_all_array<const N: usize> (iter: [Self; N]) -> Result<[Self::Result; N]> {
        let all = Self::wait_all(iter)?;
        let all = match TryInto::<[Self::Result; N]>::try_into(all) {
            Ok(x) => x,
            Err(e) => panic!("Returned vector is not the same size as the input: expected {N}, got {}", e.len())
        };

        Ok(all)
    }

    #[inline(always)]
    fn command_queue (&self) -> Result<CommandQueue> {
        BaseEvent::command_queue(self.borrow_base())
    }

    #[inline(always)]
    fn ty (&self) -> Result<CommandType> {
        BaseEvent::ty(self.borrow_base())
    }

    #[inline(always)]
    fn status (&self) -> Result<EventStatus> {
        BaseEvent::status(self.borrow_base())
    }

    #[inline(always)]
    fn map<O, F: Unpin + FnOnce(Self::Result) -> O> (self, f: F) -> Map<O, Self, F> {
        Map::new(self, f)
    }

    #[inline(always)]
    fn then<F: Unpin + FnOnce(&mut Self::Result)> (self, f: F) -> Then<Self, F> {
        Then::new(self, f)
    }

    #[inline(always)]
    fn swap<O: Unpin> (self, v: O) -> Swap<O, Self> {
        Swap::new(self, v)
    }

    #[inline(always)]
    fn borrow_base (&self) -> &BaseEvent {
        self.as_ref()
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