macro_rules! impl_random {
    ($($kernel:ident = $u:ident as $uf:ident $(& $s:ident as $sf:ident)?),+) => {
        $(
            pub fn $uf (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<$u>, BaseEvent>> {
                assert_ne!(len, 0);
                
                let seeds_len = self.seeds.len()?;
                let max_wgs = queue.device()?.max_work_group_size()?.get();
                let wgs = seeds_len.min(max_wgs);
        
                let div = len / seeds_len;
                let rem = len % seeds_len;
        
                let mut this_wait = self.wait_for.lock();
                let wait_for = this_wait.iter()
                    .cloned()
                    .chain(wait.into_iter().map(|x| x.as_ref().clone()))
                    .collect::<Vec<_>>();
        
                let out = unsafe { MemBuffer::uninit_with_context(&self.context()?, len, flags)? };
                let mut kernel = self.$kernel.lock();
        
                let mut wait;
                if div > 0 {
                    wait = self.inner_random(queue, &mut kernel, &out, 0, len, wgs, wait_for)?;
                    for i in 1..div {
                        wait = self.inner_random(queue, &mut kernel, &out, i * seeds_len, len, wgs, [wait])?;
                    }
        
                    if rem > 0 {
                        wait = self.inner_random(queue, &mut kernel, &out, div * seeds_len, rem, wgs, [wait])?;
                    }
                } else {
                    wait = self.inner_random(queue, &mut kernel, &out, div * seeds_len, rem, wgs, wait_for)?;
                }
        
                drop(kernel);
                *this_wait = Some(wait.clone());
                drop(this_wait);
                Ok(wait.swap(out))
            }

            $(
                pub fn $sf (&self, queue: &CommandQueue, len: usize, flags: MemFlag, wait: impl IntoIterator<Item = impl AsRef<BaseEvent>>) -> Result<Swap<MemBuffer<$s>, BaseEvent>> {
                    assert_ne!(len, 0);
                    
                    let seeds_len = self.seeds.len()?;
                    let max_wgs = queue.device()?.max_work_group_size()?.get();
                    let wgs = seeds_len.min(max_wgs);
            
                    let div = len / seeds_len;
                    let rem = len % seeds_len;
            
                    let mut this_wait = self.wait_for.lock();
                    let wait_for = this_wait.iter()
                        .cloned()
                        .chain(wait.into_iter().map(|x| x.as_ref().clone()))
                        .collect::<Vec<_>>();
            
                    let out = unsafe { MemBuffer::uninit_with_context(&self.context()?, len, flags)? };
                    let mut kernel = self.$kernel.lock();
            
                    let mut wait;
                    if div > 0 {
                        wait = self.inner_random(queue, &mut kernel, &out, 0, len, wgs, wait_for)?;
                        for i in 1..div {
                            wait = self.inner_random(queue, &mut kernel, &out, i * seeds_len, len, wgs, [wait])?;
                        }
            
                        if rem > 0 {
                            wait = self.inner_random(queue, &mut kernel, &out, div * seeds_len, rem, wgs, [wait])?;
                        }
                    } else {
                        wait = self.inner_random(queue, &mut kernel, &out, div * seeds_len, rem, wgs, wait_for)?;
                    }
            
                    drop(kernel);
                    *this_wait = Some(wait.clone());
                    drop(this_wait);
                    Ok(wait.swap(out))
                }
            )?
        )*
    };
}