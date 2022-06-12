use core::mem::MaybeUninit;
use alloc::format;
use alloc::vec::{Vec};
use cl_sys::{cl_context, cl_context_properties, CL_CONTEXT_PLATFORM, CL_CONTEXT_INTEROP_USER_SYNC, clCreateContext, clReleaseContext, clRetainContext, cl_context_info, clGetContextInfo, CL_CONTEXT_REFERENCE_COUNT, CL_CONTEXT_NUM_DEVICES, CL_CONTEXT_DEVICES};
use crate::error::ErrorCL;
use crate::prelude::{Platform, Device, Result};

/// OpenCL context
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Context (pub(crate) cl_context);

impl Context {
    pub fn new (props: Option<ContextProps>, devices: &[Device]) -> Result<Self> {
        let props = match props {
            Some(x) => x.build().as_mut_ptr(),
            None => core::ptr::null_mut()
        };

        let len = u32::try_from(devices.len()).expect("too many devices");
        let mut err = 0;

        let id = unsafe {
            clCreateContext(props, len, devices.as_ptr().cast(), None, core::ptr::null_mut(), &mut err)
        };

        if err == 0 {
            return Ok(Context(id))
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidPlatform => report.attach_printable("props is NULL and no platform could be selected or platform value specified in props is not a valid platform"),
                    ErrorCL::InvalidValue => report.attach_printable("context property name in properties is not a supported property name, devices is NULL or num_devices is equal to zero"),
                    ErrorCL::InvalidDevice => report.attach_printable("devices contains an invalid device or they are not associated with the specified platform"),
                    ErrorCL::DeviceNotAvailable => report.attach_printable("a device in devices is currently not available"),
                    ErrorCL::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(ErrorCL::from(err))
            }
        }
    }

    /// Return the context reference count. The reference count returned should be considered immediately stale. It is unsuitable for general use in applications. This feature is provided for identifying memory leaks. 
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_CONTEXT_REFERENCE_COUNT)
    }

    /// Return the number of devices in context. 
    #[inline(always)]
    pub fn device_count (&self) -> Result<u32> {
        self.get_info(CL_CONTEXT_NUM_DEVICES)
    }

    /// Return the list of devices in context.
    #[inline]
    pub fn devices (&self) -> Result<Vec<Device>> {
        let count = self.device_count()?;
        let mut result = Vec::<Device>::with_capacity(count as usize);

        let size = result.capacity() * core::mem::size_of::<Device>();
        let err = unsafe {
            clGetContextInfo(self.0, CL_CONTEXT_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut())
        };
        Self::parse_error(&self, err, CL_CONTEXT_DEVICES, size)?;

        unsafe { result.set_len(result.capacity()); }
        Ok(result)
    }

    #[inline]
    pub fn properties (&self) -> Result<ContextProps> {
        todo!()
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn default () -> &'static Context {
        crate::utils::ContextManager::default()
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_context_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetContextInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, ty, core::mem::size_of::<T>())?;
            Ok(value.assume_init())
        }
    }

    fn parse_error (&self, err: i32, ty: cl_context_info, size: usize) -> Result<()> {
        if err == 0 {
            return Ok(());
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidContext => report.attach_printable(format!("'{:?}' is not a valid context", self.0)),
                    ErrorCL::InvalidValue => report.attach_printable(format!("'{ty}' is not one of the supported values or size in bytes specified by {size} is < size of return type as specified in the table above and '{ty}' is not a NULL value")),
                    ErrorCL::OutOfResources => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the device"),
                    ErrorCL::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(ErrorCL::from(err))
            }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextProps {
    pub platform: Option<Platform>,
    pub interop_user_sync: bool,
    // TODO rest of the properties
}

impl ContextProps {
    #[inline(always)]
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