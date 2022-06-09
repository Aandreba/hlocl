use core::{mem::MaybeUninit, ptr::addr_of};
use alloc::{string::{String, ToString}, vec::Vec};
use cl_sys::{cl_kernel, clRetainKernel, clReleaseKernel, clCreateKernel, clGetKernelInfo, cl_kernel_info, CL_KERNEL_FUNCTION_NAME, CL_KERNEL_NUM_ARGS, CL_KERNEL_REFERENCE_COUNT, CL_KERNEL_CONTEXT, CL_KERNEL_PROGRAM, clSetKernelArg, cl_kernel_arg_info, CL_KERNEL_ARG_ADDRESS_GLOBAL, CL_KERNEL_ARG_ADDRESS_LOCAL, CL_KERNEL_ARG_ADDRESS_CONSTANT, CL_KERNEL_ARG_ADDRESS_PRIVATE, CL_KERNEL_ARG_ADDRESS_QUALIFIER, CL_KERNEL_ARG_ACCESS_READ_ONLY, CL_KERNEL_ARG_ACCESS_WRITE_ONLY, CL_KERNEL_ARG_ACCESS_READ_WRITE, CL_KERNEL_ARG_ACCESS_NONE, CL_KERNEL_ARG_ACCESS_QUALIFIER, clGetKernelArgInfo, CL_KERNEL_ARG_NAME, CL_KERNEL_ARG_TYPE_NAME, CL_KERNEL_ARG_TYPE_CONST, CL_KERNEL_ARG_TYPE_RESTRICT, CL_KERNEL_ARG_TYPE_VOLATILE, CL_KERNEL_ARG_TYPE_QUALIFIER, clEnqueueNDRangeKernel, cl_mem};
use crate::{prelude::{ErrorCL, Program, Context, CommandQueue, BaseEvent}, error::ErrorType, buffer::UnsafeBuffer};

#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Kernel (pub(crate) cl_kernel);

impl Kernel {
    #[inline]
    pub fn new (program: &Program, name: &str) -> Result<Self, ErrorCL> {
        let mut name = name.as_bytes().to_vec();
        name.push(0);
        
        let mut err = 0;
        let id = unsafe {
            clCreateKernel(program.0, name.as_ptr().cast(), &mut err)
        };

        if err == 0 {
            return Ok(Self(id));
        }

        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn set_arg<T: Copy> (&mut self, idx: u32, v: T) -> Result<(), ErrorCL> {
        let err = clSetKernelArg(self.0, idx, core::mem::size_of::<T>(), addr_of!(v).cast());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn set_mem_arg<T: Copy + Unpin> (&mut self, idx: u32, v: &UnsafeBuffer<T>) -> Result<(), ErrorCL> {
        let err = clSetKernelArg(self.0, idx, core::mem::size_of::<cl_mem>(), addr_of!(v.0).cast());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    #[inline(always)]
    pub unsafe fn alloc_arg (&mut self, idx: u32, size: usize) -> Result<(), ErrorCL> {
        let err = clSetKernelArg(self.0, idx, size, core::ptr::null_mut());
        if err == 0 { return Ok(()); }
        Err(ErrorCL::from(err))
    }

    /// Return the kernel function name.
    #[inline(always)]
    pub fn name (&self) -> Result<String, ErrorCL> {
        self.get_info_string(CL_KERNEL_FUNCTION_NAME)
    }

    /// Return the number of arguments to _kernel_.
    #[inline(always)]
    pub fn num_args (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_KERNEL_NUM_ARGS)
    }

    /// Return the _kernel_ reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32, ErrorCL> {
        self.get_info(CL_KERNEL_REFERENCE_COUNT)
    }

    /// Return the context associated with _kernel_.
    #[inline(always)]
    pub fn context (&self) -> Result<Context, ErrorCL> {
        self.get_info(CL_KERNEL_CONTEXT)
    }

    /// Return the program object associated with _kernel_.
    #[inline(always)]
    pub fn program (&self) -> Result<Program, ErrorCL> {
        self.get_info(CL_KERNEL_PROGRAM)
    }

    /// Returns the address qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_address_qualifier (&self, idx: u32) -> Result<AddrQualifier, ErrorCL> {
        self.get_arg_info(CL_KERNEL_ARG_ADDRESS_QUALIFIER, idx)
    }

    /// Returns the access qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_access_qualifier (&self, idx: u32) -> Result<AccessQualifier, ErrorCL> {
        self.get_arg_info(CL_KERNEL_ARG_ACCESS_QUALIFIER, idx)
    }

    /// Returns the type name specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_type_name (&self, idx: u32) -> Result<String, ErrorCL> {
        self.get_arg_info_string(CL_KERNEL_ARG_TYPE_NAME, idx)
    }

    /// Returns the type qualifier specified for the argument given by ```idx```.
    #[inline(always)]
    pub fn arg_qualifier (&self, idx: u32) -> Result<String, ErrorCL> {
        self.get_arg_info(CL_KERNEL_ARG_TYPE_QUALIFIER, idx)
    }

    /// Returns the name specified for the argument given by ```idx```. 
    #[inline(always)]
    pub fn arg_name (&self, idx: u32) -> Result<String, ErrorCL> {
        self.get_arg_info_string(CL_KERNEL_ARG_NAME, idx)
    }

    pub fn enqueue<'a, const N: usize> (&mut self, queue: &CommandQueue, global_dims: &[usize; N], local_dims: Option<&[usize; N]>, wait: impl IntoIterator<Item = &'a BaseEvent>) -> Result<BaseEvent, ErrorCL> {        
        let dim_len = u32::try_from(N).expect("Too many work dimensions");
        let local_dims = match local_dims {
            Some(x) => x.as_ptr(),
            None => core::ptr::null()
        };

        let wait = wait.into_iter().map(|x| x.0).collect::<Vec<_>>();
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
        Err(ErrorCL::from(err))
    }

    #[inline]
    fn get_info_string (&self, ty: cl_kernel_info) -> Result<String, ErrorCL> {
        unsafe {
            let mut len = 0;
            tri_panic!(clGetKernelInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<u8>::with_capacity(len);
            tri_panic!(clGetKernelInfo(self.0, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            String::from_utf8(result).map_err(|e| ErrorCL::new(ErrorType::InvalidValue, Some(e.to_string())))
        }
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_kernel_info) -> Result<T, ErrorCL> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetKernelInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(value.assume_init());
            }
            
            Err(ErrorCL::from(err))
        }
    }

    #[inline]
    fn get_arg_info_string (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<String, ErrorCL> {
        unsafe {
            let mut len = 0;
            let err = clGetKernelArgInfo(self.0, idx, ty, 0, core::ptr::null_mut(), &mut len);

            if err != 0 {
                return Err(ErrorCL::from(err));
            }

            let mut result = Vec::<u8>::with_capacity(len);
            let err = clGetKernelArgInfo(self.0, idx, ty, len, result.as_mut_ptr().cast(), core::ptr::null_mut());

            if err != 0 {
                return Err(ErrorCL::from(err));
            }
            
            result.set_len(len - 1);
            String::from_utf8(result).map_err(|e| ErrorCL::new(ErrorType::InvalidValue, Some(e.to_string())))
        }
    }

    #[inline]
    fn get_arg_info<T> (&self, ty: cl_kernel_arg_info, idx: u32) -> Result<T, ErrorCL> {
        let mut value = MaybeUninit::<T>::uninit();
        
        unsafe {
            let err = clGetKernelArgInfo(self.0, idx, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut());
            if err == 0 {
                return Ok(value.assume_init());
            }
            
            Err(ErrorCL::from(err))
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