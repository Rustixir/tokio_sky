use tokio::sync::oneshot;



pub fn start_shutdown_manager(shutdown_recv: oneshot::Receiver<()>, list_shutdown: Vec<oneshot::Sender<()>>) {
    
    tokio::spawn(async move {
        
        // Signal
        let _signal = shutdown_recv.await;

        // Send to all channels
        for shutdown in list_shutdown {
            _ = shutdown.send(());
        }
    });

}