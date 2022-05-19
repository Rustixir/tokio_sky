

use std::time::Duration;

use crate::batcher::{BatchProcessor, self};
use crate::{shutdown_manager::start_shutdown_manager};
use crate::processor::Processor;
use indexmap::IndexMap;
use tokio::sync::{mpsc::{channel, self}, oneshot};

use crate::{producer::{Producer, self}, dispatcher::{Dispatcher, RouterType}, processor};


pub const PRODUCER_FILLBUFFER_TIMEOUT_REALTIME: Duration = Duration::from_millis(2);
pub const PRODUCER_FILLBUFFER_TIMEOUT_BATCH: Duration = Duration::from_millis(50);

pub const BATCH_SIZE: usize = 100;
pub const BATCH_TIMEOUT: Duration = Duration::from_millis(50);
pub const CONCURRENCY: i32 = 1;
pub const BUFFER_SIZE: usize = 10;
pub const BUFFER_POOL_SIZE: usize = 100;


/// Realtime, timeout is 1 milliseconds
/// Batch,    timeout is 50 milliseconds
/// 
/// ## Batch
///     producer await until buffer full or timeout
///     in Batch timeout is 50 milliseconds increase latency 
///     if need fast serving data this is not best way
/// 
/// ## RealTime
///     producer await until buffer full or timeout
///     in RealTime timeout is 1 milliseconds decrease latency
///     but may increase cpu usage 
/// 
/// ## CustomTimeout
///     if want set timeout custom  
pub enum ProcessingType {
    RealTime,
    Batch,
    CustomTimeout(Duration)
}




fn start_producer<T, Prod, F> (producer_factory: F,
                               mut concurrency: i32,
                               router: RouterType, 
                               proc_channels: IndexMap<String, mpsc::Sender<T>>, 
                               mut buffer_pool_size: usize,
                               shutdown: oneshot::Receiver<()>)
where
    T: Clone + Send + 'static,
    F: Fn() -> Prod + Send + 'static,
    Prod: Producer<T> + Send + 'static
{
    if concurrency <= 0 {
        concurrency = CONCURRENCY;
    }

    if buffer_pool_size <= 0 {
        buffer_pool_size = BUFFER_POOL_SIZE;
    }

    if proc_channels.len() <= 0 {
        panic!("==> Producer must at-least have 1_Processor_layer")
    }

 


    let mut list_shutdown = vec![];

    for _elem in 0..concurrency {
    
        
        let (sx, rx) = oneshot::channel();

        let dispatcher = Dispatcher::new(proc_channels.clone(), router).unwrap();

        producer::Context::new(dispatcher, 
                               producer_factory(), 
                               buffer_pool_size,
                               rx).unwrap().run();

        list_shutdown.push(sx)
    }


    start_shutdown_manager(shutdown, list_shutdown)
}




pub struct Config<Output> {
    proc_channels : IndexMap<String, mpsc::Sender<Output>>,
    router_type: RouterType
}


fn start_processor<Input, Output, Proc, F> (processor_factory: F,
                                            mut concurrency: i32,
                                            mut buffer_size: usize,
                                            cfg: Option<Config<Output>>) -> IndexMap<String,mpsc::Sender<Input>> 
where
    Input  : Clone + Send + 'static,
    Output : Clone + Send + 'static,
    F      : Fn() -> Proc + Send + 'static,
    Proc   : Processor<Input, Output> + Send + 'static
{
    if concurrency <= 0 {
        concurrency = CONCURRENCY;
    }


    if buffer_size <= 0 {
        buffer_size = BUFFER_SIZE;
    }
    


    let mut list = IndexMap::with_capacity(concurrency as usize);

    for elem in 0..concurrency {
    
        let (sender, recv) = channel(buffer_size);
        list.insert(format!("{}", elem), sender);

        let dispatcher = match cfg {
            Some(ref c) => {
                Dispatcher::new(c.proc_channels.clone(), c.router_type).unwrap()
            }
            None => {
                Dispatcher::new(IndexMap::new(), RouterType::RoundRobin).unwrap()
            }
        };
    
        processor::Context::<Input, Output, Proc>::new(recv, dispatcher, processor_factory()).run();

    }

    return list
}





fn start_batch_processor<Input, Proc, F> (batcher_factory: F,
                                          mut concurrency: i32,
                                          mut buffer_size: usize,
                                          mut batch_size: usize,
                                          mut batch_timeout: Duration
                                        ) -> IndexMap<String,mpsc::Sender<Input>> 
