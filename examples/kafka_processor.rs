
#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    // Kafka Config 
    let brokers = "localhost:9092";
    let topic_name = "topic1";
    let message_timeout_ms = "5000";

    let proc_factory = 
        || KafkaProcessor::new(brokers, 
                               topic_name, 
                               message_timeout_ms);

    let proc_concurrency = 1;
    let proc_buffer_size = 100;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  


    //              / \     
    //  producer-1 /   \
    //  producer-2 ----- >   Kafka_processor
    //  producer-3 \   /
    //              \ / 
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
    safe_shutdown.send(());

}



struct Prod;
#[async_trait]
impl Producer<ProcKafkaMessage> for Prod {

    async fn init(&mut self) {}

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<ProcKafkaMessage>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<ProcKafkaMessage>, Terminate> {

        Ok((0..buffer_size)
            .into_iter()
            .map(|i| {
                ProcKafkaMessage {
                    key: format!("{}", i).into_bytes(),
                    payload: format!("payload - {}", i),
                    topic: format!("topic1"),
                    partition: 1,
                }
            })
            .collect::<VecDeque<ProcKafkaMessage>>())
    }
} 



