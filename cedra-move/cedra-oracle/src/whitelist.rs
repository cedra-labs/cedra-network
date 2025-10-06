use move_core_types::{
    language_storage::{TypeTag},
};

use std::sync::{RwLock};
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub struct Whitelist {
    stablecoins: RwLock<Vec<TypeTag>>,
}

impl Whitelist {
    pub fn new() -> Self {
        Self {
           stablecoins: RwLock::new(Self::fetch_whitelist()),
        } 
    }

    fn fetch_whitelist() -> Vec<TypeTag> {
        Vec::new() // TODO: fetch whitelist.
    }

    pub fn get_whitelist(&self) -> Vec<TypeTag>{
        let list = self.stablecoins.read().unwrap();
        list.clone()
    }

    pub fn exist(&self, stablecoin: TypeTag) -> bool {
        let list = self.stablecoins.read().unwrap();
        list.iter().any(|t| t == &stablecoin)
    }

    pub async fn update_whitelist(&self) {
        loop {
            let new_list = Self::fetch_whitelist();
            let mut list = self.stablecoins.write().unwrap();
            *list = new_list;

            sleep(Duration::from_secs(10));
        }
    }
}