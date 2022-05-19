use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc;




pub struct BatcherTerminate<T>(pub Vec<T>);



#[async_trait]
pub trait BatchProcessor<Input> {
    
    async fn init(&mut self);
    
    /// if return `ProcResult::Dispatch` and Dispatcher mode is `Partition` must set `batch_key`

    async fn handle_batch(&mut self, batch: Vec<Input>) -> Result<(), BatcherTerminate<Input>>;

    async fn drain(&mut self, batch: Vec<Input>);

    async fn terminate(&mut self);
}



pub struct Context<Input, Proc>
where
    Input: Send + 'static,
    Proc: BatchProcessor<Input> + Send + 'static
{
    recv: mpsc::Receiver<Input>,
    
    batch_size: usize,
    batch_timeout: Duration,

    proc: Proc
}

impl<Input, Proc> Context<Input, Proc> 
where
    Input  : Clone + Send + 'static,
    Proc   : BatchProcessor<Input> + Send + 'static
{
    
    pub fn new(recv: mpsc::Receiver<Input>,
               proc: Proc,
               batch_size: usize,
               batch_timeout: Duration
               ) -> Self 
    {
        Context { 
            recv, 
            batch_size: batch_size,
            batch_timeout,
            proc,  
        }
    }


    #[inline]
    pub fn run(mut self) {        
        // spawn
        tokio::spawn(async move {            

            // init Processor
            self.proc.init().await;
            

            // create time for timeout
            let sleep = tokio::time::sleep(self.batch_timeout);
            tokio::pin!(sleep);
        

            // create batch empty
            let mut batch = Vec::with_capacity(self.batch_size); 

            loop {
                tokio::select! {
                    _ = &mut sleep => {
                        
                        // if batch was not empty 
                        if batch.len() > 0 {

                            // handle_batch
                            if let Err(bt) = self.proc.handle_batch(batch).await {
                                
                                // drain
                                self.proc.drain(bt.0).await;

                                self.proc.terminate().await;
                            }

                            // create batch empty
                            batch = Vec::with_capacity(self.batch_size);
                        }
                    }
                    res = self.recv.recv() => {
                        match res {
                            Some(msg) => {
                                // push to batch
                                batch.push(msg);

                                // if batch was full
                                if batch.len() == self.batch_size {
                                    
                                    // handle batch
                                    let _ = self.proc.handle_batch(batch).await;

                                    // create batch empty
                                    batch = Vec::with_capacity(self.batch_size);

                                }
                            }
                            None => {
                                if batch.len() > 0 {

                                    // handle_batch
                                    if let Err(bt) = self.proc.handle_batch(batch).await {

                                        // drain
                                        self.proc.drain(bt.0).await;
                                    
                                    }

                                    // call terminate
                                    self.proc.terminate().await;

                                    return

                                } else {

                                    // call terminate
                                    self.proc.terminate().await;

                                    return
                                }
                            }
                        }
                    }
                }
            }

        });
    }


}
