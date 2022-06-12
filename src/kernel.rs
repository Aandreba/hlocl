use core::{mem::MaybeUninit, ptr::addr_of};
use alloc::{string::{String}, vec::Vec, format};
use cl_sys::{cl_kernel, clRetainKernel, clReleaseKernel, clCreateKernel, clGetKernelInfo, cl_kernel_info, CL_KERNEL_FUNCTION_NAME, CL_KERNEL_NUM_ARGS, CL_KERNEL_REFERENCE_COUNT, CL_KERNEL_CONTEXT, CL_KERNEL_PROGRAM, clSetKernelArg, cl_kernel_arg_info, CL_KERNEL_ARG_ADDRESS_GLOBAL, CL_KERNEL_ARG_ADDRESS_LOCAL, CL_KERNEL_ARG_ADDRESS_CONSTANT, CL_KERNEL_ARG_ADDRESS_PRIVATE, CL_KERNEL_ARG_ADDRESS_QUALIFIER, CL_KERNEL_ARG_ACCESS_READ_ONLY, CL_KERNEL_ARG_ACCESS_WRITE_ONLY, CL_KERNEL_ARG_ACCESS_READ_WRITE, CL_KERNEL_ARG_ACCESS_NONE, CL_KERNEL_ARG_ACCESS_QUALIFIER, clGetKernelArgInfo, CL_KERNEL_ARG_NAME, CL_KERNEL_ARG_TYPE_NAME, CL_KERNEL_ARG_TYPE_CONST, CL_KERNEL_ARG_TYPE_RESTRICT, CL_KERNEL_ARG_TYPE_VOLATILE, CL_KERNEL_ARG_TYPE_QUALIFIER, clEnqueueNDRangeKernel, cl_mem};
use crate::{prelude::{ErrorCL, Program, Context, CommandQueue, BaseEvent}, error::Result, buffer::UnsafeBuffer};

#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Kernel (pub(crate) cl_kernel);

