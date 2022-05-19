use pulsar::ProducerOptions;
use tokio_sky::builtin::pulsar_processor::PulsarProcessor;


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



    // Pulsar Config 
    let addr = "pulsar://127.0.0.1:6650";
    let topic   = "non-persistent://public/default/topic1";
    let pulsar_instance_name = "pulsar_batch_processor"; 

    let pulsar: Pulsar<_> = Pulsar::builder(addr, TokioExecutor).build().await.unwrap();
    let opts = ProducerOptions {
        schema: Some(proto::Schema {
            r#type: proto::schema::Type::String as i32,
            ..Default::default()
        }),
        ..Default::default()
    };


    let batcher_factory =  || PulsarProcessor::new(pulsar, opts, topic, pulsar_instance_name).unwrap();
    let batcher_concurrency = 3;
    let batcher_buffer_size = 10;
    let batcher_batch_size = 10;
    let batcher_batch_timeout: BATCH_TIMEOUT;
    
    //                                      ---> pulsar_batcher
    //                                    /       
    //              /     processor-1    /
    //  producer-1 /                    -----> pulsar_batcher 
    //  producer-2 ----   processor-2   \
    //  producer-3 \                     \ 
    //              \     processor-3      -----> pulsar_batcher

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





#[derive(Serialize, Deserialize)]
struct Cat {
    age: u8    
}

impl SerializeMessage for Cat {
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
impl Producer<Cat> for Prod {

    async fn init(&mut self) {}

    async fn terminate(&mut self) {}

    async fn drain(&mut self, _buffer: VecDeque<Cat>) {}

    async fn fill_buffer(&mut self, buffer_size: usize) -> Result<VecDeque<Cat>, Terminate> {

        Ok((0..buffer_size)
            .into_iter()
            .map(|i| {
                Cat {
                    age: buffer_size % 10
                }
            })
            .collect::<VecDeque<Cat>>())
    }
} 




struct Layer1Process;
#[async_trait]
impl Processor<Cat, Cat> for Layer1Process {
    async fn init(&mut self) {}
    async fn terminate(&mut self) {}

    async fn handle_message(&mut self, msg: Cat) ->  ProcResult<Cat> {

        // Dispatch to Batcher
        ProcResult::Dispatch(msg,  None)
    } 
}
