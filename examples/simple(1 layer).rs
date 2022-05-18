

#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
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


    //              /     processor-1 
    //  producer-1 /
    //  producer-2 ----   processor-2
    //  producer-3 \
    //              \     processor-3
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
impl Producer<usize> for Prod {

    async fn init(&mut self) {}

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<usize>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<usize>, Terminate> {

        Ok((0..buffer_size)
            .into_iter()
            .map(|i| i)
            .collect::<VecDeque<usize>>())
    }
} 


struct Layer1Process;
#[async_trait]
impl Processor<usize, ()> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: usize) ->  ProcResult<()> {
        
        // print
        println!("==> {}", msg);

        // return
        ProcResult::Continue
    } 
}



