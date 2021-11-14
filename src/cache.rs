use tokio::sync::{mpsc, RwLock, RwLockReadGuard};
use std::io::Cursor;
use anyhow::Error;
use std::sync::Arc;
use std::collections::HashMap;
use glium::texture::SrgbTexture2d;
use crate::data::{Data, Set};
use glium::glutin::event_loop::EventLoopProxy;
use crate::{Call, Context};
use bytes::Bytes;

pub fn cache_set(set: Set, cacher: mpsc::Sender<String>){
    let urls :Vec<String> = set.items.iter().map( |item| item.image_url.clone() ).collect();
    for url in urls {
        // here we rely on our large queue size to prevent an overflow...
        // if I had more time I would create a way to signal the UX that the cacher was overwellmed,
        // instead of just 'dropping' these images
        cacher.try_send(url).unwrap_or_default();
    }
}


// it's best to have only one cacher working at any given time, otherwise
// they compete with each other for resources.  The goal of the cacher is to download and cache
// texture files in the order of the queue.  simply send tx.send(url) and your texture will be ready in seconds (we hope!)
pub async fn create_cacher(proxy: EventLoopProxy<Call>) -> mpsc::Sender<String> {
    let (tx,mut rx) :(mpsc::Sender<String>,mpsc::Receiver<String>)= mpsc::channel(16*1024);
    tokio::spawn( async move {
       while let Option::Some(url) = rx.recv().await {
           match fetch(url.clone() ).await {
               Ok(bytes) => {
                   proxy.send_event(Call::ToTexture {url,bytes} );
               }
               Err(error) => eprintln!("encountered an error when attempting to cache texture url: {}", url)
           }
       }
       proxy.send_event(Call::TextureCachingBatchComplete);
    } );
    tx
}

async fn fetch(url: String ) -> Result<Bytes,Error> {
    let response = reqwest::get(url.clone()).await?;
    Ok(response.bytes().await?)
}