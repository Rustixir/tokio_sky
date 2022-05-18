


#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    let proc1_factory = || Layer1Process;
    let proc1_concurrency = 3;
    let proc1_router = RouterType::RoundRobin;
    let proc1_buffer_size = 10;


    let proc2_factory = || Layer2Process;
    let proc2_concurrency = 3;
    let proc2_buffer_size = 10;

    

    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  

    //                                         /   layer2_processor-1
    //              /     layer1_processor-1  / 
    //  producer-1 /                         /
    //  producer-2 ----   layer1_processor-2 ----- layer2_processor-2
    //  producer-x \                         \
    //              \     layer1_processor-x  \
    //                                         \   layer2_processor-x
    //               

    let safe_shutdown = 
                run_topology_2(
                   producer_factory,
                   producer_concurrency,
                   producer_router,
                   producer_buffer_pool,
                
                   proc1_factory,
                   proc1_concurrency,
                   proc1_router,
                   proc1_buffer_size,

                   proc2_factory,
                   proc2_concurrency,
                   proc2_buffer_size
                );

    
    // Safe Shutdown from (Producer) to (Layer_X_Processor)
    safe_shutdown.send(());


}



struct Prod;
#[async_trait]
impl Producer<usize> for Prod {

    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<usize>) {}


    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<usize>, Terminate> {
        
        Ok((0..buffer_size)
            .into_iter()
            .map(|i| i )
            .collect::<VecDeque<usize>>())
    }
} 


struct Layer1Process;
#[async_trait]
impl Processor<usize, String> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: usize) ->  ProcResult<String> {

        let new_msg = format!("msg-{}", i);

        ProcResult::Dispatch(new_batch,  None)
    } 
}


struct Layer2Process;
#[async_trait]
impl Processor<String, ()> for Layer2Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: String) ->  ProcResult<()> {
        
        println!("==> {}", msg);

        ProcResult::Continue
    } 
}



