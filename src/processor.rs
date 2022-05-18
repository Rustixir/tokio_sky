
use crate::dispatcher::Dispatcher;
use async_trait::async_trait;
use tokio::sync::mpsc;



#[async_trait]
pub trait Processor<Input, Output> {
    
    async fn init(&mut self);
    
    /// if return ProcResult::Dispatch
    ///    and Dispatcher is Partition must set batch_key
    async fn handle_message(&mut self, msg: Input) -> ProcResult<Output>;

    async fn terminate(&mut self);
}



type BatchKey = String;

pub enum ProcResult<Output> {
    
    /// skip this Batch
    Continue,

    /// dispatch by dispatcher
    /// if dispatcher mode is not partition, pass None 
    Dispatch(Output, Option<BatchKey>)
}


// ------------------------------------------------------



pub struct Context<Input, Output, Proc>
where
    Input: Send + 'static,
    Output: Send + 'static,
    Proc: Processor<Input, Output> + Send + 'static
{
    recv: mpsc::Receiver<Input>,
    dispatcher: Dispatcher<Output>,
    proc: Proc
}
impl<Input, Output, Proc> Context<Input, Output, Proc> 
where
    Input  : Clone + Send + 'static,
    Output : Clone + Send + 'static,
    Proc   : Processor<Input, Output> + Send + 'static
{
    
    pub fn new(recv: mpsc::Receiver<Input>,
               dispatcher: Dispatcher<Output>,
               proc: Proc) -> Self 
    {
        Context { 
            recv, 
            dispatcher, 
            proc,  
        }
    }


    #[inline]
    pub fn run(mut self) {        
        // spawn
        tokio::spawn(async move {

            self.proc.init().await;
            
            while let Some(msg) = self.recv.recv().await {
                match self.proc.handle_message(msg).await {
                    ProcResult::Continue => (),
                    ProcResult::Dispatch(m, pk) => {

                        let _ = self.dispatcher.dispatch(m, pk).await;

                    }
                }
            }
        });
    }
}
