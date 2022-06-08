use cl_sys::{cl_command_queue_properties, CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE, CL_QUEUE_PROFILING_ENABLE, cl_command_queue};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CommandQueue (cl_command_queue);

bitflags::bitflags! {
    /// Describes the command-queue properties supported by the device.
    #[repr(transparent)]
    pub struct CommandQueueProps: cl_command_queue_properties {
        const OUT_OF_ORDER_EXEC_MODE_ENABLE = CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE;
        const PROFILING_ENABLE = CL_QUEUE_PROFILING_ENABLE;
    }
}