use std::{collections::VecDeque, time::Duration};

use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

use crate::{Producer, producer::Terminate};



/// This module provide a data collector
/// 
/// listen on channel get and push to buffer
/// 
///     * when buffer was full then dispatch 
///     * or timeout happen, then dispatch( if buffer.len > 0) 

pub struct Collector<Input> {
    recv: Receiver<Input>,
    timeout: Duration
}

impl<Input> Collector<Input> {
    pub fn new(recv: Receiver<Input>, timeout: Duration) -> Self {
        Collector {
            recv,
            timeout
        }
    }
}


#[async_trait]
impl<Input> Producer<Input> for Collector<Input> 
where
    Input: Send
{
   
    async fn init(&mut self) { }

    async fn terminate(&mut self) { }

    async fn drain(&mut self, _buffer: VecDeque<Input>) { }

    async fn fill_buffer(&mut self, buffer_size:usize) ->  Result<VecDeque<Input>, Terminate> {

        let mut buffer = VecDeque::with_capacity(buffer_size);

        let sleep = tokio::time::sleep(self.timeout);
        tokio::pin!(sleep);
        
        loop {
            tokio::select! {
                _ = &mut sleep => {

                    // if was not empty
                    if buffer.len() > 0 {

                        // return buffer
                        return Ok(buffer)
                    }
                }
                res = self.recv.recv() => {
                    match res {
                        Some(stream) => {
                            buffer.push_back(stream);
                            if buffer.len() == buffer_size {
                                return Ok(buffer)
                            }
                        }
                        None => {
                            // not exist any sender, shutdown
                            if buffer.len() > 0 {
                                
                                // return buffer
                                return Ok(buffer)
                            
                            } else {
                                return Err(Terminate);
                            }
                        }
                    }
                }
            }
        }
    }

}

