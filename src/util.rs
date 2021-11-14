use std::hash::Hash;

use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};

use std::collections::HashMap;
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio::sync::oneshot::error::RecvError;



enum AsyncHashMapCommand<K, V>
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
{
    Put {
        key: K,
        value: V,
    },
    Get {
        key: K,
        tx: oneshot::Sender<Option<V>>,
    },
    Remove {
        key: K,
        tx: oneshot::Sender<Option<V>>,
    },
    Contains {
        key: K,
        tx: oneshot::Sender<bool>,
    },
    GetMap(oneshot::Sender<HashMap<K, V>>),
    SetMap(HashMap<K, V>),
    Clear,
}

#[derive(Clone)]
pub struct AsyncHashMap<K, V>
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
{
    tx: mpsc::Sender<AsyncHashMapCommand<K, V>>,
}

impl<K, V> AsyncHashMap<K, V>
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        let (tx, mut rx): (
            mpsc::Sender<AsyncHashMapCommand<K, V>>,
            mpsc::Receiver<AsyncHashMapCommand<K, V>>,
        ) = mpsc::channel(1);

        tokio::spawn(async move {
            let mut map = HashMap::new();
            while let Option::Some(command) = rx.recv().await {
                match command {
                    AsyncHashMapCommand::Put { key, value } => {
                        map.insert(key, value);
                    }
                    AsyncHashMapCommand::Get { key, tx } => {
                        let opt = map.get(&key).cloned();
                        tx.send(opt).unwrap_or_default();
                    }
                    AsyncHashMapCommand::Remove { key, tx } => {
                        let opt = map.remove(&key).clone();
                        tx.send(opt).unwrap_or_default();
                    }
                    AsyncHashMapCommand::Contains { key, tx } => {
                        tx.send(map.contains_key(&key)).unwrap_or_default();
                    }
                    AsyncHashMapCommand::GetMap(tx) => {
                        tx.send(map.clone());
                    }
                    AsyncHashMapCommand::SetMap(new_map) => map = new_map,
                    AsyncHashMapCommand::Clear => {
                        map.clear();
                    }
                }
            }
        });

        AsyncHashMap { tx: tx }
    }

    pub fn clear(&self) -> Result<(), Error> {
        self.tx.try_send(AsyncHashMapCommand::Clear)?;
        Ok(())
    }

    pub async fn put(&self, key: K, value: V) -> Result<(), Error> {
        self.tx
            .send(AsyncHashMapCommand::Put { key, value })
            .await?;
        Ok(())
    }

    pub async fn get(&self, key: K) -> Result<Option<V>, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AsyncHashMapCommand::Get { key, tx }).await?;
        Ok(rx.await?)
    }

    pub async fn remove(&self, key: K) -> Result<Option<V>, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AsyncHashMapCommand::Remove { key, tx })
            .await?;
        Ok(rx.await?)
    }

    pub async fn contains(&self, key: K) -> Result<bool, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AsyncHashMapCommand::Contains { key, tx })
            .await?;
        Ok(rx.await?)
    }

    pub async fn into_map(self) -> Result<HashMap<K, V>, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AsyncHashMapCommand::GetMap(tx)).await?;
        Ok(rx.await?)
    }

    pub fn set_map(&self, map: HashMap<K, V>) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            tx.send(AsyncHashMapCommand::SetMap(map)).await;
        });
    }
}

impl<K, V> From<HashMap<K, V>> for AsyncHashMap<K, V>
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
{
    fn from(map: HashMap<K, V>) -> Self {
        let async_map = AsyncHashMap::new();
        async_map.set_map(map);
        async_map
    }
}



pub struct Error {
    pub message: String
}

impl <K,V> From<tokio::sync::mpsc::error::SendError<AsyncHashMapCommand<K, V>>> for Error
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,{

    fn from(s: SendError<AsyncHashMapCommand<K, V>>) -> Self {
        Error{ message: s.to_string() }
    }
}

impl <K,V> From<tokio::sync::mpsc::error::TrySendError<AsyncHashMapCommand<K, V>>> for Error
    where
        K: Clone + Hash + Eq + PartialEq + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,{

    fn from(s: TrySendError<AsyncHashMapCommand<K, V>>) -> Self {
        Error{ message: s.to_string() }
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(r: RecvError) -> Self {
        Error {
            message: r.to_string()
        }
    }
}

