use tokio_sky::BatchProcessor;


#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    let proc_factory = || Layer1Process;
    let proc_concurrency = 3;
    let proc_router = RouterType::Partition;
    let proc_buffer_size = 10;


    // Mysql Config
    let database_url = "...";
    let pool = mysql_async::Pool::new(database_url);


    let batcher_factory =  || MysqlBatcher::new(pool);
    let batcher_concurrency = 2;
    let batcher_buffer_size = 10;
    let batcher_batch_size = 10;
    let batcher_batch_timeout: BATCH_TIMEOUT;
    
    //               /    processor-1  \                                    
    //              /                   \
    //  producer-1 /                     -----> MysqlBatcher[fullname] 
    //  producer-2 ----   processor-2    
    //  producer-3 \                     -----> MysqlBatcher[fullname]
    //              \                    /
    //               \    processor-3   / 


    let safe_shutdown = 
                run_topology_1_with_batcher(
                   producer_factory,
                   producer_concurrency,
                   producer_router,
                   producer_buffer_pool,
                
                   proc_factory,
                   proc_concurrency,
                   proc_router,
                   proc_buffer_size,

                   batcher_factory,
                   batcher_concurrency,
                   batcher_buffer_size,
                   batcher_batch_size,
                   batcher_batch_timeout
                );


    // Safe Shutdown from (Producer) to (Layer_X_Processor)
    safe_shutdown.send(());


}








struct Prod;
#[async_trait]
impl Producer<User> for Prod {

    async fn init(&mut self) {}

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<User>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<User>, Terminate> {

        Ok((0..buffer_size)
            .into_iter()
            .map(|i| {
                User {
                    age: buffer_size % 14,
                    fullname:  format!("fname_lname")
                }
            })
            .collect::<VecDeque<User>>())
    }
} 




struct Layer1Process;
#[async_trait]
impl Processor<User, User> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: User) ->  ProcResult<User> {

        // ordering - partitioning based on 'fullname'
        let pk = msg.fullname.clone();

        // Dispatch to Batcher
        ProcResult::Dispatch(msg, pk)
    } 
}



struct MysqlBatcher {
    pool: Pool
}
 
impl MysqlBatcher {
    pub fn new(pool: Pool) -> MysqlBatcher {
        MysqlBatcher { pool }
    }
}

#[async_trait]
impl BatchProcessor<User> for MysqlBatcher {
    
    async fn init(&mut self) { }

    async fn terminate(&mut self) { }

    async fn drain(&mut self, batch: Vec<User>) { }

    
    async fn handle_batch(&mut self, batch: Vec<User>) -> Result<(), BatcherTerminate<User>> {
        
        let conn = pool.get_conn().await.unwrap();

        r"INSERT INTO user (age, fullname)
          VALUES (:age, :fullname)"
            .with(batch.iter().map(|user| params! {
                "age" => user.customer_id,
                "fullname" => user.fullname.as_ref()
            }))
            .batch(&mut conn)
            .await?;
    }

}



struct User {
    age: u8,
    fullname: Strnig
}