where
    Input  : Clone + Send + 'static,
    F      : Fn() -> Proc + Send + 'static,
    Proc   : BatchProcessor<Input> + Send + 'static
{

    if concurrency <= 0 {
        concurrency = CONCURRENCY;
    }

    if buffer_size <= 0 {
        buffer_size = BUFFER_SIZE;
    }

    if batch_size <= 0 {
        batch_size = BATCH_SIZE;
    }

    if batch_timeout <= Duration::ZERO {
        batch_timeout = BATCH_TIMEOUT;
    }



    let mut list = IndexMap::with_capacity(concurrency as usize);

    for elem in 0..concurrency {
    
        let (sender, recv) = channel(buffer_size);
        list.insert(format!("{}", elem), sender);
    

        batcher::Context::<Input, Proc>::new(recv, 
                                             batcher_factory(),
                                             batch_size,
                                             batch_timeout).run();

    }

    return list
}









// ------------------------------------------------------------




/// producer -> processor
/// 
pub fn run_topology_1<Prod, ProdFactory, Input, Output, Proc, ProcFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     processor_factory: ProcFactory,
     proc_concurrency: i32,
     proc_buffer_size: usize) -> oneshot::Sender<()>

where
    Prod: Producer<Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,

    Input: Clone + Send + 'static,
    Output: Clone + Send + 'static,

    Proc: Processor<Input, Output> + Send + 'static,
    ProcFactory: Fn() -> Proc + Send + 'static
{
    
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let proc_channels  = 
            start_processor(processor_factory, proc_concurrency, proc_buffer_size, None);
    

    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   proc_channels, 
                   buffer_pool_size, 
                   rx);

    sx
}





/// producer -> processor -> processor
/// 
pub fn run_topology_2<Prod, ProdFactory, 
                      Layer1Input, Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_buffer_size: usize
    

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static
{
    
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            None);
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);

    sx
}



/// producer -> processor -> processor -> processor
/// 
pub fn run_topology_3<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_buffer_size: usize,

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            None);
            

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}




/// producer -> processor -> processor -> processor -> processor
/// 
pub fn run_topology_4<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory,
                      Layer4Output, Layer4Proc, Layer4ProcFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_router: RouterType,
     layer3_buffer_size: usize,

     layer4_processor_factory: Layer4ProcFactory,
     layer4_proc_concurrency: i32,
     layer4_buffer_size: usize,

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer4Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static,

    Layer4Proc: Processor<Layer3Output, Layer4Output> + Send + 'static,
    Layer4ProcFactory: Fn() -> Layer4Proc + Send + 'static
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let layer4_proc_channels  = 
            start_processor(layer4_processor_factory, 
                            layer4_proc_concurrency, 
                            layer4_buffer_size, 
                            None);
    

    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            Some(Config { proc_channels: layer4_proc_channels, router_type: layer3_router }));
                            
    

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}





/// producer -> processor -> processor -> processor -> processor
/// 
pub fn run_topology_5<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory,
                      Layer4Output, Layer4Proc, Layer4ProcFactory,
                      Layer5Output, Layer5Proc, Layer5ProcFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_router: RouterType,
     layer3_buffer_size: usize,

     layer4_processor_factory: Layer4ProcFactory,
     layer4_proc_concurrency: i32,
     layer4_router: RouterType,
     layer4_buffer_size: usize,

     layer5_processor_factory: Layer5ProcFactory,
     layer5_proc_concurrency: i32,
     layer5_buffer_size: usize,

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer4Output: Clone + Send + 'static,

    Layer5Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static,

    Layer4Proc: Processor<Layer3Output, Layer4Output> + Send + 'static,
    Layer4ProcFactory: Fn() -> Layer4Proc + Send + 'static,

    Layer5Proc: Processor<Layer4Output, Layer5Output> + Send + 'static,
    Layer5ProcFactory: Fn() -> Layer5Proc + Send + 'static
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let layer5_proc_channels  = 
            start_processor(layer5_processor_factory, 
                            layer5_proc_concurrency, 
                            layer5_buffer_size, 
                            None);

    
    let layer4_proc_channels  = 
            start_processor(layer4_processor_factory, 
                            layer4_proc_concurrency, 
                            layer4_buffer_size, 
                            Some(Config { proc_channels: layer5_proc_channels, router_type: layer4_router }));
                            
        

    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            Some(Config { proc_channels: layer4_proc_channels, router_type: layer3_router }));
                            
    

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}



// -----------------------------------------------






