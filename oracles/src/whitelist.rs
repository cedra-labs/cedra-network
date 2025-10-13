use move_core_types::{
    language_storage::{TypeTag},
};
use cedra_vm::CedraVM;
use std::{str::FromStr, sync::RwLock};
use tokio::time::{sleep, Duration};
use cedra_api_types::{
    AsConverter, EntryFunctionId, ViewFunction, ViewRequest, MoveValue
};

use cedra_storage_interface::{
    DbReader, 
    state_store::state_view::db_state_view::{
        DbStateViewAtVersion, DbStateView,
    }
};
use std::sync::Arc;
use cedra_types::{
    indexer::indexer_db_reader::IndexerReader, CedraCoinType, CoinType
};
use serde_json::Value;

#[derive(Debug, Clone)]
// StablecoinInfo contains FA stablecoin metadata.
pub struct StablecoinInfo {
    fa_address: String,
    decimals: u8,
}

impl StablecoinInfo {
    pub fn cedara_coin_info() -> Self {
        Self { fa_address: CedraCoinType::type_tag().to_string(), decimals: 8 }
    }

    pub fn get_fa_address(&self) -> String {
        self.fa_address.clone()
    }

    pub fn get_decimals(&self) -> u8 {
        self.decimals.clone()
    }
}

// Whitelist represents stablecoin whitelist that allow to get and update whitelist data.
pub struct Whitelist {
    stablecoins: RwLock<Vec<StablecoinInfo>>,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
}

impl Whitelist {
    pub fn new(db_reader: Arc<dyn DbReader>, indexer_reader: Option<Arc<dyn IndexerReader>>) -> Self {
        Self {
           stablecoins: RwLock::new(Self::fetch_whitelist(db_reader.clone(), indexer_reader.clone())),
           db_reader: db_reader,
           indexer_reader: indexer_reader,
        } 
    }

    pub fn get_whitelist(&self) -> Vec<StablecoinInfo>{
        let list = self.stablecoins.read().unwrap();
        list.clone()
    }

    // get_fa_address_coin_info (use this method only in pare with method exist)
    pub fn get_fa_address_coin_info(&self, fa_address: String) -> StablecoinInfo {
        if fa_address == CedraCoinType::type_tag().to_string() {
            return StablecoinInfo::cedara_coin_info();
        }
        let list = self.stablecoins.read().unwrap();
        let metadata = list.iter().find(|p| p.fa_address == fa_address);

        return metadata.unwrap().clone();
    }

    // exist returns true if requested stablecoin exists in the whitelist.
    pub fn exist(&self, stablecoin: String) -> bool {
        let list = self.stablecoins.read().unwrap();
        list.iter().any(|t| t.fa_address == stablecoin)
    }

    // update_whitelist - update whitlist data (should be run as a background task).
    pub async fn update_whitelist(&self) {
        loop {
            let new_list = Self::fetch_whitelist(self.db_reader.clone(), self.indexer_reader.clone());
            let mut list = self.stablecoins.write().unwrap();
            *list = new_list;

            sleep(Duration::from_secs(10)).await;
        }
    }

    fn fetch_whitelist(db_reader: Arc<dyn DbReader>, indexer_reader: Option<Arc<dyn IndexerReader>>) -> Vec<StablecoinInfo> {
        let latest_version = db_reader.get_latest_ledger_info_version();
        let state_view = db_reader.state_view_at_version(Some(latest_version.unwrap())).unwrap();

        // create view request.
        let request = ViewRequest {
            function: EntryFunctionId::from_str("0x1::whitelist::get_metadata_list").unwrap(),
            type_arguments: vec![],
            arguments: vec![],
        };

        // execute view request.
        let view_function_res = state_view.as_converter(db_reader.clone(), indexer_reader.clone())
            .convert_view_function(request)
            .map_err(|err| { 
                // we should log error and not panic, since storage can be empty or not intialized at the start.
                eprintln!("Failed to fetch whitelist: {:?}", err);
            });
        
        let view_function = view_function_res.unwrap();

        let output = CedraVM::execute_view_function(
            &state_view,
            view_function.module.clone(),
            view_function.function.clone(),
            view_function.ty_args.clone(),
            view_function.args.clone(),
        10000,
        ); 

        let values = output.values;

        if let Err(err) = values {
            println!("fetch_whitelist err: {:?}", err);
            // We shouldn't panic sience new storage hasn't whitelist registry.
            let wh: Vec<StablecoinInfo> = Vec::new();
            return wh;
        }
        
        let whitelist_bytes = values.unwrap();

        let return_types = Self::get_return_types(state_view.clone(), db_reader.clone(), indexer_reader.clone(), view_function.clone());
        let move_values = Self::get_move_vals(
            state_view.clone(), 
            db_reader.clone(), 
            indexer_reader.clone(), 
            return_types.clone(), 
            whitelist_bytes.clone());


        let whitelist = Self::parse_stablecoins(move_values.clone());

        whitelist
    }

