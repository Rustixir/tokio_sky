
#[tokio::main]
async fn main() {

    let (colletor_sender, collector_recv) = channel::<i32>(500);


    // fill buffer until buffer become full or timeout happen
    let timeout = Duration::from_secs(1);

    let producer_factory = || Collector::new(collector_recv, timeout);
    let producer_concurrency = 1;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;



    let proc_factory = || Layer1Process;
    let proc_concurrency = 3;
    let proc_buffer_size = 10;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  

    //                                 processor-1
    //                               /       
    //  producer-1 \                /
    //  producer-2 -> Collector ->  -----  processor-2
    //  producer-x /                \
    //                               \  
    //                                 processor-x
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


    
    // Burn Collector
    for elem in 0..500 {
        let _ = colletor_sender.send(elem).await;
    }
    
    

        
    // Safe Shutdown from (Producer) to (Layer_X_Processor)
    safe_shutdown.send(());

}




struct Layer1Process;
#[async_trait]
impl Processor<i32, ()> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: i32) ->  ProcResult<()> {
        
        // print
        println!("==> {}", msg);

        // return
        ProcResult::Continue
    } 
}



