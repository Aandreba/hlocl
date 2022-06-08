use alloc::vec::{Vec};
use cl_sys::{cl_context, cl_context_properties, CL_CONTEXT_PLATFORM, CL_CONTEXT_INTEROP_USER_SYNC, clCreateContext, cl_uint, clReleaseContext, clRetainContext};
use crate::error::ErrorCL;
use crate::prelude::{Platform, Device};

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

        let len = cl_uint::try_from(devices.len()).expect("too many devices");
        #[cfg(feature = "std")]
        let pfn_notify = Some(notify);
        #[cfg(not(feature = "std"))]
        let pfn_notify = None;
        let mut err = 0;

        let id = unsafe {
            clCreateContext(props, len, devices.as_ptr().cast(), pfn_notify, core::ptr::null_mut(), &mut err)
        };

        if err != 0 {
            return Err(ErrorCL::from(err));
        }

        Ok(Context(id))
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

#[cfg(feature = "std")]
extern "C" fn notify (program: cl_sys::cl_program, cb: *const cl_sys::c_void, private_info: cl_sys::size_t, user_data: *mut cl_sys::c_void) {
    let errinfo = std::ffi::CStr::from_ptr(errinfo);
    std::io::stderr().write_all(errinfo.to_bytes()).unwrap()
}