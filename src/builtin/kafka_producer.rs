use std::{collections::VecDeque, time::Duration};

use async_trait::async_trait;
use rdkafka::{ClientConfig, config::RDKafkaLogLevel, ClientContext, consumer::{ConsumerContext, Rebalance, StreamConsumer, Consumer, CommitMode}, error::KafkaResult, TopicPartitionList, Message};

use crate::{Producer, producer::Terminate};


// A type alias with your custom consumer can be created for convenience.
type LoggingConsumer = StreamConsumer<CustomContext>;


#[derive(Clone)]
pub struct ProdKafkaMessage {
    pub key       : Option<Vec<u8>>,
    pub payload   : String,
    pub topic     : String,
    pub partition : i32,
}





struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        println!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        println!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        println!("Committing offsets: {:?}", result);
    }
}


pub struct KafkaProducer {
    kafka_consumer: LoggingConsumer
}

impl KafkaProducer {
    pub fn new(brokers: &str, 
               group_id: &str, 
               topics: &[&str],
               enable_partition_eof: bool,
               session_timeout_ms: &str,
               auto_offset_reset: &str) -> Self {
        
        let context = CustomContext;

        let consumer: LoggingConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", format!("{}",enable_partition_eof))
            .set("session.timeout.ms", session_timeout_ms)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", auto_offset_reset)
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");
    
        consumer
            .subscribe(&topics.to_vec())
            .expect("Can't subscribe to specified topics");
    
        KafkaProducer {
            kafka_consumer: consumer
        }
    }
}

#[async_trait]
impl Producer<ProdKafkaMessage> for KafkaProducer {
   
    async fn init(&mut self) { }

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<ProdKafkaMessage>) { }

    

    async fn fill_buffer(&mut self, buffer_size:usize) ->  Result<VecDeque<ProdKafkaMessage>, Terminate> {

        // create buffer
        let mut buffer = VecDeque::with_capacity(buffer_size);


        // if cannot recv after from
        let sleep = tokio::time::sleep(Duration::from_millis(50));
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
                res = self.kafka_consumer.recv() => {
                    match res {
                        Err(e) => eprintln!("Kafka error: {}", e),
                        Ok(m) => {
                            let payload = match m.payload_view::<str>() {
                                None => "".to_owned(),
                                Some(Ok(s)) => s.to_owned(),
                                Some(Err(e)) => {
                                    eprintln!("Error while deserializing message payload: {:?}", e);                                    
                                    "".to_owned()
                                }
                            };       
                            
                            // if was not empty convert to vec
                            let key = match m.key() {
                                Some(bytes) => {
                                    Some(bytes.to_vec())
                                }
                                None => {
                                    None
                                }
                            };

                            // create ProdKafkaMessage
                            let kmsg = ProdKafkaMessage { 
                                key, 
                                payload, 
                                topic: m.topic().to_owned(), 
                                partition: m.partition()
                            };
                            
                            // push to buffer
                            buffer.push_back(kmsg);

                            // if buffer full
                            if buffer.len() == buffer_size {
                                return Ok(buffer)
                            }

            
                            self.kafka_consumer.commit_message(&m, CommitMode::Async).unwrap();
                        }
                    }
                }
            }
        }
    }

}

