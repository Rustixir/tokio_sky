
use blueriver::{run_topology_1_with_batcher, batcher::BatchProcessor, BATCH_TIMEOUT};


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


    let batcher_factory = || Batcher;
    let batcher_concurrency = 2;
    let batcher_buffer_size = 10;
    let batcher_batch_size = 10;
    let batcher_batch_timeout: BATCH_TIMEOUT;
    
    

    //                                      ---> batcher[category_id]
    //                                    /       
    //              /     processor-1    /
    //  producer-1 /                    -----> batcher[category_id] 
    //  producer-2 ----   processor-2   \
    //  producer-3 \                     \ 
    //              \     processor-3      -----> batcher[category_id]

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




enum Category {
    Cars,
    Mobiles,
    Accessories
}

struct Product {
    ctype: Category   
}


struct Prod;
#[async_trait]
impl Producer<Product> for Prod {

    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<Product>) {}


    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<Product>, Terminate> {
        
        Ok((0..buffer_size)
            .into_iter()
            .map(|i| {

                let ctype = match i {
                    1 | 2  => Category::Cars,
                    3 | 4  => Category::Mobiles,
                    5 => Category::Accessories 
                };


                Product {
                    ctype
                }

            })
            .collect::<VecDeque<Product>>())
    }
} 


struct Layer1Process;
#[async_trait]
impl Processor<Product, Product> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: Product) ->  ProcResult<Product> {

        // Parition_key
        let pk = match msg.utype {
            UserType::Admin => "admin".to_owned(),
            UserType::Client => "client".to_owned(),
        };

        ProcResult::Dispatch(msg,  Some(pk))
    } 
}


struct Batcher;
impl Batcher {
    pub fn batch_cars_insert(&self, batch: Vec<Product>) {
        ()
    }
    pub fn batch_mobiles_insert(&self, msg: Product) {
        ()
    }
    pub fn batch_accessories_insert(&self, msg: Product) {
        ()
    }
}
#[async_trait]
impl BatchProcessor<Product, ()> for Batcher {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn drain(&mut self, batch: Vec<Product>);

    async fn handle_batch(&mut self, batch: Vec<Product>) -> Result<(), BatcherTerminate<Input>> {
        
        // we just check first product
        // because we now all others same for current instance
        
        let ct = batch[0].ctype;

        match ct {
            Category::Cars => {
                self.batch_cars_insert(batch);
            }
            Category::Mobiles => {
                self.batch_mobiles_insert(batch);
            }
            Category::Accessories => {
                self.batch_accessories_insert(batch);
            }
        }


        ProcResult::Continue
    } 
}



