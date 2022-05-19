use pulsar::Error;
use tokio_sky::builtin::kafka_processor::KafkaProcessor;


#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    // Pulsar Config 
    let addr = "pulsar://127.0.0.1:6650";
    let topic   = "non-persistent://public/default/topic1";
    let pulsar_instance_name = "pulsar_processor"; 

    let pulsar: Pulsar<_> = Pulsar::builder(addr, TokioExecutor).build().await.unwrap();
    let opts = producer::ProducerOptions {
        schema: Some(proto::Schema {
            r#type: proto::schema::Type::String as i32,
            ..Default::default()
        }),
        ..Default::default()
    };


    let pulsar_proc_factory = || || PulsarProcessor::new(pulsar, opts, topic, pulsar_instance_name).unwrap();
    let pulsar_proc_concurrency = 1;
    let pulsar_proc_router = RouterType::RoundRobin;
    let pulsar_proc_buffer_size = 100;


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
    //  producer-2 ----- >  Pulsar_processor -> processor
    //  producer-3 \   /
    //              \ / 
    //               

    let safe_shutdown = 
                run_topology_2(
                   producer_factory,
                   producer_concurrency,
                   producer_router,
                   producer_buffer_pool,
                
                   pulsar_proc_factory,
                   pulsar_proc_concurrency,
                   pulsar_proc_router,
                   pulsar_proc_buffer_size,

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
impl Processor<DeliveryResult, ()> for Layer2Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: DeliveryResult) ->  ProcResult<()> {
        
        match msg {
            Some(send_future) => {
                
            }
            Err(pulsar_err) => {

            }
        }

        ProcResult::Continue
    } 
}




