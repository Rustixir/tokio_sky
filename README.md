
# TokioSky

<div align="center">

  <!-- Downloads -->
  <a href="https://crates.io/crates/tokio_sky">
    <img src="https://img.shields.io/crates/d/tokio_sky.svg?style=flat-square"
      alt="Download" />
  </a>
</div>



Build concurrent and multi-stage data ingestion and data processing 
pipelines with Rust+Tokio. TokioSky allows developers to consume data efficiently 
from different sources, known as producers, such as Apache Kafka and others. 
inspired by elixir broadway



## Features

TokioSky takes the burden of defining concurrent GenStage topologies and provide 
a simple configuration API that automatically defines concurrent producers, 
concurrent processing, leading to both time and cost efficient 
ingestion and processing of data. It features:

  * **Producer** - is source of data piplines 

  * **Processor** - process message also can dispath to next stage by `dispatcher` 

  * **BatchProcessor** process group of message, that is used for last stage, 
        have not next stage   

  * **Dispatcher** - dispatch message with three mode (`RoundRobin`, `BroadCast`, `Partition`)

  * **Customizable** - can use built-in Producer (like Apache Kafka) or write your own Producer 

  * **Batching** - TokioSky provides built-in batching, allowing you to 
        group messages either by size and/or by time. This is important in systems
        such as Amazon SQS, where batching is the most efficient way to consume messages, 
        both in terms of time and cost. **Good Example**  imagine processor has to check out 
        a database connection to insert a record for every single insert operation, That’s 
        pretty inefficient, especially if we’re processing lots of inserts.Fortunately, 
        with TokioSky we can use this technique, is grouping operations into batches, 
        otherwise known as Partitioning. For insert operation, we want to insert entries 
        into the database, but there’s certainly no need to do so one entry at a time.
  
  * **Dynamic batching** - TokioSky allows developers to batch messages based 
        on custom criteria. For example, if your pipeline needs to build batches 
        based on the user_id, email address, etc, 

  * **Ordering and Partitioning**  - TokioSky allows developers to partition 
        messages across workers, guaranteeing messages within the same partition 
        are processed in order. For example, if you want to guarantee all 
        events tied to a given user_id are processed in order and not concurrently, 
        you can use Dispatcher with  `Partition` mode option. See "Ordering and partitioning".

  * **Data Collector** - when source `Producer` of your app is web server and
        need absorb data from client request can use 'Collector' as `Producer`, 
        that asynchronous absorb data, then feeds to pipelines 

  * **Graceful shutdown** - first terminate Producers, wait until all processors job done, 
        then shutdown
  
  * **Topology** - create and syncing components



# Examples :

The complete Examples on [Link](https://github.com/Rustixir/tokio_sky/tree/main/examples).




# Explain: 

  * **factory** - factory instance 
  
  * **concurrency** - creates multiple instance (For parallelism)  

  * **router** - used by dispatcher for routing message (`RoundRobin` || `BroadCast` || `Partition`)

  * **producer_buffer_pool** - producer internally used buffer for increase throughout

  * **run_topology** - TokioSky always have one Producer Layer
        and at-least have 1 processor layer and at-max 5 processor layer
        and 1 optional layer `batcher` for creating and syncing components 
        must use `run_topology_X` or `run_topology_X_with_batcher` 


# Attention
  
  * Producer.dispatcher cannot be `Partition` mode 
  
  * Processor if have not next stage channel must return `ProcResult::Continue` 
        unless processor (skip) that message  



# Crates.io

```
tokio_sky = 1.0.0
```



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


## Author
*   DanyalMh


# License

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