    // get_return_types - extract response type from the ViewFunction response.
    fn get_return_types(
        state_view: DbStateView, 
        db_reader: Arc<dyn DbReader>, 
        indexer_reader: Option<Arc<dyn IndexerReader>>, 
        view_function: ViewFunction,
    ) -> Vec<TypeTag>{
          let return_types = state_view
                .as_converter(db_reader, indexer_reader)
                .function_return_types(&view_function)
                .and_then(|tys| {
                    tys.iter()
                        .map(TypeTag::try_from)
                        .collect::<anyhow::Result<Vec<_>>>()
                })
                .map_err(|err| {eprintln!("Failed to fetch whitelist: {:?}", err);});
            return_types.unwrap()
    }

    // get_move_vals - extracts Vec<Vec<u8> into Vec<MoveValue> from view request response.
    fn get_move_vals(
        state_view: DbStateView, 
        db_reader: Arc<dyn DbReader>, 
        indexer_reader: Option<Arc<dyn IndexerReader>>, 
        return_types: Vec<TypeTag>,
        values: Vec<Vec<u8>>,
    ) -> Vec<MoveValue> {
        let move_vals = values
                .into_iter()
                .zip(return_types.into_iter())
                .map(|(v, ty)| {
                    state_view
                        .as_converter(db_reader.clone(), indexer_reader.clone())
                        .try_into_move_value(&ty, &v)
                })
                .collect::<anyhow::Result<Vec<_>>>()
                .map_err(|err| {eprintln!("Failed to fetch whitelist: {:?}", err);});
        move_vals.unwrap()        
    }

    // parse_stablecoins - decodes received move values from `0x1::whitelist::get_metadata_list` into WhitelistMetadata.
    fn parse_stablecoins(metadata_vec: Vec<MoveValue>) -> Vec<StablecoinInfo> {
        let mut stablecoins = Vec::new();

        let metadata_len = metadata_vec.len();
        
        // exit if received value is empty.
        if metadata_len == 0 {
            return stablecoins;
        }

         // by default we should have only 0 index on the top level.
        let metadata = &metadata_vec[0];

        if let MoveValue::Vector(move_value) = metadata {
            for value in move_value {
                if let MoveValue::Struct(move_struct) = value {
                    let mut owner_address = String::new();
                    let mut decimals: u8 = 0;
                    let mut module_name = String::new();
                    let mut symbol = String::new();

                    // Can we remove this loop???
                    for (key, val) in &move_struct.0 {
                        let key_str = &key.0; // Identifier
                        match (key_str.as_str(), val) {
                            ("owner_address", Value::String(s)) => owner_address = s.to_string(),      
                            ("decimals", Value::Number(num)) => decimals = num.as_u64().unwrap() as u8,
                            ("module_name", Value::String(s)) => module_name = s.to_string(),
                            ("symbol", Value::String(s)) => symbol = s.to_string(),
                            _ => {}
                        }
                    }

                    let address = owner_address + "::" + &module_name + "::" + &symbol;
                    stablecoins.push(StablecoinInfo { fa_address: address,  decimals: decimals });
                }
            }
        }

        stablecoins
    }
}
