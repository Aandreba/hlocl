use opencl_sys::{cl_svm_mem_flags, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, CL_MEM_READ_ONLY, CL_MEM_SVM_FINE_GRAIN_BUFFER, CL_MEM_SVM_ATOMICS};

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct SvmFlag: cl_svm_mem_flags {
        /// This flag specifies that the SVM buffer will be read and written by a kernel. This is the default.
        const READ_WRITE = CL_MEM_READ_WRITE;
        /// This flag specifies that the SVM buffer will be written but not read by a kernel. Reading from a SVM buffer created with [`Self::WRITE_ONLY`] inside a kernel is undefined. [`Self::READ_WRITE`] and [`Self::WRITE_ONLY`] are mutually exclusive.
        const WRITE_ONLY = CL_MEM_WRITE_ONLY;
        /// This flag specifies that the SVM buffer object is a read-only memory object when used inside a kernel. Writting to a SVM buffer created with [`Self::READ_ONLY`] inside a kernel is undefined. [`Self::READ_WRITE`] and [`Self::READ_ONLY`] are mutually exclusive.
        const READ_ONLY = CL_MEM_READ_ONLY;
        /// This specifies that the application wants the OpenCL implementation to do a fine-grained allocation.
        const FINE_GRAIN_BUFFER = CL_MEM_SVM_FINE_GRAIN_BUFFER;
        /// This flag is valid only if [`Self::FINE_GRAIN_BUFFER`] is specified in flags. It is used to indicate that SVM atomic operations can control visibility of memory accesses in this SVM buffer.
        const ATOMICS = CL_MEM_SVM_ATOMICS;
    }
}

impl Default for SvmFlag {
    #[inline(always)]
    fn default() -> Self {
        Self::READ_WRITE
    }
}