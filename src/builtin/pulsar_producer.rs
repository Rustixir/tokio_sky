
use std::{sync::{Arc, atomic::{AtomicI32, Ordering}}, collections::VecDeque};

use async_trait::async_trait;

use futures::StreamExt;

use pulsar::{
    message::Payload, 
    Consumer
};

pub use pulsar::{
    Pulsar,
    TokioExecutor,
    DeserializeMessage,
    message::proto::command_subscribe::SubType,
    Error
};

use tokio::select;


static GLOBAL_PULASR_PRODUCER_INSTANCE_COUNTER: Arc<AtomicI32> = Arc::new(Atomic::new(1));


use crate::{topology:: {
    ProcessingType,
    PRODUCER_FILLBUFFER_TIMEOUT_BATCH,
    PRODUCER_FILLBUFFER_TIMEOUT_REALTIME
}, dispatcher::Dispatcher};

/// PulsarProducer when created 
/// need to unique name for each instance 
/// this module privade a Global Atomic Counter for PulsarProducer
/// 
struct PulsarProducer<Output> {
    pulsar_consumer: Consumer<Output, _>,
    tp: ProcessingType
}
impl<Output> PulsarProducer<Output> 
where
    Output: DeserializeMessage
{
    pub fn new(pulsar: Pulsar<_>,
               topics: &[&str],
               pulsar_instance_name: &str,
               subscription_type: SubType,
               buffer_size: u32, 
               tp: ProcessingType
               ) -> Result<Self, Error> 
    {
        let new_id = get_new_id();

        // Create new Consumer
        let mut consumer: Consumer<TestData, _> = pulsar
            .consumer()
            .with_topics(topics)
            .with_batch_size(batch_size)
            .with_consumer_name(format!("{}-{}", pulsar_instance_name, new_id))
            .with_subscription_type(subscription_type)
            .with_subscription(format!("{}-{}_subscription", pulsar_instance_name, new_id))
            .build()
            .await?;
        
        Ok(PulsarProducer {
            pulsar_consumer: consumer,
            tp
        })
    }
}

#[async_trait]
impl<Output> Producer<Output> for PulsarProducer<Output>
where
    Output: DeserializeMessage 
{

    async fn init(&mut self) {}
    
    async fn terminate(&mut self) {}

    async fn drain(&mut self, buffer: VecDeque<Output>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<Output>> {
        
        // create buffer
        let mut buffer = VecDeque::with_capacity(buffer_size);


        let sleep = match self.tp {
            ProcessingType::RealTime => PRODUCER_FILLBUFFER_TIMEOUT_REALTIME,
            ProcessingType::Batch    => PRODUCER_FILLBUFFER_TIMEOUT_BATCH,
            ProcessingType::CustomTimeout(d) => d
        };

        tokio::pin!(sleep);

        loop {

            select! {

                _ = &mut sleep => {
                    
                    // if buffer was not empty
                    if buffer.len() > 0 {

                        // return buffer
                        return Ok(buffer)
                    }
                }

                res = self.pulsar_consumer.next() => {
                    match res {
                        Some(msg) => {
                            self.pulsar_consumer.ack(&msg).await;
                            let data = match msg.deserialize() {
                                Ok(data) => {
                                    buffer.push_back(data);
                                    if buffer.len() == buffer_size {
                                        return Ok(buffer)
                                    }
                                }
                                Err(e) => {

                                    // TODO: this change to tracing or anothr logger in next vsn
                                    
                                    eprintln!("could not deserialize message: {:?}", e);
                                    
                                }
                            };
                        
                            if data.data.as_str() != "data" {
                                log::error!("Unexpected payload: {}", &data.data);
                                break;
                            }
                            
                            
                        }
                        None => {
                            // not exist any sender, shutdown
                            if buffer.len() > 0 {

                                // return buffer
                                return Ok(buffer)
                            } 

                            return Err(Terminate);

                        }

                    }
                }

            }

        }
        

    }
}





fn get_new_id() -> i32 {
    GLOBAL_PULASR_PRODUCER_INSTANCE_COUNTER
        .fetch_add(1, Ordering::SeqCst)
}