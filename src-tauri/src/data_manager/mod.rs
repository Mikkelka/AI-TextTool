pub mod commands;
pub mod manager;
pub mod types;

use std::sync::Arc;
use tokio::sync::Mutex;
use manager::DataManager;

pub struct SharedDataManager(pub Arc<Mutex<DataManager>>);

impl SharedDataManager {
    pub async fn new() -> Result<Self, types::DataError> {
        let mut dm = DataManager::new();
        dm.initialize().await?;
        Ok(Self(Arc::new(Mutex::new(dm))))
    }
}

pub use commands::*;
