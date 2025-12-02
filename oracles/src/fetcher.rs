use crate::whitelist::Whitelist;
use cedra_validator_transaction_pool::{TxnGuard, VTxnPoolState};
use anyhow::Result;
use cedra_types::{
    validator_txn::{Topic, ValidatorTransaction},
};
use cedra_types::oracle::PriceInfo;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{metadata::MetadataValue, Request};

pub mod pricefeed {
    tonic::include_proto!("stream");
}

use pricefeed::stream_service_client::StreamServiceClient;
use pricefeed::PriceRequest;
use pricefeed::StablecoinPrice;

pub struct PricesFetcher {
    whitelist: Arc<Whitelist>,
    vtxn_pool: VTxnPoolState,
        _permanent_guards: Mutex<Vec<TxnGuard>>,

    }

impl PricesFetcher {
    pub fn new(
         whitelist: Arc<Whitelist>,
        vtxn_pool: VTxnPoolState,
    ) -> Self {
        Self {
            whitelist,
            vtxn_pool,
            _permanent_guards: Mutex::new(Vec::new()),
        }
    }
     

    pub async fn fetch(&self) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Connecting to price feed server...");
    
    let mut client = StreamServiceClient::connect("http://37.27.228.131:40041").await?;
    println!("✅ Connected to server successfully!");


    let whitelist = self.whitelist.get_whitelist();

    // for fa_address in whitelist {
    let mut request = Request::new(PriceRequest {
        fa_address: "0x1::cedra_coin::CedraCoin".to_string(),
    });

    let header_value = MetadataValue::from_static("oracle-stream-646581e0-f5a2-44bd-8298-696ee5ac4f00-ryMBQwsomuv0TF0ae5AEzeEfU");
    request.metadata_mut().insert("authorization", header_value);
    
    println!("📡 Starting price stream for CedraCoin...");
    
    let mut stream = client.white_list_price_stream(request).await?.into_inner();

    while let Some(price_update) = stream.next().await {
        match price_update {
            Ok(price) => {
                let formatted_price = Self::format_price(&price);
                           println!("WhiteList Price Update - FA: {:?}, Price: {:?}, Timestamp: {}", 
                    formatted_price, price.fa_address, price.timestamp);

                self.update_price(formatted_price).await.expect("Error with put  price update txn to TransactionValidation");
                                
            }
            Err(e) => {
                println!("❌ Error receiving price: {}", e);
                // Implement retry logic here if needed
                break;
            }
        }
    // }
    }
    
    Ok(())
}

fn format_price(price: &StablecoinPrice) -> PriceInfo {
        let scale = price.scale as u32;
        let scaled_price = if scale <= 18 {
            price.price as u64
        } else {
            (price.price / 10_i64.pow(scale - 18)) as u64
        };

        PriceInfo::new(String::from("0xcf457e2e62739e7cc6d2b906acba3f17a708e0b98ed13518b221f79026dcd7b4::usdt::USDT"), scaled_price, price.scale as u8)
}

    async fn update_price(&self, price_info: PriceInfo) -> Result<(), Box<dyn std::error::Error>>  {
        let mut permanent_guards = self._permanent_guards.lock().await;

        // if !self.whitelist.exist(price_info.fa_address.clone()) {
        //     return Ok(());
        // }

        let txn = ValidatorTransaction::PriceUpdate(price_info);
        let topic = Topic::from("oracle_prices");
        let guard = self.vtxn_pool.put(topic, Arc::new(txn), None);

        permanent_guards.push(guard);
            return Ok(());


}

}
