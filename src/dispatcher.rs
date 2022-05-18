

use hashring::HashRing;
use indexmap::IndexMap;
use tokio::sync::mpsc::{self, error::TrySendError};



pub type StageName = String;




// pub enum Request<T> {
//     Register(Sender<T>),
//     Dispatch(T)
// }




enum InternalDispatchError<T> {

    // T is msg produced but not exit any channel to consume it 
    DestinationDown(T),

    NotExist(T),

    Full(T),

}


pub enum DispatchError<T> {

    // for roundrobin
    NotExist(T),
    NotFound
}


#[derive(Copy , Clone)]
pub enum RouterType {
    RoundRobin,
    Broadcast,
    Partition
}


#[derive(Debug)]
pub enum StatusResult {
    SendersRepetive
}


pub struct Dispatcher<T> {
    c: usize,
    channels: IndexMap<StageName, mpsc::Sender<T>>,
    pub router_type: RouterType,
    hashring: Option<HashRing<usize>>
}

impl<T> Dispatcher<T> 
where
    T: Clone + Send + 'static
{
    

    pub fn new(channels: IndexMap<StageName, mpsc::Sender<T>>, 
               router_type: RouterType) -> Result<Self, StatusResult> {

        // Check channels to not be repetive
        if let Err(_) = Dispatcher::list_check(&channels) {
            return Err(StatusResult::SendersRepetive);
        }

        let hashring = if let RouterType::Partition = router_type {
            
            // Create HashRing
            let mut hr = HashRing::new();
            
            for (index, _) in channels.iter().enumerate() {
                hr.add(index);
            }

            Some(hr)
        
        } else {
            None
        };



        Ok(Dispatcher { 
            c: 0, 
            channels,
            router_type,
            hashring
        })
    }


    #[inline]
    pub async fn dispatch(&mut self, msg: T, batch_key: Option<String>) -> Result<(), DispatchError<T>> {
        match self.router_type {

            RouterType::Partition => {
                
                // if batch_key not exist panic
                self.partition(msg, batch_key.unwrap()).await
            }
            RouterType::RoundRobin => {
                self.roundrobin(msg).await
            }
            RouterType::Broadcast => {
                self.broadcast(msg).await
            }
        }
    }


    /* 
    
    /// return Err if key or chan exist
    #[inline]
    pub fn subscribe(&mut self, key: StageName, chan: mpsc::Sender<T>) -> Result<(), ()> {
        match self.check(&key, &chan) {
            Ok(_) => {
               self.channels.insert(key, chan);
               Ok(()) 
            }
            Err(_) => {
                Err(())
            }
        }
    }


    /// remove channel by key
    #[inline]
    pub fn unsubscribe(&mut self, key: &StageName) {
        self.channels.remove(key);
    }


    /// return Err if key not exist 
    #[inline]
    pub fn replace(&mut self, key: StageName, chan: mpsc::Sender<T>) -> Result<(), ()> {
        match self.channels.contains_key(&key) {
            true => {
               self.channels.insert(key, chan);
               Ok(()) 
            }
            false => {
                Err(())
            }
        }
    }

    #[inline]
    pub fn get_len(&self) -> usize {
        self.channels.len()
    }
    

    #[inline]
    pub fn get_type(&self) -> &RouterType {
        &self.router_type
    }
    

    */

    // ----------------Internal fn------------------------------------

    
    
    #[inline]
    async fn partition(&mut self, msg: T, batch_key: String) -> Result<(), DispatchError<T>> {

        let hashring = unsafe { self.hashring.as_mut().unwrap_unchecked() };

        let res = hashring.get(&batch_key);
        match res {
            Some(index) => {

                match self.channels.get_index(*index) {
                    Some((keychan, sender)) => {

                        // Send
                        if let Err(_) = sender.send(msg).await{

                            // if closed remove channel
                            let kc = keychan.to_owned();
                            let _ = self.channels.remove(&kc);

                            // remove from hashring
                            let i = index.to_owned(); 
                            hashring.remove(&i);
                            return Err(DispatchError::NotFound)

                        } else {
                            return Ok(())
                        }
                        
                    }
                    None => {
                        return Err(DispatchError::NotFound)
                    }
                }                            
            }
            None => Err(DispatchError::NotFound)
        }
    }
    

    #[inline]
    async fn broadcast(&mut self, msg: T) -> Result<(), DispatchError<T>> {

        if self.channels.len() == 0 {
            return Err(DispatchError::NotExist(msg))
        }

        if self.channels.len() == 1 {

            // if channel closed
            if let Err(_e) = self.channels[0].send(msg).await {

                // channel closed, get key by index
                let c = &self.channels;
                let key = c.get_index(0).unwrap().0.clone();

                self.channels.remove(&key);

            } 
            return Ok(())
        }

        let mut list = vec![];
        
        for index in 0..=(self.channels.len() - 2){
            let cmsg = msg.clone();

            if let Err(_e) = self.channels[index].send(cmsg).await {
                // channel closed, get key by index
                let c = &self.channels;
                let (key, _) = c.get_index(index).unwrap();
                list.push(key.clone());

            } 
        }

        let _ = self.channels[self.channels.len() - 1].send(msg).await;


        // remove closed list from channels  
        list.iter().for_each(|key| {
            self.channels.remove(key);
        });

        Ok(())
    }


    /// send msg to next destination
    /// 
    /// roundrobin is safe if a destination terminate
    /// auto detect it and remove from channels 
    #[inline]
    async fn roundrobin(&mut self, mut msg: T) -> Result<(), DispatchError<T>> {

        if self.channels.len() == 0 {
            return Err(DispatchError::NotExist(msg))
        }

        if self.channels.len() == 1 {

            // if channel closed
            if let Err(e) = self.channels[0].send(msg).await {

                // channel closed, get key by index
                let c = &self.channels;
                let key = c.get_index(0).unwrap().0.clone();

                self.channels.remove(&key);

                return Err(DispatchError::NotExist(e.0));
            } 
            return Ok(())
        }


        loop {
            match self.logic_roundrobin(msg).await {
                Ok(_) => return Ok(()),
                
                // if destinationDown but exist channel try again
                Err(InternalDispatchError::DestinationDown(m)) => {
                    msg = m;
                }

                // processor buffer was full, try for next processor
                Err(InternalDispatchError::Full(m)) => {
                    msg = m;
                }

                // not exist any channel
                Err(InternalDispatchError::NotExist(m)) => {
                    msg = m;
                    return Err(DispatchError::NotExist(msg));
                }
                              
                
            }
        }
    }

    #[inline]
    async fn logic_roundrobin(&mut self, mut msg: T) -> Result<(), InternalDispatchError<T>> {
        
        // get next index
        let index = self.next_index();

        // send msg 
        match self.channels[index].try_send(msg) {
            
            // sending was successful 
            Ok(_ok) => {

                // return Ok() 
                return Ok(())
            }
            
            // channel closed
            Err(TrySendError::Closed(b)) => {

                // take ownership of msg
                msg = b;

                // this sender remove from channels
                let (key, _) = self.channels.get_index(index).unwrap();
                let key = key.clone();
                self.channels.remove(&key);
                

                // if not exist destination return Err
                if self.channels.len() == 0 {

                    return Err(InternalDispatchError::NotExist(msg))
                }

                return Err(InternalDispatchError::DestinationDown(msg)) 

            }
       
            Err(TrySendError::Full(b)) => {
                msg = b;
                return Err(InternalDispatchError::Full(msg))
            }
       
        }
                
    }




    fn next_index(&mut self) -> usize {
        let mut index = self.c;

        self.c += 1;

        if index >= self.channels.len() {
            self.c = 0;
            index = 0;
        }

        return index
    }

    
    /// Check channels to not be repetive
    fn list_check(channels: &IndexMap<StageName, mpsc::Sender<T>>) -> Result<(), ()> {
        for (oindex, (outer_key, outer_chan)) in channels.iter().enumerate() {
            for (iindex, (inner_key, inner_chan)) in channels.iter().enumerate() {
            
                // if not was itself && channel was same
                if oindex != iindex && ( outer_key == inner_key || outer_chan.same_channel(inner_chan)) {
                    return Err(())
                }
            
            }
        }

        Ok(())
    }

    /* 
    /// Check channel to registered before
    fn check(&self, pkey: &StageName, pchan: &mpsc::Sender<T>) -> Result<(), ()> {
        for (_index, (key, chan)) in self.channels.iter().enumerate() {
            
            // if channel was same
            if pkey == key || pchan.same_channel(chan) {
                return Err(())
            }

        }
        Ok(())
    }

    */
}



