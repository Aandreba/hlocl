use cl_sys::{cl_mem_flags, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, CL_MEM_READ_ONLY, CL_MEM_USE_HOST_PTR, CL_MEM_ALLOC_HOST_PTR, CL_MEM_COPY_HOST_PTR};

bitflags::bitflags! {
    /// A bit-field that is used to specify allocation and usage information such as the memory arena that should be used to allocate the buffer object and how it will be used.
    #[repr(transparent)]
    pub struct MemFlag : cl_mem_flags {
        /// This flag specifies that the memory object will be read and written by a kernel. This is the default.
        const READ_WRITE = CL_MEM_READ_WRITE;
        /// This flag specifies that the memory object is a write-only memory object when used inside a kernel. Reading from a buffer or image object created with CL_MEM_WRITE_ONLY inside a kernel is undefined.
        const WRITE_ONLY = CL_MEM_WRITE_ONLY;
        /// This flag specifies that the memory object is a read-only memory object when used inside a kernel. Writing to a buffer or image object created with CL_MEM_READ_ONLY inside a kernel is undefined.
        const READ_ONLY = CL_MEM_READ_ONLY;
        const USE_HOST_PTR = CL_MEM_USE_HOST_PTR;
        const ALLOC_HOST_PTR = CL_MEM_ALLOC_HOST_PTR;
        const COPY_HOST_PTR = CL_MEM_COPY_HOST_PTR;
    }
}

impl Default for MemFlag {
    #[inline(always)]
    fn default() -> Self {
        Self::READ_WRITE
    }
}