use core::{mem::MaybeUninit, intrinsics::transmute, num::{NonZeroUsize, NonZeroU32, NonZeroU64}, fmt::Debug};
use alloc::{vec::Vec, string::{String, FromUtf8Error}};
use cl_sys::{cl_device_id, clGetDeviceIDs, CL_DEVICE_TYPE_ALL, cl_device_info, clGetDeviceInfo, c_uchar, CL_DEVICE_PLATFORM, cl_uint, CL_DEVICE_ADDRESS_BITS, cl_bool, CL_DEVICE_AVAILABLE, CL_FP_DENORM, CL_FP_INF_NAN, CL_FP_ROUND_TO_NEAREST, CL_FP_ROUND_TO_ZERO, CL_FP_ROUND_TO_INF, cl_device_fp_config, CL_DEVICE_DOUBLE_FP_CONFIG, CL_DEVICE_ENDIAN_LITTLE, CL_DEVICE_ERROR_CORRECTION_SUPPORT, cl_device_exec_capabilities, CL_EXEC_KERNEL, CL_EXEC_NATIVE_KERNEL, CL_DEVICE_EXECUTION_CAPABILITIES, CL_DEVICE_EXTENSIONS, cl_ulong, CL_DEVICE_GLOBAL_MEM_CACHE_SIZE, CL_NONE, CL_READ_ONLY_CACHE, cl_device_mem_cache_type, CL_DEVICE_GLOBAL_MEM_CACHE_TYPE, CL_READ_WRITE_CACHE, CL_DEVICE_GLOBAL_MEM_CACHELINE_SIZE, CL_DEVICE_GLOBAL_MEM_SIZE, CL_DEVICE_HALF_FP_CONFIG, CL_DEVICE_IMAGE_SUPPORT, size_t, CL_DEVICE_IMAGE2D_MAX_HEIGHT, CL_DEVICE_IMAGE2D_MAX_WIDTH, CL_DEVICE_IMAGE3D_MAX_WIDTH, CL_DEVICE_IMAGE3D_MAX_HEIGHT, CL_DEVICE_IMAGE3D_MAX_DEPTH, CL_DEVICE_LOCAL_MEM_SIZE, CL_LOCAL, CL_GLOBAL, cl_device_local_mem_type, CL_DEVICE_LOCAL_MEM_TYPE, CL_DEVICE_MAX_CLOCK_FREQUENCY, CL_DEVICE_MAX_COMPUTE_UNITS, CL_DEVICE_MAX_CONSTANT_ARGS, CL_DEVICE_MAX_CONSTANT_BUFFER_SIZE, CL_DEVICE_MAX_MEM_ALLOC_SIZE, CL_DEVICE_MAX_PARAMETER_SIZE, CL_DEVICE_MAX_READ_IMAGE_ARGS, CL_DEVICE_MAX_SAMPLERS, CL_DEVICE_MAX_WORK_GROUP_SIZE, CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS, CL_DEVICE_MAX_WORK_ITEM_SIZES, CL_DEVICE_MAX_WRITE_IMAGE_ARGS, CL_DEVICE_MEM_BASE_ADDR_ALIGN, CL_DEVICE_MIN_DATA_TYPE_ALIGN_SIZE, CL_DEVICE_NAME, CL_DEVICE_PREFERRED_VECTOR_WIDTH_CHAR, CL_DEVICE_PREFERRED_VECTOR_WIDTH_SHORT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_INT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_LONG, CL_DEVICE_PREFERRED_VECTOR_WIDTH_FLOAT, CL_DEVICE_PREFERRED_VECTOR_WIDTH_DOUBLE, CL_DEVICE_PROFILE, CL_DEVICE_PROFILING_TIMER_RESOLUTION, CL_DEVICE_QUEUE_PROPERTIES, CL_DEVICE_SINGLE_FP_CONFIG, cl_device_type, CL_DEVICE_TYPE_CPU, CL_DEVICE_TYPE_GPU, CL_DEVICE_TYPE_ACCELERATOR, CL_DEVICE_TYPE_CUSTOM, CL_DEVICE_TYPE, CL_DEVICE_VENDOR, CL_DEVICE_VENDOR_ID, CL_DEVICE_VERSION, CL_DRIVER_VERSION};
use crate::{platform::Platform, queue::CommandQueueProps};

