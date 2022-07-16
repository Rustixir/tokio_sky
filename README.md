
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
from different sources, known as producers, such as Apache Kafka, Apache Pulsar and others. 
inspired by elixir broadway



## Features

TokioSky takes the burden of defining concurrent GenStage topologies and provide 
a simple configuration API that automatically defines concurrent producers, 
concurrent processing, leading to both time and cost efficient 
ingestion and processing of data. It features:

  * **Producer** - source of data piplines 

  * **Processor** - process message also can dispath to next stage by `dispatcher` 

  * **BatchProcessor** process group of message, that is used for last stage, 
        have not next stage   

  * **Dispatcher** - dispatch message with three mode (`RoundRobin`, `BroadCast`, `Partition`)

  * **Customizable** - can use built-in `Producer`, `Processor`, `BatchProcessor` 
      like **Apache Kafka**, **Apache Pulsar** or 
      write your custom `Producer`, `Processor`, `BatchProcessor`

  * **Batching** - TokioSky provides built-in batching, allowing you to 
        group messages either by size and/or by time. This is important in systems
        such as Amazon SQS, where batching is the most efficient way to consume messages, 
        both in terms of time and cost. 
        **Good Example**  imagine processor has to check out 
        a database connection to insert a record for every single insert operation, That’s 
        pretty inefficient, especially if we’re processing lots of inserts.Fortunately, 
        with TokioSky we can use this technique, is grouping operations into batches, 
        otherwise known as Partitioning.
        See [Example](https://github.com/Rustixir/tokio_sky/tree/main/examples/batch_insert_mysql.rs) 

  * **Dynamic batching** - TokioSky allows developers to batch messages based 
        on custom criteria. For example, if your pipeline needs to build batches 
        based on the user_id, email address, etc, 
        See [Example](https://github.com/Rustixir/tokio_sky/tree/main/examples/dynamic_batching.rs) 

  * **Ordering and Partitioning**  - TokioSky allows developers to partition 
        messages across workers, guaranteeing messages within the same partition 
        are processed in order. For example, if you want to guarantee all 
        events tied to a given user_id are processed in order and not concurrently, 
        you can use Dispatcher with  `Partition` mode option. 
        See [Example](https://github.com/Rustixir/tokio_sky/tree/main/examples/ordering_and_paritioning.rs).

  * **Data Collector** - when source `Producer` of your app is web server and
        need absorb data from client request can use 'Collector' as `Producer`, 
        that asynchronous absorb data, then feeds to pipelines 
        See [Example](https://github.com/Rustixir/tokio_sky/tree/main/examples/collector.rs)

  * **Graceful shutdown** - first terminate Producers, wait until all processors job done, 
        then shutdown
  
  * **Topology** - create and syncing components



# Examples

The complete Examples on [Link](https://github.com/Rustixir/tokio_sky/tree/main/examples).




# Explain 

  * **factory** - instance factory  
  
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

  * All **Built-in processor** if have next stage, **must dispatcher not be partition mode**


# Crates.io

```
tokio_sky = 1.0.0
```


## Author
*   DanyalMh


# License

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
