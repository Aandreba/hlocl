use cl_sys::{clCreateUserEvent, clSetUserEventStatus};
use crate::prelude::{ErrorCL, Context};
use super::{BaseEvent, EventStatus};

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UserEvent (BaseEvent);

impl UserEvent {
    #[inline(always)]
    pub fn new (ctx: &Context) -> Result<Self, ErrorCL> {
        let mut err = 0;
        let id = unsafe {
            clCreateUserEvent(ctx.0, &mut err)
        };

        if err == 0 {
            return Err(ErrorCL::from(err));
        }

        let id = BaseEvent::new(id)?;
        Ok(Self(id))
    }

    #[inline(always)]
    pub fn set_status (&self, status: EventStatus) -> Result<(), ErrorCL> {
        let err = unsafe {
            clSetUserEventStatus(self.0.0, status as i32)
        };

        if err == 0 {
            return Ok(())
        }

        Err(ErrorCL::from(err))
    }
}