impl Kernel {
    #[inline]
    pub fn new (program: &Program, name: &str) -> Result<Self> {
        let mut name = name.as_bytes().to_vec();
        name.push(0);
        
        let mut err = 0;
        let id = unsafe {
            clCreateKernel(program.0, name.as_ptr().cast(), &mut err)
        };

        if err == 0 { return Ok(Self(id)); }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidProgram => report.attach_printable("program is not a valid program object"),
                    ErrorCL::InvalidProgramExecutable => report.attach_printable("there is no successfully built executable for program"),
                    ErrorCL::InvalidKernelName => report.attach_printable("kernel name not found in program"),
                    ErrorCL::InvalidKernelDefinition => report.attach_printable("the function definition for __kernel function given by kernel name does not exist"),
                    ErrorCL::InvalidValue => report.attach_printable("kernel name is null"),
                    ErrorCL::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(ErrorCL::from(err))
            }
        }
    }

    #[inline(always)]
    pub unsafe fn set_arg<T: Copy> (&mut self, idx: u32, v: T) -> Result<()> {
        let err = clSetKernelArg(self.0, idx, core::mem::size_of::<T>(), addr_of!(v).cast());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn set_mem_arg<T: Copy + Unpin> (&mut self, idx: u32, v: &UnsafeBuffer<T>) -> Result<()> {
        let err = clSetKernelArg(self.0, idx, core::mem::size_of::<cl_mem>(), addr_of!(v.0).cast());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn alloc_arg<T> (&mut self, idx: u32, len: usize) -> Result<()> {
        let arg_size = len.checked_mul(core::mem::size_of::<T>()).expect("Kernel argument size overflow");
        let err = clSetKernelArg(self.0, idx, arg_size, core::ptr::null_mut());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    /// Return the kernel function name.
    #[inline(always)]
    pub fn name (&self) -> Result<String> {
        self.get_info_string(CL_KERNEL_FUNCTION_NAME)
    }

    /// Return the number of arguments to _kernel_.
    #[inline(always)]
    pub fn num_args (&self) -> Result<u32> {
        self.get_info(CL_KERNEL_NUM_ARGS)
    }

    /// Return the _kernel_ reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_KERNEL_REFERENCE_COUNT)
    }

    /// Return the context associated with _kernel_.
    #[inline(always)]
    pub fn context (&self) -> Result<Context> {
        self.get_info(CL_KERNEL_CONTEXT)
    }

    /// Return the program object associated with _kernel_.
    #[inline(always)]
    pub fn program (&self) -> Result<Program> {
        self.get_info(CL_KERNEL_PROGRAM)
    }

    /// Returns the address qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_address_qualifier (&self, idx: u32) -> Result<AddrQualifier> {
        self.get_arg_info(CL_KERNEL_ARG_ADDRESS_QUALIFIER, idx)
    }

    /// Returns the access qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_access_qualifier (&self, idx: u32) -> Result<AccessQualifier> {
        self.get_arg_info(CL_KERNEL_ARG_ACCESS_QUALIFIER, idx)
    }

    /// Returns the type name specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_type_name (&self, idx: u32) -> Result<String> {
        self.get_arg_info_string(CL_KERNEL_ARG_TYPE_NAME, idx)
    }

    /// Returns the type qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_qualifier (&self, idx: u32) -> Result<String> {
        self.get_arg_info(CL_KERNEL_ARG_TYPE_QUALIFIER, idx)
    }

    /// Returns the name specified for the argument given by ```idx```. 
    #[inline(always)]
    pub fn arg_name (&self, idx: u32) -> Result<String> {
        self.get_arg_info_string(CL_KERNEL_ARG_NAME, idx)
    }

    pub fn enqueue<const N: usize> (&mut self, queue: &CommandQueue, global_dims: &[usize; N], local_dims: Option<&[usize; N]>, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<BaseEvent> {        
        let dim_len = u32::try_from(N).expect("Too many work dimensions");
        let local_dims = match local_dims {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let wait = wait.into_iter().map(|x| x.as_ref().0).collect::<Vec<_>>();
        let wait_len = u32::try_from(wait.len()).unwrap();
        let wait = match wait_len {
            0 => core::ptr::null(),
            _ => wait.as_ptr()
        };

        let mut event = core::ptr::null_mut();
        let err = unsafe {
            clEnqueueNDRangeKernel(queue.0, self.0, dim_len, core::ptr::null_mut(), global_dims.as_ptr(), local_dims, wait_len, wait, &mut event)
        };

        if err == 0 { return BaseEvent::new(event); }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidProgramExecutable => report.attach_printable("there is no successfully built program executable available for device associated with the command queue"),
                    ErrorCL::InvalidCommandQueue => report.attach_printable("command queue is not a valid command-queue."),
                    ErrorCL::InvalidKernel => report.attach_printable("kernel is not a valid kernel object"),
                    ErrorCL::InvalidContext => report.attach_printable("context associated with the command queue and kernel is not the same or the context associated with command queue and events in the event wait list are not the same"),
                    ErrorCL::InvalidKernelArgs => report.attach_printable("the kernel argument values have not been specified"),
                    ErrorCL::InvalidWorkDimension => report.attach_printable("work-dimension is not a valid value (i.e. a value between 1 and 3)"),
                    ErrorCL::InvalidWorkGroupSize => report.attach_printable("local work size is specified and is invalid (i.e. specified values in local work size exceed the maximum size of workgroup for the device associated with queue)"),
                    ErrorCL::InvalidGlobalOffset => report.attach_printable("global work offset is not NULL"),
                    ErrorCL::OutOfResources => report.attach_printable("there is a failure to queue the execution instance of kernel on the command-queue because of insufficient resources needed to execute the kernel"),
                    ErrorCL::MemObjectAllocationFailure => report.attach_printable("there is a failure to allocate memory for data store associated with image or buffer objects specified as arguments to kernel"),
                    ErrorCL::InvalidEventWaitList => report.attach_printable("event objects in event wait list are not valid events"),
                    ErrorCL::OutOfHostMemory => report.attach_printable("failure to allocate resources required by the OpenCL implementation on the host"),
                    _ => report
                };

                Err(report)
            } else {
                Err(ErrorCL::from(err))
            }
        }
    }

    #[inline]
    fn get_info_string (&self, ty: cl_kernel_info) -> Result<String> {
        unsafe {
            let mut len = 0;
            let err = clGetKernelInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len);
            self.parse_error(err, ty, 0)?;

            let mut result = Vec::<u8>::with_capacity(len);
            let err = clGetKernelInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, ty, len)?;

            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_kernel_info) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetKernelInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error(err, ty, core::mem::size_of::<T>())?;
            Ok(value.assume_init())
        }
    }

    #[inline]
    fn get_arg_info_string (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<String> {
        unsafe {
            let mut len = 0;
            let err = clGetKernelArgInfo(self.0, idx, ty, 0, core::ptr::null_mut(), &mut len);
            self.parse_error_arg(err, ty, 0)?;

            let mut result = Vec::<u8>::with_capacity(len);
            let err = clGetKernelArgInfo(self.0, idx, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error_arg(err, ty, len)?;
            
            result.set_len(len - 1);
            Ok(String::from_utf8(result).unwrap())
        }
    }

    #[inline]
    fn get_arg_info<T> (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<T> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetKernelArgInfo(self.0, idx, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            self.parse_error_arg(err, ty, core::mem::size_of::<T>())?;
            Ok(value.assume_init())
        }
    }

    fn parse_error (&self, err: i32, ty: cl_kernel_info, size: usize) -> Result<()> {
        if err == 0 {
            return Ok(());
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidKernel => report.attach_printable(format!("'{:?}' is not a valid kernel", self.0)),
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

    fn parse_error_arg (&self, err: i32, ty: cl_kernel_arg_info, size: usize) -> Result<()> {
        if err == 0 {
            return Ok(());
        }

        todo!();
        cfg_if::cfg_if! {
            if #[cfg(feature = "error-stack")] {
                let err = ErrorCL::from(err);
                let report = error_stack::Report::new(err);

                let report = match err {
                    ErrorCL::InvalidKernel => report.attach_printable(format!("'{:?}' is not a valid kernel", self.0)),
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

impl Clone for Kernel {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainKernel(self.0))
        }
        
        Self(self.0)
    }
}

impl Drop for Kernel {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseKernel(self.0))
        }
    }
}

unsafe impl Send for Kernel {}
unsafe impl Sync for Kernel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AddrQualifier {
    Global = CL_KERNEL_ARG_ADDRESS_GLOBAL,
    Local = CL_KERNEL_ARG_ADDRESS_LOCAL,
    Constant = CL_KERNEL_ARG_ADDRESS_CONSTANT,
    Private = CL_KERNEL_ARG_ADDRESS_PRIVATE
}

impl Default for AddrQualifier {
    #[inline(always)]
    fn default() -> Self {
        Self::Private
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AccessQualifier {
    ReadOnly = CL_KERNEL_ARG_ACCESS_READ_ONLY,
    WriteOnly = CL_KERNEL_ARG_ACCESS_WRITE_ONLY,
    ReadWrite = CL_KERNEL_ARG_ACCESS_READ_WRITE,
    None = CL_KERNEL_ARG_ACCESS_NONE
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct TypeQualifier: u64 {
        const CONST = CL_KERNEL_ARG_TYPE_CONST;
        const RESTRICT = CL_KERNEL_ARG_TYPE_RESTRICT;
        const VOLATILE = CL_KERNEL_ARG_TYPE_VOLATILE;
    }
}

impl Default for TypeQualifier {
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}