use cl_sys::{cl_mem_flags, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, CL_MEM_READ_ONLY, CL_MEM_USE_HOST_PTR, CL_MEM_ALLOC_HOST_PTR, CL_MEM_COPY_HOST_PTR};

bitflags::bitflags! {
    /// A bit-field that is used to specify allocation and usage information such as the memory arena that should be used to allocate the buffer object and how it will be used.
    #[repr(transparent)]
    pub struct MemFlags : cl_mem_flags {
        const READ_WRITE = CL_MEM_READ_WRITE;
        const WRITE_ONLY = CL_MEM_WRITE_ONLY;
        const READ_ONLY = CL_MEM_READ_ONLY;
        const USE_HOST_PTR = CL_MEM_USE_HOST_PTR;
        const ALLOC_HOST_PTR = CL_MEM_ALLOC_HOST_PTR;
        const COPY_HOST_PTR = CL_MEM_COPY_HOST_PTR;
    }
}

impl Default for MemFlags {
    #[inline(always)]
    fn default() -> Self {
        Self::READ_WRITE
    }
}