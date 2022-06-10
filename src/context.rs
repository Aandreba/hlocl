use core::mem::MaybeUninit;

use alloc::vec::{Vec};
use cl_sys::{cl_context, cl_context_properties, CL_CONTEXT_PLATFORM, CL_CONTEXT_INTEROP_USER_SYNC, clCreateContext, clReleaseContext, clRetainContext, cl_context_info, clGetContextInfo, CL_CONTEXT_REFERENCE_COUNT, CL_CONTEXT_NUM_DEVICES, CL_CONTEXT_DEVICES};
use crate::error::ErrorCL;
use crate::prelude::{Platform, Device};

#[cfg(feature = "def")]
lazy_static! {
    static ref CONTEXT: Context = crate::utils::ContextManager::default().context().clone();
}

/// OpenCL context
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Context (pub(crate) cl_context);

impl Context {
    pub fn new (props: Option<ContextProps>, devices: &[Device]) -> Result<Self, ErrorCL> {
        let props = match props {
            Some(x) => x.build().as_mut_ptr(),
            None => core::ptr::null_mut()
        };

        let len = u32::try_from(devices.len()).expect("too many devices");
        let mut err = 0;

        let id = unsafe {
            clCreateContext(props, len, devices.as_ptr().cast(), None, core::ptr::null_mut(), &mut err)
        };

        if err != 0 {
            return Err(ErrorCL::from(err));
        }

        Ok(Context(id))
    }

    /// Return the context reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_CONTEXT_REFERENCE_COUNT)
    }

    /// Return the number of devices in context. 
    #[inline(always)]
    pub fn device_count (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_CONTEXT_NUM_DEVICES)
    }

    /// Return the list of devices in context.
    #[inline]
    pub fn devices (&self) -> Result<Vec<Device>, ErrorCL> {
        let count = self.device_count()?;
        let mut result = Vec::<Device>::with_capacity(count as usize);

        let err = unsafe {
            clGetContextInfo(self.0, CL_CONTEXT_DEVICES, result.capacity() * core::mem::size_of::<Device>(), result.as_mut_ptr().cast(), core::ptr::null_mut())
        };

        if err != 0 {
            return Err(ErrorCL::from(err));
        }

        unsafe { result.set_len(result.capacity()); }
        Ok(result)
    }

    #[inline]
    pub fn properties (&self) -> Result<ContextProps, ErrorCL> {
        todo!()
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn default () -> &'static Context {
        &CONTEXT
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_context_info) -> Result<T, ErrorCL> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetContextInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(value.assume_init());
            }
            
            Err(ErrorCL::from(err))
        }
    }
}

impl Clone for Context {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainContext(self.0))
        }

        Self(self.0)
    }
}

impl Drop for Context {
    #[inline(always)]
    fn drop (&mut self) {
        unsafe {
            tri_panic!(clReleaseContext(self.0));
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

// TODO fix
/// OpenCL context properties
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextProps {
    pub platform: Option<Platform>,
    pub interop_user_sync: bool,
    // TODO rest of the properties
}

impl ContextProps {
    pub fn new () -> Self {
        Self { 
            platform: None,
            interop_user_sync: false
        }
    }

    #[inline]
    pub fn build (self) -> Vec<cl_context_properties> {
        let mut result = Vec::<cl_context_properties>::with_capacity(5);

        // interop_user_sync
        result.extend([
            CL_CONTEXT_INTEROP_USER_SYNC as cl_context_properties, 
            self.interop_user_sync as cl_context_properties
        ]);

        // platform
        if let Some(platform) = self.platform {
            result.extend([
                CL_CONTEXT_PLATFORM as cl_context_properties, 
                platform.0 as cl_context_properties
            ])
        }

        result.push(0);
        result
    }
}

impl Default for ContextProps {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for ContextProps {}
unsafe impl Sync for ContextProps {}