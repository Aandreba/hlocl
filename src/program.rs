use core::{mem::MaybeUninit, num::NonZeroUsize};
use alloc::{string::{String}, vec::Vec};
use opencl_sys::{cl_program, clReleaseProgram, clCreateProgramWithSource, clRetainProgram, clBuildProgram, cl_program_info, clGetProgramInfo, CL_PROGRAM_REFERENCE_COUNT, CL_PROGRAM_CONTEXT, CL_PROGRAM_NUM_DEVICES, CL_PROGRAM_DEVICES, CL_PROGRAM_SOURCE, clRetainContext};
use crate::{prelude::{Result, Error, Context, Device}};

#[cfg(feature = "error-stack")]
use {alloc::format, opencl_sys::{clGetProgramBuildInfo, CL_PROGRAM_BUILD_LOG}};

/// OpenCL program
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Program (pub(crate) cl_program);

impl Program {
    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn from_source (source: &str) -> Result<Self> {
        Self::from_source_with_context(Context::default(), source)
    }

    #[inline(always)]
    pub fn from_source_with_context (ctx: &Context, source: &str) -> Result<Self> {
        let len = [source.len()].as_ptr();
        let strings = [source.as_ptr().cast()].as_ptr();

        let mut err = 0;
        let id = unsafe {
            clCreateProgramWithSource(ctx.0, 1, strings, len, &mut err)
        };

        if err == 0 {
            let this = Self(id);
            this.build(ctx)?;
            return Ok(this)
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidContext => report.attach_printable(format!("'{:?}' is not a valid context", ctx.0)),
                    Error::InvalidValue => report.attach_printable(format!("source count is zero or any entry in strings is NULL")),
                    _ => report
                };

                Err(report)
            } else {
                Err(Error::from(err))
            }
        }
    }

    /// Return the program reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_REFERENCE_COUNT)
    }

    /// Return the context specified when the program object is created
    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        let ctx : Context = self.get_info(CL_PROGRAM_CONTEXT)?;
        unsafe { tri_panic!(clRetainContext(ctx.0)); }
        Ok(ctx)
    }

    /// Return the number of devices associated with program.
    #[inline(always)]
    pub fn device_count (&self) -> Result<u32> {
        self.get_info(CL_PROGRAM_NUM_DEVICES)
    }

    /// Return the list of devices associated with the program object. This can be the devices associated with context on which the program object has been created or can be a subset of devices that are specified when a progam object is created using clCreateProgramWithBinary.
    #[inline]
    pub fn devices (&self) -> Result<Vec<Device>> {
        let count = self.device_count()?;
        let mut result = Vec::<Device>::with_capacity(count as usize);
        let size = result.capacity().checked_mul(core::mem::size_of::<Device>()).expect("Too many devices");

        let err = unsafe {
            clGetProgramInfo(self.0, CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut())
        };
        
        self.parse_error(err, CL_PROGRAM_DEVICES, size)?;
        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }

    /// Return the program source code
    #[inline(always)]
    pub fn source (&self) -> Result<String> {
        self.get_info_string(CL_PROGRAM_SOURCE)
    }

    /// Returns an array that contains the size in bytes of the program binary for each device associated with program. The size of the array is the number of devices associated with program. If a binary is not available for a device(s), a size of zero is returned.
    #[inline]
    pub fn binary_sizes (&self) -> Result<Vec<Option<NonZeroUsize>>> {
        let count = self.device_count()?;
        let mut result = Vec::<Option<NonZeroUsize>>::with_capacity(count as usize);
        let size = result.capacity().checked_mul(core::mem::size_of::<usize>()).expect("Too many binaries");

        let err = unsafe {
            clGetProgramInfo(self.0, CL_PROGRAM_DEVICES, size, result.as_mut_ptr().cast(), core::ptr::null_mut())
        };

        self.parse_error(err, CL_PROGRAM_DEVICES, size)?;
        unsafe { result.set_len(result.capacity()) }
        Ok(result)
    }

    #[inline]
    pub fn binaries (&self) -> Result<Vec<Option<Vec<u8>>>> {
        todo!()
    }

    #[allow(unused_variables)]
    #[inline(always)]
    fn build (&self, cx: &Context) -> Result<()> {
        let build_result = unsafe {
            clBuildProgram(self.0, 0, core::ptr::null(), core::ptr::null(), None, core::ptr::null_mut())
        };

        if build_result == 0 {
            return Ok(());
        }

        #[cfg(feature = "error-stack")]
        let devices = cx.devices()?;
        #[cfg(feature = "error-stack")]
        let mut build_result = error_stack::Report::new(Error::from(build_result));
        #[cfg(not(feature = "error-stack"))]
        let build_result = Error::from(build_result);

        #[cfg(feature = "error-stack")]        
        for device in devices {
            let mut len = 0;
            let err = unsafe {
                clGetProgramBuildInfo(self.0, device.0, CL_PROGRAM_BUILD_LOG, 0, core::ptr::null_mut(), &mut len)
            };

            self.parse_error(err, CL_PROGRAM_BUILD_LOG, 0)?;
            if len == 0 { continue }

            let mut result = Vec::<u8>::with_capacity(len);
            let err = unsafe {
                clGetProgramBuildInfo(self.0, device.0, CL_PROGRAM_BUILD_LOG, len, result.as_mut_ptr().cast(), core::ptr::null_mut())
            };

            self.parse_error(err, CL_PROGRAM_BUILD_LOG, len)?;
            unsafe { result.set_len(len) }

            if let Ok(result) = String::from_utf8(result) {
                build_result = build_result.attach_printable(result);
            }
        } 

        Err(build_result)
    }

    #[inline]
    fn get_info_string (&self, ty: cl_program_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            let err = clGetProgramInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len);
            self.parse_error(err, ty, 0)?;

            let mut result = Vec::<u8>::with_capacity(len);
            let err = clGetProgramInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, ty, len)?;
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_program_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetProgramInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, ty, core::mem::size_of::<T>())?;
            Ok(value.assume_init())
        }
    }

    #[allow(unused_variables)]
    fn parse_error (&self, err: i32, ty: cl_program_info, size: usize) -> Result<()> {
        if err == 0 {
            return Ok(());
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = Error::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    Error::InvalidProgram => report.attach_printable(format!("'{:?}' is not a valid program", self.0)),
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

impl Clone for Program {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainProgram(self.0))
        }

        Self(self.0)
    }
}

impl Drop for Program {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseProgram(self.0));
        }
    }
}

unsafe impl Send for Program {}
unsafe impl Sync for Program {}