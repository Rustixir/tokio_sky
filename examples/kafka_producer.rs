


#[tokio::main]
async fn main() {

    // Kafka Config 
    let brokers = "localhost:9092";
    let group_id = "example_consumer_group_id";
    let topics = &["topic1", "topic2", "topic3"];
    let enable_partition_eof = false;
    let session_timeout_ms = "6000";
    let auto_offset_reset = "smallest";


    let producer_factory = 
        move || KafkaProducer::new(brokers, 
                                   group_id, 
                                   topics, 
                                   enable_partition_eof, 
                                   session_timeout_ms, 
                                   auto_offset_reset,
                                   ProcessingType::Batch);
    
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;



    let proc_factory = || Layer1Process;
    let proc_concurrency = 3;
    let proc_buffer_size = 10;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X kafka_producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  


    //                    /     processor-1 
    //  kafka_producer-1 /
    //  kafka_producer-2 ----   processor-2
    //  kafka_producer-3 \
    //                    \     processor-3
    //               

    let safe_shutdown = 
                run_topology_1(
                   producer_factory,
                   producer_concurrency,
                   producer_router,
                   producer_buffer_pool,
                
                   proc_factory,
                   proc_concurrency,
                   proc_buffer_size,
                );


    // Safe Shutdown from (Producer) to (Layer_X_Processor)
    let _ = safe_shutdown.send(());

}




struct Layer1Process;
#[async_trait]
impl Processor<ProdKafkaMessage, ()> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: ProdKafkaMessage) ->  ProcResult<()> {
        
        let key = match msg.key {
            Some(v) => {
                String::from_utf8(v).unwrap()
            }
            None => {
                "".to_owned()
            }
        };

        // print
        println!("==> {}-{}-{}-{}", key, msg.payload, msg.topic, msg.partition);

        // return
        ProcResult::Continue
    } 
}



