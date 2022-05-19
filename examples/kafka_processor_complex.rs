use tokio_sky::builtin::kafka_processor::KafkaProcessor;


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


    let kafka_proc_factory = || KafkaProcessor::new(brokers, topic_name, message_timeout_ms);
    let kafka_proc_concurrency = 1;
    let kafka_proc_router = RouterType::RoundRobin;
    let kafka_proc_buffer_size = 100;


    let result_handler_proc_factory = || DeliveryHandler;
    let result_handler_proc_concurrency = 1;
    let result_handler_proc_buffer_size = 10;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  


    //              / \     
    //  producer-1 /   \
    //  producer-2 ----- >  Kafka_processor -> processor
    //  producer-3 \   /
    //              \ / 
    //               

    let safe_shutdown = 
                run_topology_2(
                   producer_factory,
                   producer_concurrency,
                   producer_router,
                   producer_buffer_pool,
                
                   kafka_proc_factory,
                   kafka_proc_concurrency,
                   kafka_proc_router,
                   kafka_proc_buffer_size,

                   result_handler_proc_factory,
                   result_handler_proc_concurrency,
                   result_handler_proc_buffer_size
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





struct DeliveryHandler;
#[async_trait]
impl Processor<OwnedDeliveryResult, ()> for Layer2Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: OwnedDeliveryResult) ->  ProcResult<()> {
        
        match msg {
            Some((partition, offset)) => {

            }
            Err((kafka_err, msg)) => {

            }
        }

        ProcResult::Continue
    } 
}