lazy_static::lazy_static! {
    static ref DEVICES : Vec<Device> = unsafe {
        let mut result = Vec::<Device>::new();

        for platform in Platform::all() {
            let mut cnt = 0;
            tri_panic!(clGetDeviceIDs(platform.id(), CL_DEVICE_TYPE_ALL, 0, core::ptr::null_mut(), &mut cnt));
            let cnt_size = usize::try_from(cnt).unwrap();

            result.reserve(cnt_size);
            tri_panic!(clGetDeviceIDs(platform.id(), CL_DEVICE_TYPE_ALL, cnt, result.as_mut_ptr().add(result.len()).cast(), core::ptr::null_mut()));
            result.set_len(result.len() + cnt_size);
        }

        result
    };
}

/// OpenCL device
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Device (cl_device_id);

impl Device {
    #[inline(always)]
    pub fn id (&self) -> cl_device_id {
        self.0
    }

    /// The default compute device address space size specified as an unsigned integer value in bits. Currently supported values are 32 or 64 bits.
    #[inline(always)]
    pub fn address_bits (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_ADDRESS_BITS)
    }

    /// Is ```true``` if the device is available and ```false``` if the device is not available.
    #[inline(always)]
    pub fn available (&self) -> bool {
        self.get_info_bits::<cl_bool>(CL_DEVICE_AVAILABLE) != 0
    }

    /// Describes the OPTIONAL double precision floating-point capability of the OpenCL device
    #[inline(always)]
    pub fn double_fp_config (&self) -> FpConfig {
        self.get_info_bits(CL_DEVICE_DOUBLE_FP_CONFIG)
    }

    /// Is ```true``` if the OpenCL device is a little endian device and ```false``` otherwise.
    #[inline(always)]
    pub fn endian_little (&self) -> bool {
        self.get_info_bits::<cl_bool>(CL_DEVICE_ENDIAN_LITTLE) != 0
    }

    /// Is ```true``` if the device implements error correction for the memories, caches, registers etc. in the device. Is ```false``` if the device does not implement error correction. This can be a requirement for certain clients of OpenCL.
    #[inline(always)]
    pub fn error_connection_support (&self) -> bool {
        self.get_info_bits::<cl_bool>(CL_DEVICE_ERROR_CORRECTION_SUPPORT) != 0
    }

    /// Describes the execution capabilities of the device
    #[inline(always)]
    pub fn execution_capabilities (&self) -> ExecCapabilities {
        self.get_info_bits(CL_DEVICE_EXECUTION_CAPABILITIES)
    }

    /// Returns a list of extension names (the extension names themselves do not contain any spaces)
    #[inline]
    pub fn extensions (&self) -> Vec<String> {
        self.get_info_string(CL_DEVICE_EXTENSIONS).unwrap()
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
    }

    /// Size of global memory cache in bytes.
    #[inline(always)]
    pub fn global_mem_cache_size (&self) -> cl_ulong {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_CACHE_SIZE)
    }

    /// Type of global memory cache supported.
    #[inline(always)]
    pub fn global_mem_cache_type (&self) -> Option<MemCacheType> {
        match self.get_info_bits::<cl_device_mem_cache_type>(CL_DEVICE_GLOBAL_MEM_CACHE_TYPE) {
            CL_NONE => None,
            other => unsafe { Some(transmute(other)) }
        }
    }

    /// Size of global memory cache line in bytes.
    #[inline(always)]
    pub fn global_mem_cahceline_size (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_CACHELINE_SIZE)
    }

    /// Size of global memory in bytes.
    #[inline(always)]
    pub fn global_mem_size (&self) -> cl_ulong {
        self.get_info_bits(CL_DEVICE_GLOBAL_MEM_SIZE)
    }

    /// Describes the OPTIONAL half precision floating-point capability of the OpenCL device
    #[inline(always)]
    pub fn half_fp_config (&self) -> FpConfig {
        self.get_info_bits(CL_DEVICE_HALF_FP_CONFIG)
    }
    
    /// Is ```true``` if images are supported by the OpenCL device and ```false``` otherwise.
    #[inline(always)]
    pub fn image_support (&self) -> bool {
        self.get_info_bits::<cl_bool>(CL_DEVICE_IMAGE_SUPPORT) != 0
    }

    /// Max height of 2D image in pixels. The minimum value is 8192 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image2d_max_height (&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.get_info_bits::<size_t>(CL_DEVICE_IMAGE2D_MAX_HEIGHT))
    }

    /// Max width of 2D image in pixels. The minimum value is 8192 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image2d_max_width (&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.get_info_bits::<size_t>(CL_DEVICE_IMAGE2D_MAX_WIDTH))
    }

    /// Max depth of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_depth (&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.get_info_bits::<size_t>(CL_DEVICE_IMAGE3D_MAX_DEPTH))
    }

    /// Max height of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_height (&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.get_info_bits::<size_t>(CL_DEVICE_IMAGE3D_MAX_HEIGHT))
    }

    /// Max width of 3D image in pixels. The minimum value is 2048 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn image3d_max_width (&self) -> Option<NonZeroUsize> {
        NonZeroUsize::new(self.get_info_bits::<size_t>(CL_DEVICE_IMAGE3D_MAX_WIDTH))
    }

    /// Size of local memory arena in bytes. The minimum value is 16 KB.
    #[inline(always)]
    pub fn local_mem_size (&self) -> NonZeroU64 {
        unsafe {
            NonZeroU64::new_unchecked(self.get_info_bits::<cl_ulong>(CL_DEVICE_LOCAL_MEM_SIZE))
        }
    }

    /// Type of local memory supported.
    #[inline(always)]
    pub fn local_mem_type (&self) -> LocalMemType {
        unsafe { transmute(self.get_info_bits::<cl_device_local_mem_type>(CL_DEVICE_LOCAL_MEM_TYPE)) }
    }

    /// Maximum configured clock frequency of the device in MHz.
    #[inline(always)]
    pub fn max_clock_frequency (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_MAX_CLOCK_FREQUENCY)
    }

    /// The number of parallel compute cores on the OpenCL device. The minimum value is 1.
    #[inline(always)]
    pub fn max_compute_units (&self) -> NonZeroU32 {
        unsafe { 
            NonZeroU32::new_unchecked(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_COMPUTE_UNITS))
        }
    }

    /// Max number of arguments declared with the ```__constant``` qualifier in a kernel. The minimum value is 8.
    #[inline(always)]
    pub fn max_constant_args (&self) -> NonZeroU32 {
        unsafe { 
            NonZeroU32::new_unchecked(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_CONSTANT_ARGS))
        }
    }

    /// Max size in bytes of a constant buffer allocation. The minimum value is 64 KB.
    #[inline(always)]
    pub fn max_constant_buffer_size (&self) -> NonZeroU64 {
        unsafe { 
            NonZeroU64::new_unchecked(self.get_info_bits::<cl_ulong>(CL_DEVICE_MAX_CONSTANT_BUFFER_SIZE))
        }
    }

    /// Max size of memory object allocation in bytes. The minimum value is max (1/4th of [```global_mem_size```](), 128*1024*1024)
    #[inline(always)]
    pub fn max_mem_alloc_size (&self) -> NonZeroU64 {
        unsafe { 
            NonZeroU64::new_unchecked(self.get_info_bits::<cl_ulong>(CL_DEVICE_MAX_MEM_ALLOC_SIZE))
        }
    }

    /// Max size in bytes of the arguments that can be passed to a kernel. The minimum value is 256.
    #[inline(always)]
    pub fn max_parameter_size (&self) -> NonZeroUsize {
        unsafe { 
            NonZeroUsize::new_unchecked(self.get_info_bits::<size_t>(CL_DEVICE_MAX_PARAMETER_SIZE))
        }
    }

    /// Max number of simultaneous image objects that can be read by a kernel. The minimum value is 128 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_read_image_args (&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_READ_IMAGE_ARGS))
    }

    /// Maximum number of samplers that can be used in a kernel. The minimum value is 16 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_samplers (&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_SAMPLERS))
    }

    /// Maximum number of work-items in a work-group executing a kernel using the data parallel execution model. The minimum value is 1.
    #[inline(always)]
    pub fn max_work_group_size (&self) -> NonZeroUsize {
        unsafe {
            NonZeroUsize::new_unchecked(self.get_info_bits::<size_t>(CL_DEVICE_MAX_WORK_GROUP_SIZE))
        }
    }

    /// Maximum dimensions that specify the global and local work-item IDs used by the data parallel execution model. The minimum value is 3.
    #[inline(always)]
    pub fn max_work_item_dimensions (&self) -> NonZeroU32 {
        unsafe {
            NonZeroU32::new_unchecked(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS))
        }
    }

    /// Maximum number of work-items that can be specified in each dimension of the work-group to clEnqueueNDRangeKernel. Returns n ```usize``` entries, where n is the value returned by the query for [```max_work_item_dimensions```]. The minimum value is (1, 1, 1).
    #[inline(always)]
    pub fn max_work_item_sizes (&self) -> Vec<NonZeroUsize> {
        let n = usize::try_from(self.max_work_item_dimensions().get()).unwrap();
        // FIXME: maybe using nonzero ints messes up the alignment?
        let mut max_work_item_sizes = Vec::<NonZeroUsize>::with_capacity(n);

        let len = n.checked_mul(core::mem::size_of::<size_t>()).expect("Integer multiplication oveflow. Too many work items to fit in a vector");
        unsafe {
            clGetDeviceInfo(self.0, CL_DEVICE_MAX_WORK_ITEM_SIZES, len, max_work_item_sizes.as_mut_ptr().cast(), core::ptr::null_mut());
            max_work_item_sizes.set_len(n);
        }

        max_work_item_sizes
    }

    /// Max number of simultaneous image objects that can be written to by a kernel. The minimum value is 8 if [```image_support```] is ```true```.
    #[inline(always)]
    pub fn max_write_image_args (&self) -> Option<NonZeroU32> {
        NonZeroU32::new(self.get_info_bits::<cl_uint>(CL_DEVICE_MAX_WRITE_IMAGE_ARGS))
    }

    /// Describes the alignment in bits of the base address of any allocated memory object.
    #[inline(always)]
    pub fn mem_base_addr_align (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_MEM_BASE_ADDR_ALIGN)
    }

    /// The smallest alignment in bytes which can be used for any data type.
    #[inline(always)]
    pub fn min_data_type_align_size (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_MIN_DATA_TYPE_ALIGN_SIZE)
    }

    /// Device name string.
    #[inline(always)]
    pub fn name (&self) -> String {
        self.get_info_string(CL_DEVICE_NAME).unwrap()
    }

    /// The platform associated with this device.
    #[inline(always)]
    pub fn platform (&self) -> Platform {
        self.get_info_bits(CL_DEVICE_PLATFORM)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_char (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_CHAR)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_short (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_SHORT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_int (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_INT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_long (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_LONG)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector.
    #[inline(always)]
    pub fn preferred_vector_width_float (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_FLOAT)
    }

    /// Preferred native vector width size for built-in scalar types that can be put into vectors. The vector width is defined as the number of scalar elements that can be stored in the vector. if the ```cl_khr_fp64``` extension is not supported, it must return 0.
    #[inline(always)]
    pub fn preferred_vector_width_double (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_PREFERRED_VECTOR_WIDTH_DOUBLE)
    }

    /// OpenCL profile string. Returns the profile name supported by the device (see note)
    #[inline(always)]
    pub fn profile (&self) -> String {
        self.get_info_string(CL_DEVICE_PROFILE).unwrap()
    }

    /// Describes the resolution of device timer. This is measured in nanoseconds.
    #[inline(always)]
    pub fn profiling_timer_resolution (&self) -> size_t {
        self.get_info_bits(CL_DEVICE_PROFILING_TIMER_RESOLUTION)
    }

    /// Describes the command-queue properties supported by the device.
    #[inline(always)]
    pub fn queue_properties (&self) -> CommandQueueProps {
        self.get_info_bits(CL_DEVICE_QUEUE_PROPERTIES)
    }

    /// Describes single precision floating-point capability of the device.
    #[inline(always)]
    pub fn single_fp_config (&self) -> FpConfig {
        self.get_info_bits(CL_DEVICE_SINGLE_FP_CONFIG)
    }

    /// The OpenCL device type.
    #[inline(always)]
    pub fn ty (&self) -> DeviceType {
        self.get_info_bits(CL_DEVICE_TYPE)
    }

    /// Vendor name string.
    #[inline(always)]
    pub fn vendor (&self) -> String {
        self.get_info_string(CL_DEVICE_VENDOR).unwrap()
    }

    /// A unique device vendor identifier. An example of a unique device identifier could be the PCIe ID.
    #[inline(always)]
    pub fn vendor_id (&self) -> cl_uint {
        self.get_info_bits(CL_DEVICE_VENDOR_ID)
    }

    /// OpenCL version string.
    #[inline(always)]
    pub fn version (&self) -> String {
        self.get_info_string(CL_DEVICE_VERSION).unwrap()
    }

    /// OpenCL software driver version string in the form _major_number_._minor_number_.
    #[inline(always)]
    pub fn driver_version (&self) -> String {
        self.get_info_string(CL_DRIVER_VERSION).unwrap()
    }
    
    #[inline(always)]
    pub fn all () -> &'static [Device] {
        &DEVICES
    }

    #[inline(always)]
    pub fn from_platform (platform: Platform) -> impl Iterator<Item = Device> {
        DEVICES.iter().copied().filter(move |x| x.platform() == platform)
    }

    #[inline]
    fn get_info_string (&self, ty: cl_device_info) -> Result<String, FromUtf8Error> {
        unsafe {
            let mut len = 0;
            tri_panic!(clGetDeviceInfo(self.0, ty, 0, core::ptr::null_mut(), &mut len));

            let mut result = Vec::<c_uchar>::with_capacity(len);
            tri_panic!(clGetDeviceInfo(self.0, ty, len * core::mem::size_of::<c_uchar>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            
            result.set_len(len - 1);
            String::from_utf8(result)
        }
    }

    #[inline]
    fn get_info_bits<T: Copy> (&self, ty: cl_device_info) -> T {
        unsafe {
            let mut value = MaybeUninit::<T>::uninit();
            tri_panic!(clGetDeviceInfo(self.0, ty, core::mem::size_of::<T>(), value.as_mut_ptr().cast(), core::ptr::null_mut()));
            value.assume_init()
        }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Device")
        .field("name", &self.name())
        .field("vendor", &self.vendor())
        .field("vendor_id", &self.vendor_id())
        .field("profile", &self.profile())
        .field("version", &self.version())
        .field("driver_version", &self.driver_version())
        .field("extensions", &self.extensions())
        .field("address_bits", &self.address_bits())
        .field("available", &self.available())
        .field("double_fp_config", &self.double_fp_config())
        .field("endian_little", &self.endian_little())
        .field("error_correction_support", &self.error_connection_support())
        .field("execution_capabilities", &self.execution_capabilities())
        .field("global_mem_cache_size", &self.global_mem_cache_size())
        .field("global_mem_cache_type", &self.global_mem_cache_type())
        .field("global_mem_cacheline_size", &self.global_mem_cahceline_size())
        .field("global_mem_size", &self.global_mem_size())
        .field("image_support", &self.image_support())
        .field("image2d_max_height", &self.image2d_max_height())
        .field("image2d_max_width", &self.image2d_max_width())
        .field("image3d_max_depth", &self.image3d_max_depth())
        .field("image3d_max_height", &self.image3d_max_height())
        .field("image3d_max_width", &self.image3d_max_width())
        .field("local_mem_size", &self.local_mem_size())
        .field("local_mem_type", &self.local_mem_type())
        .field("max_clock_frequency", &self.max_clock_frequency())
        .field("max_compute_units", &self.max_compute_units())
        .field("max_constant_args", &self.max_constant_args())
        .field("max_constant_buffer_size", &self.max_constant_buffer_size())
        .field("max_mem_alloc_size", &self.max_mem_alloc_size())
        .field("max_parameter_size", &self.max_parameter_size())
        .field("max_read_image_args", &self.max_read_image_args())
        .field("max_samplers", &self.max_samplers())
        .field("max_work_group_size", &self.max_work_group_size())
        .field("max_work_item_dimensions", &self.max_work_item_dimensions())
        .field("max_work_item_sizes", &self.max_work_item_sizes())
        .field("max_write_image_args", &self.max_write_image_args())
        .field("mem_base_addr_align", &self.mem_base_addr_align())
        .field("min_data_type_align_size", &self.min_data_type_align_size())
        .field("preferred_vector_width_char", &self.preferred_vector_width_char())
        .field("preferred_vector_width_double", &self.preferred_vector_width_double())
        .field("preferred_vector_width_float", &self.preferred_vector_width_float())
        .field("preferred_vector_width_int", &self.preferred_vector_width_int())
        .field("preferred_vector_width_long", &self.preferred_vector_width_long())
        .field("preferred_vector_width_short", &self.preferred_vector_width_short())
        .field("profile", &self.profile())
        .field("profiling_timer_resolution", &self.profiling_timer_resolution())
        .field("queue_properties", &self.queue_properties())
        .field("single_fp_config", &self.single_fp_config())
        .field("type", &self.ty())
        .field("version", &self.version())
        .finish()
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

bitflags::bitflags! {
    /// The OpenCL device type.
    #[repr(transparent)]
    pub struct DeviceType : cl_device_type {
        const CPU = CL_DEVICE_TYPE_CPU;
        const GPU = CL_DEVICE_TYPE_GPU;
        const ACCELERATOR = CL_DEVICE_TYPE_ACCELERATOR;
        const DEFAULT = CL_DEVICE_TYPE_CUSTOM;
    }

    /// Describes the floating-point capability of the OpenCL device.
    #[repr(transparent)]
    pub struct FpConfig : cl_device_fp_config {
        const DENORM = CL_FP_DENORM;
        const INF_NAN = CL_FP_INF_NAN;
        const ROUND_TO_NEAREST = CL_FP_ROUND_TO_NEAREST;
        const ROUND_TO_ZERO = CL_FP_ROUND_TO_ZERO;
        const ROUND_TO_INF = CL_FP_ROUND_TO_INF;
    }

    /// Describes the execution capabilities of the device
    #[repr(transparent)]
    pub struct ExecCapabilities : cl_device_exec_capabilities {
        const KERNEL = CL_EXEC_KERNEL;
        const NATIVE_KERNEL = CL_EXEC_NATIVE_KERNEL;
    }
}

/// Type of global memory cache supported.
#[derive(Debug)]
#[repr(u32)]
pub enum MemCacheType {
    ReadOnly = CL_READ_ONLY_CACHE,
    ReadWrite = CL_READ_WRITE_CACHE,
}

/// Type of local memory supported. This can be set to [```Self::Local```] implying dedicated local memory storage such as SRAM, or [```Self::Global```].
#[derive(Debug)]
#[repr(u32)]
pub enum LocalMemType {
    Local = CL_LOCAL,
    Global = CL_GLOBAL
}