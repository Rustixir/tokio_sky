
use std::collections::VecDeque;

use async_trait::async_trait;
use tokio::sync::oneshot;



use crate::{dispatcher::{Dispatcher, DispatchError}, RouterType};



pub struct Terminate;


#[async_trait]
pub trait Producer<T> {

    // Call inside self Process/Task
    async fn init(&mut self);

    // if have not enough message 'buffer_size', no problem 
    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<T>, Terminate>;


    // when got shutdown signal and alreadt buffer is not empty call this
    async fn drain(&mut self, buffer: VecDeque<T>);


    // Call before shutdown
    async fn terminate(&mut self);



}



// ------------------------------------------------------


pub struct Context<T, Prod> 
where
    Prod: Producer<T>
{
    dispatcher: Dispatcher<T>,
    producer: Prod,
    buffer_size: usize,
    shutdown: oneshot::Receiver<()>
}
impl<T, Prod> Context<T, Prod> 
where
    T    : Clone + Send + 'static,
    Prod : Producer<T> + Send + 'static
{
    
    pub fn new(dispatcher: Dispatcher<T>,
               producer: Prod,
               buffer_size: usize,
               shutdown: oneshot::Receiver<()>) -> Result<Self, ()> {        


        // Producer dispatcher cannot be partition mode 
        if let RouterType::Partition = dispatcher.router_type  {
            return Err(())
        }
                

        Ok(Context {
            dispatcher,
            producer,
            buffer_size,
            shutdown
        })
    }

    #[inline]
    pub fn run(mut self) {
        // spawn
        tokio::spawn(async move {

            let mut buffer = VecDeque::new();

            // outer loop
            loop {


                if let Ok(_) = self.shutdown.try_recv() {
                    return
                }


                // if buffer was empty , fill buffer
                if buffer.len() == 0 {

                    match self.producer.fill_buffer(self.buffer_size).await {
                        Ok(buff) => {
                            buffer = buff;
                        }
                        Err(_) => {
                            self.producer.terminate().await;
                            return
                        }
                    }
                    
                }                
                

                // loop Dispatch until exist, at least one channel
                loop {
                    match buffer.pop_front() {
                        Some(b) => {
                            if let Err(DispatchError::NotExist(b)) = self.dispatcher.dispatch(b, None).await {
                        
                                // back to buffer because not exist any channel
                                buffer.push_front(b);
        
        
                                // drain
                                self.producer.drain(buffer).await;
        
                                // terminate
                                self.producer.terminate().await;
        
                                return
        
                                // DispatchError::NotFound just used by partition
                                //   also producer.dispatcher cannot be partition mode 
                            }
                        }
                        None => {
                            break;
                        }
                    }
                }


                
            }
        });
    }


}



