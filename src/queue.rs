use core::mem::MaybeUninit;
use opencl_sys::{cl_command_queue_properties, CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE, CL_QUEUE_PROFILING_ENABLE, cl_command_queue, clRetainCommandQueue, clReleaseCommandQueue, cl_command_queue_info, clGetCommandQueueInfo, CL_QUEUE_CONTEXT, CL_QUEUE_DEVICE, CL_QUEUE_PROPERTIES};
use crate::{prelude::{Context, Error, Device}, utils::ContextManager};

/// OpenCL command queue
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CommandQueue (pub(crate) cl_command_queue);

impl CommandQueue {
    #[inline]
    pub fn new (ctx: &Context, device: &Device, props: Option<CommandQueueProps>) -> Result<Self, Error> {
        let props = match props {
            Some(x) => x,
            None => CommandQueueProps::default()
        };

        let mut err = 0;

        #[cfg(feature = "cl2")]
        let id = unsafe {
            opencl_sys::clCreateCommandQueueWithProperties(ctx.0, device.0, &props.bits(), &mut err)
        };

        #[cfg(not(feature = "cl2"))]
        let id = unsafe {
            opencl_sys::clCreateCommandQueue(ctx.0, device.0, props.bits(), &mut err)
        };

        if err == 0 {
            return Ok(Self(id));
        }

        Err(Error::from(err))
    }

    /// Return the context specified when the command-queue is created.
    #[inline(always)]
    pub fn context (&self) -> Result<Context, Error> {
        self.get_info(CL_QUEUE_CONTEXT)
    }

    /// Return the device specified when the command-queue is created.
    #[inline(always)]
    pub fn device (&self) -> Result<Device, Error> {
        self.get_info(CL_QUEUE_DEVICE)
    }

    /// Return the currently specified properties for the command-queue.
    #[inline(always)]
    pub fn properties (&self) -> Result<CommandQueueProps, Error> {
        self.get_info(CL_QUEUE_PROPERTIES)
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn default () -> &'static CommandQueue {
        ContextManager::default().queue()
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_command_queue_info) -> Result<T, Error> {
        let mut result = MaybeUninit::<T>::uninit();
        unsafe {
            let err = clGetCommandQueueInfo(self.0, ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(result.assume_init());
            }

            Err(Error::from(err))
        }
    }
}

impl Clone for CommandQueue {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainCommandQueue(self.0));
        }

        Self(self.0)
    }
}

impl Drop for CommandQueue {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseCommandQueue(self.0))
        }
    }
}

unsafe impl Send for CommandQueue {}
unsafe impl Sync for CommandQueue {}

bitflags::bitflags! {
    /// Describes the command-queue properties supported by the device.
    #[derive(Default)]
    #[repr(transparent)]
    pub struct CommandQueueProps: cl_command_queue_properties {
        const OUT_OF_ORDER_EXEC_MODE_ENABLE = CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE;
        const PROFILING_ENABLE = CL_QUEUE_PROFILING_ENABLE;
    }
}