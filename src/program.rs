use cl_sys::{cl_program, clReleaseProgram, clCreateProgramWithSource, clRetainProgram, clBuildProgram};
use crate::{prelude::{ErrorCL, Context, Device}, context::ContextProps};

/// OpenCL program
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Program (pub(crate) cl_program);

impl Program {
    fn build (&self) -> Result<(), ErrorCL> {
        let result = unsafe {
            clBuildProgram(self.0, 0, core::ptr::null(), core::ptr::null(), None, core::ptr::null_mut())
        };

        if result == 0 {
            return Ok(());
        }

        Err(ErrorCL::from(result))
    }
}

impl Program {
    #[inline(always)]
    pub fn from_source (ctx: &Context, source: &str) -> Result<Self, ErrorCL> {
        let len = [source.len()].as_ptr();
        let strings = [source.as_ptr().cast()].as_ptr();

        let mut err = 0;
        let id = unsafe {
            clCreateProgramWithSource(ctx.0, 1, strings, len, &mut err)
        };

        if err != 0 {
            return Err(ErrorCL::from(err));
        }

        let this = Self(id);
        this.build()?;
        Ok(this)
    }

    #[inline(always)]
    pub fn context_from_source (props: Option<ContextProps>, devices: &[Device], source: &str) -> Result<(Context, Self), ErrorCL> {
        let ctx = Context::new(props, devices)?;
        let prog = Self::from_source(&ctx, source)?;
        Ok((ctx, prog))
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
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseProgram(self.0));
        }
    }
}