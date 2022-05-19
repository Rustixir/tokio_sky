use std::time::Duration;

use async_trait::async_trait;
use rdkafka::{producer::{FutureProducer, FutureRecord}, ClientConfig, error::KafkaError, util::Timeout};

use crate::{ProcResult, Processor};


#[derive(Clone)]
pub struct ProcKafkaMessage {
    pub key       : Vec<u8>,
    pub payload   : String,
    pub topic     : String,
    pub partition : i32,
}

pub struct KafkaProcessor {
    fut_producer: FutureProducer,
    topic_name: String
}

impl KafkaProcessor {
    pub fn new(brokers: &str, topic_name: &str, message_timeout_ms: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", message_timeout_ms)
            .create()
            .expect("Producer creation error");

        KafkaProcessor {
            fut_producer: producer,
            topic_name: topic_name.to_owned()
        }
    }    
}


/// If message delivery was successful, `OwnedDeliveryResult` will return the
/// partition and offset of the message. If the message failed to be delivered
/// an error will be returned, together with an owned copy of the original
/// message.
pub type OwnedDeliveryResult = Result<(Partition, Offset), (KafkaError, ProcKafkaMessage)>;

#[derive(Clone)]
pub struct Partition(i32);

#[derive(Clone)]
pub struct Offset(i64);


#[async_trait]
impl Processor<ProcKafkaMessage, OwnedDeliveryResult> for KafkaProcessor {
    
    async fn init(&mut self) { }

    async fn terminate(&mut self) { }

    async fn handle_message(&mut self, msg: ProcKafkaMessage) -> ProcResult<OwnedDeliveryResult> {

        let rec = 
            FutureRecord::to(&self.topic_name) 
                    .payload(&msg.payload)
                    .key(&msg.key)
                    .partition(msg.partition);
        

        let res = self.fut_producer.send(rec, Timeout::Never).await;

        let delivery: OwnedDeliveryResult = match res {
                Ok((p, o)) => {
                    Ok((Partition(p), Offset(o)))
                }   
                Err((ke, _)) => {
                    Err((ke, msg))
                }         
            };
        
        
        ProcResult::Dispatch(delivery, None)
    }
    
}