/// producer -> processor -> batcher
/// 
pub fn run_topology_1_with_batcher<Prod, ProdFactory, Input, 
                                   Output, Proc, ProcFactory, 
                                   Batcher, BatcherFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     processor_factory: ProcFactory,
     proc_concurrency: i32,
     proc_router: RouterType,
     proc_buffer_size: usize,    

     batcher_factory: BatcherFactory,
     batcher_concurrency: i32,
     batcher_buffer_size: usize,
     batcher_batch_size: usize,
     batcher_batch_timeout: Duration,

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,

    Input: Clone + Send + 'static,
    Output: Clone + Send + 'static,

    Proc: Processor<Input, Output> + Send + 'static,
    ProcFactory: Fn() -> Proc + Send + 'static,

    Batcher: BatchProcessor<Output> + Send + 'static,
    BatcherFactory: Fn() -> Batcher + Send + 'static

{
    
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let batcher_channels = 
            start_batch_processor(batcher_factory, 
                                  batcher_concurrency, 
                                  batcher_buffer_size, 
                                  batcher_batch_size, 
                                  batcher_batch_timeout);
    

    let proc_channels  = 
            start_processor(processor_factory, 
                            proc_concurrency, 
                            proc_buffer_size, 
                            Some(Config{ proc_channels: batcher_channels, router_type: proc_router}));
    

    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   proc_channels, 
                   buffer_pool_size, 
                   rx);

    sx
}





/// producer -> processor -> processor -> batcher
/// 
pub fn run_topology_2_with_batcher<Prod, ProdFactory, 
                      Layer1Input, Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Batcher, BatcherFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,

     batcher_factory: BatcherFactory,
     batcher_concurrency: i32,
     batcher_buffer_size: usize,
     batcher_batch_size: usize,
     batcher_batch_timeout: Duration
    

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Batcher: BatchProcessor<Layer2Output> + Send + 'static,
    BatcherFactory: Fn() -> Batcher + Send + 'static
{
    
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let batcher_channels = 
            start_batch_processor(batcher_factory, 
                                  batcher_concurrency, 
                                  batcher_buffer_size, 
                                  batcher_batch_size, 
                                  batcher_batch_timeout); 


    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: batcher_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);

    sx
}



/// producer -> processor -> processor -> processor -> batcher
/// 
pub fn run_topology_3_with_batcher<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory,
                      Batcher, BatcherFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_router: RouterType,
     layer3_buffer_size: usize,

     batcher_factory: BatcherFactory,
     batcher_concurrency: i32,
     batcher_buffer_size: usize,
     batcher_batch_size: usize,
     batcher_batch_timeout: Duration

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static,


    Batcher: BatchProcessor<Layer3Output> + Send + 'static,
    BatcherFactory: Fn() -> Batcher + Send + 'static,
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let batcher_channels = 
            start_batch_processor(batcher_factory, 
                                  batcher_concurrency, 
                                  batcher_buffer_size, 
                                  batcher_batch_size, 
                                  batcher_batch_timeout);
    

    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            Some(Config { proc_channels: batcher_channels, router_type: layer3_router }));
            

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}




/// producer -> processor -> processor -> processor -> processor -> batcher
/// 
pub fn run_topology_4_with_batcher<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory,
                      Layer4Output, Layer4Proc, Layer4ProcFactory,
                      Batcher, BatcherFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_router: RouterType,
     layer3_buffer_size: usize,

     layer4_processor_factory: Layer4ProcFactory,
     layer4_proc_concurrency: i32,
     layer4_router: RouterType,
     layer4_buffer_size: usize,

     batcher_factory: BatcherFactory,
     batcher_concurrency: i32,
     batcher_buffer_size: usize,
     batcher_batch_size: usize,
     batcher_batch_timeout: Duration

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer4Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static,

    Layer4Proc: Processor<Layer3Output, Layer4Output> + Send + 'static,
    Layer4ProcFactory: Fn() -> Layer4Proc + Send + 'static,

    Batcher: BatchProcessor<Layer4Output> + Send + 'static,
    BatcherFactory: Fn() -> Batcher + Send + 'static
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let batcher_channels = 
            start_batch_processor(batcher_factory, 
                                  batcher_concurrency, 
                                  batcher_buffer_size, 
                                  batcher_batch_size, 
                                  batcher_batch_timeout);


    let layer4_proc_channels  = 
            start_processor(layer4_processor_factory, 
                            layer4_proc_concurrency, 
                            layer4_buffer_size, 
                            Some(Config { proc_channels: batcher_channels, router_type: layer4_router }));
    

    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            Some(Config { proc_channels: layer4_proc_channels, router_type: layer3_router }));
                            
    

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}





