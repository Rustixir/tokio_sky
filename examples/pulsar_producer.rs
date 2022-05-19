use pulsar::SubType;




#[tokio::main]
async fn main() {

    // Pulsar Config

    let addr = "pulsar://127.0.0.1:6650";
    let pulsar: Pulsar<_> = Pulsar::builder(addr, TokioExecutor).build().await.unwrap();


    let topics = ["topic1"];
    let pulsar_instance_name = "producer_name";
    let subscription_type = SubType::Shared;
    let buffer_size = 100;

    let producer_factory = 
        || PulsarProducer::<TestData>::new(pulsar, 
                                           topics, 
                                           pulsar_instance_name, 
                                           subscription_type, 
                                           buffer_size, 
                                           ProcessingType::Batch).unwrap();
    
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;



    let proc_factory = || Layer1Process;
    let proc_concurrency = 3;
    let proc_buffer_size = 10;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X pulsar_producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  


    //                     /     processor-1 
    //  pulsar_producer-1 /
    //  pulsar_producer-2 ----   processor-2
    //  pulsar_producer-3 \
    //                     \     processor-3
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



#[derive(Serialize, Deserialize)]
struct TestData {
    data: String,
}


struct Layer1Process;
#[async_trait]
impl Processor<TestData, ()> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: TestData) ->  ProcResult<()> {
        
        // print
        println!("==> {}", msg.data);

        // return
        ProcResult::Continue
    } 
}



