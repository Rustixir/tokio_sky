

/// trait Producer & starter
mod producer;

/// trait BatchProcessor & starter
mod batcher;

/// trait Processor & starter 
mod processor;


/// shutdown manager
mod shutdown_manager;


/// syncing components
mod topology;


/// dispatcher with two mode (RoundRobin & BroadCast)
mod dispatcher; 





pub use async_trait::async_trait;

/// Built-in producer & processor
pub mod builtin;


pub use batcher::{BatchProcessor, BatcherTerminate};

pub use processor::{Processor, ProcResult};

pub use dispatcher::RouterType;


pub use producer::Producer;



pub use topology::{

    run_topology_1, 
    run_topology_2, 
    run_topology_3,
    run_topology_4,
    run_topology_5,

    run_topology_1_with_batcher,
    run_topology_2_with_batcher,
    run_topology_3_with_batcher,
    run_topology_4_with_batcher,
    run_topology_5_with_batcher,

    CONCURRENCY,
    BUFFER_POOL_SIZE,
    BUFFER_SIZE,
    BATCH_SIZE,
    BATCH_TIMEOUT

};

