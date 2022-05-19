

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



pub mod topology;