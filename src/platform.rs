use core::{fmt::Debug, mem::MaybeUninit};
use alloc::{vec::Vec, string::{String}};
use cl_sys::{cl_platform_id, clGetPlatformInfo, cl_platform_info, c_uchar, CL_PLATFORM_PROFILE, CL_PLATFORM_VERSION, CL_PLATFORM_NAME, CL_PLATFORM_VENDOR, CL_PLATFORM_EXTENSIONS, CL_PLATFORM_HOST_TIMER_RESOLUTION, cl_uchar, clGetPlatformIDs};
use crate::{prelude::{Result, ErrorCL}};

lazy_static! {
    static ref PLATFORMS : Vec<Platform> = unsafe {
        let mut cnt = 0;
        tri_panic!(clGetPlatformIDs(0, core::ptr::null_mut(), &mut cnt));
        let cnt_size = usize::try_from(cnt).unwrap(); 

        let mut result = Vec::<Platform>::with_capacity(cnt_size);
        tri_panic!(clGetPlatformIDs(cnt, result.as_mut_ptr().cast(), core::ptr::null_mut()));
        result.set_len(cnt_size);

        result
    };
}

/// OpenCL platform
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Platform (pub(crate) cl_platform_id);

impl Platform {
    /// OpenCL profile string.
    #[inline(always)]
    pub fn profile (&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_PROFILE)
    }

    /// OpenCL version string.
    #[inline(always)]
    pub fn version (&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_VERSION)
    }

    /// Platform name string.
    #[inline(always)]
    pub fn name (&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_NAME)
    }

    /// Platform vendor string.
    #[inline(always)]
    pub fn vendor (&self) -> Result<String> {
        self.get_info_string(CL_PLATFORM_VENDOR)
    }

    /// Returns a list of extension names (the extension names themselves do not contain any spaces) supported by the platform. Extensions defined here must be supported by all devices associated with this platform.
    #[inline(always)]
    pub fn extensions (&self) -> Result<Vec<String>> {
        Ok(self.get_info_string(CL_PLATFORM_EXTENSIONS)?
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>())
    }

    #[inline(always)]
    pub fn host_timer_resolution (&self) -> Result<u64> {
        self.get_info_bits(CL_PLATFORM_HOST_TIMER_RESOLUTION)
    }

    #[inline(always)]
    pub fn all () -> &'static [Platform] {
        &PLATFORMS
    }

    #[inline]
    fn get_info_string (&self, ty: cl_platform_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            let err = clGetPlatformInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len);
            Self::parse_error(err)?;

            let mut result = Vec::<c_uchar>::with_capacity(len);
            let err = clGetPlatformInfo(self.0, ty, len * core::mem::size_of::<cl_uchar>(), result.as_mut_ptr().cast(), core::ptr::null_mut());
            Self::parse_error(err)?;
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info_bits<T> (&self, ty: cl_platform_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetPlatformInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            Self::parse_error(err)?;
            Ok(value.assume_init())
        }
    }

    fn parse_error (err: i32) -> Result<()> {
        if err == 0 {
            return Ok(())
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidPlatform => report.attach_printable("platform is not a valid platform"),
                    ErrorCL::InvalidValue => report.attach_printable("param_name is not one of the supported values or size in bytes specified by param_value_size is less than size of return type and param_value is not a NULL value"),
                    _ => report
                };

                Err(report)
            } else {
                Err(ErrorCL::from(err))
            }
        }
    }
}

impl Debug for Platform {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Platform")
        .field("id", &self.0)
        .field("profile", &self.profile())
        .field("version", &self.version())
        .field("name", &self.name())
        .field("vendor", &self.vendor())
        .field("extensions", &self.extensions())
        .field("host_timer_resolution", &self.host_timer_resolution())
        .finish()
    }
}

unsafe impl Send for Platform {}
unsafe impl Sync for Platform {}