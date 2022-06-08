use core::{fmt::Debug, ptr::addr_of_mut};
use alloc::{vec::Vec, string::{String, FromUtf8Error}};
use cl_sys::{clGetPlatformIDs, cl_platform_id, clGetPlatformInfo, cl_platform_info, c_uchar, CL_PLATFORM_PROFILE, CL_PLATFORM_VERSION, CL_PLATFORM_NAME, CL_PLATFORM_VENDOR, CL_PLATFORM_EXTENSIONS, cl_ulong, CL_PLATFORM_HOST_TIMER_RESOLUTION, cl_uchar};

lazy_static::lazy_static! {
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
pub struct Platform (cl_platform_id);

impl Platform {
    #[inline(always)]
    pub fn all () -> &'static [Platform] {
        &PLATFORMS
    }

    #[inline(always)]
    pub fn id (&self) -> cl_platform_id {
        self.0
    }

    /// OpenCL profile string.
    #[inline(always)]
    pub fn profile (&self) -> String {
        self.get_info_string(CL_PLATFORM_PROFILE).unwrap()
    }

    /// OpenCL version string.
    #[inline(always)]
    pub fn version (&self) -> String {
        self.get_info_string(CL_PLATFORM_VERSION).unwrap()
    }

    /// Platform name string.
    #[inline(always)]
    pub fn name (&self) -> String {
        self.get_info_string(CL_PLATFORM_NAME).unwrap()
    }

    /// Platform vendor string.
    #[inline(always)]
    pub fn vendor (&self) -> String {
        self.get_info_string(CL_PLATFORM_VENDOR).unwrap()
    }

    /// Returns a list of extension names (the extension names themselves do not contain any spaces) supported by the platform. Extensions defined here must be supported by all devices associated with this platform.
    #[inline(always)]
    pub fn extensions (&self) -> Vec<String> {
        self.get_info_string(CL_PLATFORM_EXTENSIONS).unwrap()
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
    }

    #[inline(always)]
    pub fn host_timer_resolution (&self) -> cl_ulong {
        self.get_info_ulong(CL_PLATFORM_HOST_TIMER_RESOLUTION)
    }

    #[inline(always)]
    pub(super) fn get_by_id (id: cl_platform_id) -> Option<Platform> {
        PLATFORMS.iter().copied().find(|p| p.0 == id)
    }

    #[inline]
    fn get_info_string (&self, ty: cl_platform_info) -> Result<String, FromUtf8Error> {
        unsafe {
            let mut len = 0;
            tri_panic!(clGetPlatformInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<c_uchar>::with_capacity(len);
            tri_panic!(clGetPlatformInfo(self.0, ty, len * core::mem::size_of::<cl_uchar>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            String::from_utf8(result)
        }
    }

    #[inline(always)]
    fn get_info_ulong (&self, ty: cl_platform_info) -> cl_ulong {
        let mut result : cl_ulong = 0;
        unsafe {
            tri_panic!(clGetPlatformInfo(self.0, ty, core::mem::size_of::<cl_ulong>(), addr_of_mut!(result).cast(), core::ptr::null_mut()));
        }

        result
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