/// producer -> processor -> processor -> processor -> processor
/// 
pub fn run_topology_5_with_batcher<Prod, ProdFactory, Layer1Input, 
                      Layer1Output, Layer1Proc, Layer1ProcFactory,
                      Layer2Output, Layer2Proc, Layer2ProcFactory,
                      Layer3Output, Layer3Proc, Layer3ProcFactory,
                      Layer4Output, Layer4Proc, Layer4ProcFactory,
                      Layer5Output, Layer5Proc, Layer5ProcFactory,
                      Batcher, BatcherFactory>

    (producer_factory: ProdFactory,
     prod_concurrency: i32,
     router: RouterType,
     buffer_pool_size: usize,
 
     layer1_processor_factory: Layer1ProcFactory,
     layer1_proc_concurrency: i32,
     layer1_router: RouterType,
     layer1_buffer_size: usize,
    
     layer2_processor_factory: Layer2ProcFactory,
     layer2_proc_concurrency: i32,
     layer2_router: RouterType,
     layer2_buffer_size: usize,
    
     layer3_processor_factory: Layer3ProcFactory,
     layer3_proc_concurrency: i32,
     layer3_router: RouterType,
     layer3_buffer_size: usize,

     layer4_processor_factory: Layer4ProcFactory,
     layer4_proc_concurrency: i32,
     layer4_router: RouterType,
     layer4_buffer_size: usize,

     layer5_processor_factory: Layer5ProcFactory,
     layer5_proc_concurrency: i32,
     layer5_router: RouterType,
     layer5_buffer_size: usize,

     batcher_factory: BatcherFactory,
     batcher_concurrency: i32,
     batcher_buffer_size: usize,
     batcher_batch_size: usize,
     batcher_batch_timeout: Duration

    ) -> oneshot::Sender<()>

where
    Prod: Producer<Layer1Input> + Send + 'static,
    ProdFactory: Fn() -> Prod + Send + 'static,
    Layer1Input: Clone + Send + 'static,

    Layer1Output: Clone + Send + 'static,

    Layer2Output: Clone + Send + 'static,

    Layer3Output: Clone + Send + 'static,

    Layer4Output: Clone + Send + 'static,

    Layer5Output: Clone + Send + 'static,

    Layer1Proc: Processor<Layer1Input, Layer1Output> + Send + 'static,
    Layer1ProcFactory: Fn() -> Layer1Proc + Send + 'static,

    Layer2Proc: Processor<Layer1Output, Layer2Output> + Send + 'static,
    Layer2ProcFactory: Fn() -> Layer2Proc + Send + 'static,

    Layer3Proc: Processor<Layer2Output, Layer3Output> + Send + 'static,
    Layer3ProcFactory: Fn() -> Layer3Proc + Send + 'static,

    Layer4Proc: Processor<Layer3Output, Layer4Output> + Send + 'static,
    Layer4ProcFactory: Fn() -> Layer4Proc + Send + 'static,

    Layer5Proc: Processor<Layer4Output, Layer5Output> + Send + 'static,
    Layer5ProcFactory: Fn() -> Layer5Proc + Send + 'static,

    Batcher: BatchProcessor<Layer5Output> + Send +'static,
    BatcherFactory: Fn() -> Batcher + Send + 'static
{
   
    // Shutdown channel
    let (sx, rx) = oneshot::channel::<()>(); 


    let batcher_channels = 
            start_batch_processor(batcher_factory, 
                                  batcher_concurrency, 
                                  batcher_buffer_size, 
                                  batcher_batch_size, 
                                  batcher_batch_timeout);


    let layer5_proc_channels  = 
            start_processor(layer5_processor_factory, 
                            layer5_proc_concurrency, 
                            layer5_buffer_size, 
                            Some(Config { proc_channels: batcher_channels, router_type: layer5_router }));

    
    let layer4_proc_channels  = 
            start_processor(layer4_processor_factory, 
                            layer4_proc_concurrency, 
                            layer4_buffer_size, 
                            Some(Config { proc_channels: layer5_proc_channels, router_type: layer4_router }));
                            
        

    let layer3_proc_channels  = 
            start_processor(layer3_processor_factory, 
                            layer3_proc_concurrency, 
                            layer3_buffer_size, 
                            Some(Config { proc_channels: layer4_proc_channels, router_type: layer3_router }));
                            
    

    let layer2_proc_channels  = 
            start_processor(layer2_processor_factory, 
                            layer2_proc_concurrency, 
                            layer2_buffer_size, 
                            Some(Config { proc_channels: layer3_proc_channels, router_type: layer2_router }));
    

            
    let layer1_proc_channels  = 
            start_processor(layer1_processor_factory, 
                            layer1_proc_concurrency, 
                            layer1_buffer_size, 
                            Some(Config { proc_channels: layer2_proc_channels, router_type: layer1_router  }));
    


    start_producer(producer_factory, 
                   prod_concurrency, 
                   router, 
                   layer1_proc_channels, 
                   buffer_pool_size,
                   rx);
                
    sx
}


