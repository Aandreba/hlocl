use core::{fmt::Display, hint::unreachable_unchecked};

#[cfg(feature = "error-stack")]
pub type Result<T> = error_stack::Result<T, ErrorCL>;
#[cfg(not(feature = "error-stack"))]
pub type Result<T> = Result<T, ErrorCL>;

const ERROR_MESSAGES_1 : &[&str] = &[
    "Device not found",
    "Device not available",
    "Compiler not available",
    "Memory object allocation failure",
    "Out of resources",
    "Out of host memory",
    "Profiling info not available",
    "Memory copy overlap",
    "Image format mismatch",
    "Image format not supported",
    "Build program failure",
    "Map failure",
    "Misaligned sub-buffer offset",
    "Execution status error for events in wait list",
    "Program compilation failure",
    "Linker not available",
    "Program linking failure",
    "Device partition failure",
    "Kernel argument info not available"
];

const ERROR_MESSAGES_2 : &[&str] = &[
    "Invalid value",
    "Invalid device type",
    "Invalid platform",
    "Invalid device",
    "Invalid context",
    "Invalid queue properties",
    "Invalid command queue",
    "Invalid host pointer",
    "Invalid memory object",
    "Invalid image format descriptor",
    "Invalid image size",
    "Invalid sampler",
    "Invalid binary",
    "Invalid build options",
    "Invalid program",
    "Invalid program executable",
    "Invalid kernel name",
    "Invalid kernel definition",
    "Inavlid argument index",
    "Invalid argument value",
    "Invalid argument size",
    "Invalid kernel arguments",
    "Invalid work dimension",
    "Invalid work group size",
    "Invalid work item size",
    "Invalid global offset",
    "Invalid event wait list",
    "Invalid event",
    "Invalid operation",
    "Invalid gl object",
    "Invalid buffer size",
    "Invalid mip level",
    "Invalid global work size",
    "Invalid property",
    "Invalid image descriptor",
    "Invalid compiler options",
    "Invalid linker options",
    "Invalid device partiton count",
    "Invalid pipe size",
    "Invalid device queue"
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCL {
    DeviceNotFound = -1,
    DeviceNotAvailable = -2,
    CompilerNotAvailable = -3,
    MemObjectAllocationFailure = -4,
    OutOfResources = -5,
    OutOfHostMemory = -6,
    ProfilingInfoNotAvailable = -7,
    MemCopyOverlap = -8,
    ImageFormatMismatch = -9,
    ImageFormatNotSupported = -10,
    BuildProgramFailure = -11,
    MapFailure = -12,
    MisalignedSubBufferOffset = -13,
    ExecutionStatusErrorForEventsInWaitList = -14,
    CompileProgramFailure = -15,
    LinkerNotAvailable = -16,
    LinkProgramFailure = -17,
    DevicePartitionFailed = -18,
    KernelArgInfoNotAvailable = -19,
    InvalidValue = -30,
    InvalidDeviceType = -31,
    InvalidPlatform = -32,
    InvalidDevice = -33,
    InvalidContext = -34,
    InvalidQueueProperties = -35,
    InvalidCommandQueue = -36,
    InvalidHostPtr = -37,
    InvalidMemObject = -38,
    InvalidImageFormatDescriptor = -39,
    InvalidImageSize = -40,
    InvalidSampler = -41,
    InvalidBinary = -42,
    InvalidBuildOptions = -43,
    InvalidProgram = -44,
    InvalidProgramExecutable = -45,
    InvalidKernelName = -46,
    InvalidKernelDefinition = -47,
    InvalidKernel = -48,
    InvalidArgIndex = -49,
    InvalidArgValue = -50,
    InvalidArgSize = -51,
    InvalidKernelArgs = -52,
    InvalidWorkDimension = -53,
    InvalidWorkGroupSize = -54,
    InvalidWorkItemSize = -55,
    InvalidGlobalOffset = -56,
    InvalidEventWaitList = -57,
    InvalidEvent = -58,
    InvalidOperation = -59,
    InvalidGlObject = -60,
    InvalidBufferSize = -61,
    InvalidMipLevel = -62,
    InvalidGlobalWorkSize = -63,
    InvalidProperty = -64,
    InvalidImageDescriptor = -65,
    InvalidCompilerOptions = -66,
    InvalidLinkerOptions = -67,
    InvalidDevicePartitionCount = -68,
    InvalidPipeSize = -69,
    InvalidDeviceQueue = -70,
    NvidiaIllegalBufferAction = -9999
}

#[cfg(feature = "error-stack")]
impl error_stack::Context for ErrorCL {}

impl Display for ErrorCL {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let int = -(*self as i32) as usize;
        let msg = match int {
            1..=19 => ERROR_MESSAGES_1[int - 1],
            30..=70 => ERROR_MESSAGES_2[int - 30],
            9999 => "Invalid buffer read/write",
            _ => unsafe { unreachable_unchecked() }
        };

        msg.fmt(f)
    }
}

impl Into<i32> for ErrorCL {
    #[inline(always)]
    fn into(self) -> i32 {
        self as i32
    }
}

impl From<i32> for ErrorCL {
    #[inline(always)]
    fn from(value: i32) -> Self {
        match value {
            -68..=-30 | -19..=-1 | -9999 => unsafe { core::mem::transmute(value) },
            _ => panic!("invalid error code: {}", value)
        }
    }
}