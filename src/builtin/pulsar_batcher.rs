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



pub struct PulsarBatchProcessor {
    pulsar_producer: Producer<_>
}

impl PulsarBatchProcessor {
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


        Ok(PulsarBatchProcessor {
            pulsar_producer: producer
        })
    }    
}



#[async_trait]
impl BatchProcessor<Input> for PulsarBatchProcessor 
where
    Input: Send + SerializeMessage
{
    
    async fn init(&mut self) { }

    async fn terminate(&mut self) { }

    async fn drain(&mut self, batch: Vec<Input>) {}


    async fn handle_batch(&mut self, batch: Vec<Input>) -> ProcResult<DeliveryResult> {

        let result = 
            self.pulsar_producer
                .send_all(batch)
                .await;
    
        ProcResult::Dispatch(delivery, None)
    }
    
}




fn get_new_id() -> i32 {
    GLOBAL_PULSAR_PROCESSOR_INSTANCE_COUNTER
        .fetch_add(1, Ordering::SeqCst)
}