use std::{time::Duration, sync::{Arc, atomic::{AtomicI32, Ordering}}};

use async_trait::async_trait;
use pulsar::{Producer, Pulsar};

pub use pulsar::{
    SerializeMessage,
    Error,
    ProducerOptions,
    producer::SendFuture
};


use crate::{ProcResult, Processor};


static GLOBAL_PULSAR_PROCESSOR_INSTANCE_COUNTER: Arc<AtomicI32> = Arc::new(AtomicI32::new(1));



pub struct PulsarProcessor {
    pulsar_producer: Producer<_>
}

impl PulsarProcessor {
    pub fn new(pulsar: Pulsar<_>, 
               opts: ProducerOptions,
               topic: &str, 
               pulsar_instance_name: &str,
            ) -> Result<Self, Error> {
        
        let new_id = get_new_id();


        let producer = pulsar
        .producer()
        .with_topic(topic)
        .with_name(format!("{}-{}", pulsar_instance_name, new_id))
        .with_options(opts)
        .build()
        .await?;


        Ok(PulsarProcessor {
            pulsar_producer: producer
        })
    }    
}




/// DeliveryResult returns a SendFuture or Error because the receipt can come long after this function 
/// was called, for various reasons:
/// 
/// 1.  the message was sent successfully but Pulsar did not send the receipt yet
/// 
/// 2.  the producer is batching messages, so this function must return immediately, 
///      and the receipt will come when the batched messages are actually sent

pub type DeliveryResult = Result<SendFuture, Error>;


#[async_trait]
impl Processor<Input, DeliveryResult> for PulsarProcessor 
where
    Input: Send + SerializeMessage
{
    
    async fn init(&mut self) { }

    async fn terminate(&mut self) { }

    async fn handle_message(&mut self, msg: Input) -> ProcResult<DeliveryResult> {

        let result = 
            self.pulsar_producer
                .send(msg)
                .await;

    
        ProcResult::Dispatch(delivery, None)
    }
    
}




fn get_new_id() -> i32 {
    GLOBAL_PULSAR_PROCESSOR_INSTANCE_COUNTER
        .fetch_add(1, Ordering::SeqCst)
}