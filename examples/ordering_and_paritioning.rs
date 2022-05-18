use blueriver::run_topology_1_with_batcher;




#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    let proc1_factory = || Layer1Process;
    let proc1_concurrency = 3;
    let proc1_router = RouterType::Partition;
    let proc1_buffer_size = 10;


    let proc2_factory = || Layer2Process;
    let proc2_concurrency = 2;
    let proc2_buffer_size = 10;

    

    // ***********************************
    // *
    // *   for ordering/paritioning 
    // *    
    // *   it's cost efficient to have equal number of concurrency with partition key 
    // *     if set more concurrency than partition_key
    // *         others instances is inactive whole time
    // *
    // *
    // *
    // ***********************************



    // 1. create X processor instances by 'proc_concurrency'
    //
    // 2. create X producer instances  by 'producer_concurrency'
    // 
    // 3. create topology and syncing
    //  

  
    //              /     processor-1  \
    //  producer-1 /                    -----> processor[admin_type] 
    //  producer-2 ----   processor-2  
    //  producer-3 \                    -----> processor[client_type]
    //              \     processor-3  /
    //               


    let safe_shutdown = 
                run_topology_1_with_batcher(
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




enum UserType {
    Admin,
    Client
}

struct User {
    utype: UserType,
    resource_lock: String   
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
                    utype: if i % 2 == 0 { UserType::Admin } else { UserType::Client },
                    resource_lock: i
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

        // Parition_key
        let pk = match msg.utype {
            UserType::Admin => "admin".to_owned(),
            UserType::Client => "client".to_owned(),
        };

        ProcResult::Dispatch(msg,  Some(pk))
    } 
}


struct Layer2Process;
impl Layer2Process {
    pub fn admin_handle(&self, msg: User) {
        println!("==> Alyaws get admin request{}", msg);
    }
    pub fn client_handle(&self, msg: User) {
        println!("==> Alyaws get client request{}", msg);
    }
}
#[async_trait]
impl Processor<User, ()> for Layer2Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: User) ->  ProcResult<()> {
        
        match msg.utype {
            UserType::Admin => {
                // Guarantee all Admin tied to a given 'UserType' are processed in order and not concurrently
                // by this instance
                self.admin_handle(msg);
            }
            UserType::Client => {
                // Guarantee all Client tied to a given 'UserType' are processed in order and not concurrently
                // by this instance
                self.client_handle(msg);
            }
        }

        ProcResult::Continue
    } 
}



