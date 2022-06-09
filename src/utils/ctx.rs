use crate::{prelude::{Context, Device, ErrorCL, CommandQueue}, context::ContextProps};

pub struct ContextManager {
    // We usually won't be accessing the values by key, rather iterating on them,
    // so a vec is more efficient.
    queues: Vec<CommandQueue>
}

impl ContextManager {
    pub fn new (devices: &[Device], props: Option<ContextProps>) -> Result<Self, ErrorCL> {
        let ctx = Context::new(props, &devices)?;

        todo!()
    }
}