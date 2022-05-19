use tokio_sky::builtin::pulsar_processor::PulsarProcessor;


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


    let proc_factory =  
        || PulsarProcessor::new(pulsar, opts, topic, pulsar_instance_name).unwrap();

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
    //  producer-2 ----- >   Pulsar_processor
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


#[derive(Serialize, Deserialize)]
struct TestData {
    data: String,
}

impl SerializeMessage for TestData {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}



struct Prod;
#[async_trait]
impl Producer<TestData> for Prod {

    async fn init(&mut self) {}

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<TestData>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<TestData>, Terminate> {

        Ok((0..buffer_size)
            .into_iter()
            .map(|i| {
                TestData {
                    data: format!("Pulsar !!"),
                }
            })
            .collect::<VecDeque<TestData>>())
    }
} 



