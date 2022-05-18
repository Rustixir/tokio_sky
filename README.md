

![BlueRiver](https://github.com/Rustixir/blueriver/blob/main/logo.jpeg)

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/blueriver">
    <img src="https://img.shields.io/crates/d/blueriver.svg?style=flat-square"
      alt="Download" />
  </a>
</div>


# BlueRiver

Build concurrent and multi-stage data ingestion and data processing pipelines with Rust+Tokio. 
BlueRiver allows developers to consume data efficiently from different sources, known as producers, such as Apache Kafka and others. BlueRiver pipelines are highly concurrent, robust
inspired by elixir broadway



## Features

BlueRiver takes the burden of defining concurrent GenStage topologies and provide a simple configuration API that automatically defines concurrent producers, concurrent processing, 
leading to both time and cost efficient ingestion and processing of data. It features:

  * **Producer** - is source of data pipline 

  * **Processor** - process message also can dispath to next stage by dispatcher 

  * **BatchProcessor** process group of message, that is used for last stage, have not next stage   

  * **Dispatcher** - dispatch message with three mode (RoundRobin, BroadCast, Partition)

  * **Customizable** - can use built-in Producer (like Apache Kafka) or write your own Producer 

  * **Batching** - BlueRiver provides built-in batching, allowing you to group messages 
        either by size and/or by time. This is important in systems such as Amazon SQS, 
        where batching is the most efficient way to consume messages, both in terms of time and cost.
        **Good Example** :  imagine processor has to check out a database connection to insert a record for every single insert operation, That’s pretty inefficient, especially if we’re processing lots of inserts.Fortunately, with BlueRiver we can use this technique, is grouping operations into batches, otherwise known as Partitioning. For insert operation, we want to insert entries into the database, but there’s certainly no need to do so one entry at a time.
  
  * **Dynamic batching** - BlueRiver allows developers to batch messages based on custom criteria. For    
        example, if your pipeline needs to build batches based on the user_id, email address, etc, 

  * **Ordering and Partitioning**  - BlueRiver allows developers to partition messages across workers, 
        guaranteeing messages within the same partition are processed in order. For example, if you want to guarantee all events tied to a given user_id are processed in order and not concurrently, you can use Dispatcher with  'Partition mode' option. See "Ordering and partitioning".

  * **Data Collector** - when source (Producer) of your app is web server and
        need absorb data from client request can use 'Collector' as Producer, 
        that asynchronous absorb data, then feeds to pipelines 

  * **Graceful shutdown** - first terminate Producers, wait until all processors job done, then shutdown
  
  * **Topology** - create and syncing components



# Examples :

The complete Examples on [Link](https://github.com/Rustixir/blueriver/tree/main/example).


# Explain: 

  * factory - take a fn for creating multiple instance
  
  * concurrency - creates multiple instance (For parallelism)  

  * router - used by dispatcher for routing message (RoundRobin || BroadCast)

  * producer_buffer_pool - producer internally used buffer for increase throughout

  * run_topology - BlueRiver always have one Producer Layer
        and at-least have 1_processor_layer and at-max 5_processor_layer
        and 1 optional layer `batcher` for creating and syncing components must use 
        **run_topology_X** or **run_topology_X_with_batcher** 


# Attention
  
  * Producer.dispatcher cannot be **partition** mode unless panic occur
  
  * Processor if have not next stage channel must return `ProcResult::Continue` unless 
        processor (skip) that message  




# Simple Example 

*   Customize (producer/processor) with 1 processor Layer

```rust

#[tokio::main]
async fn main() {

    let producer_factory = || Prod;
    let producer_concurrency = 3;
    let producer_router = RouterType::RoundRobin;
    let producer_buffer_pool = 100;


    let proc_factory = || Layer1Process;
    let proc_concurrency = 3;
    let proc_buffer_size = 10;

    

    // 1. create 3 processor instances by 'proc_concurrency'
    //
    // 2. create 3 producer instances  by 'producer_concurrency'
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


    tokio::time::sleep(Duration::from_secs(500)).await;

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



```