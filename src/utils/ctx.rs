use core::{sync::atomic::{AtomicUsize, Ordering}, ops::{Deref, DerefMut}};
use alloc::vec::Vec;
use crate::{prelude::{Context, Device, Result, CommandQueue}, context::ContextProps, queue::CommandQueueProps};

#[cfg(feature = "def")]
lazy_static! {
    static ref MANAGER: ContextManager = ContextManager::new(Device::all(), None, None).expect("Error initializing ContextManager");
}

pub struct ContextManager {
    ctx: Context,
    idx: AtomicUsize,
    queues: Vec<CommandQueue>
}

impl ContextManager {
    pub fn new (devices: &[Device], ctx_props: Option<ContextProps>, queue_props: Option<CommandQueueProps>) -> Result<Self> {
        let ctx = Context::new(ctx_props, &devices)?;
        let mut queues = Vec::with_capacity(devices.len());

        for device in devices {
            let queue = CommandQueue::new(&ctx, device, queue_props)?;
            queues.push(queue);
        }
        
        Ok(Self {
            ctx,
            idx: AtomicUsize::new(0),
            queues
        })
    }

    #[cfg(feature = "def")]
    #[inline(always)]
    pub fn default () -> &'static ContextManager {
        once_cell::sync::Lazy::force(&MANAGER)
    }

    #[inline(always)]
    pub fn context (&self) -> &Context {
        &self.ctx
    }

    #[inline(always)]
    pub fn queue (&self) -> &CommandQueue {
        let idx = self.idx.fetch_update(Ordering::Acquire, Ordering::Acquire, |x| {
            let next = x + 1;
            if next >= self.queues.len() { return Some(0) }
            Some(next)
        }).unwrap();
        
        &self.queues[idx]
    }
}

impl Deref for ContextManager {
    type Target = Context;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for ContextManager {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}