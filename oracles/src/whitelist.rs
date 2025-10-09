use move_core_types::{
    // value::{MoveValue as CoreMoveValue, MoveStructValue},
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
    indexer::indexer_db_reader::IndexerReader, CedraCoinType, CoinType,
};
use serde_json::Value;
use crate::{utils::decode_hex_string};

pub struct WhitelistMetadata {
    fa_address: TypeTag,
    metadata_address: String,
    decimals: u8,
}

pub struct Whitelist {
    stablecoins: RwLock<Vec<TypeTag>>, // RODO RwLock<Vec<TypeTag>> -> RwLock<Vec<WhitelistMetadata>>,
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
            let new_list = Self::fetch_whitelist(self.db_reader.clone(), self.indexer_reader.clone());
            let mut list = self.stablecoins.write().unwrap();
            *list = new_list;

            sleep(Duration::from_secs(10)).await;
        }
    }

    fn fetch_whitelist(db_reader: Arc<dyn DbReader>, indexer_reader: Option<Arc<dyn IndexerReader>>) -> Vec<TypeTag> {
        let latest_version = db_reader.get_latest_ledger_info_version();
        let state_view = db_reader.state_view_at_version(Some(latest_version.unwrap())).unwrap();

        let request = ViewRequest {
            function: EntryFunctionId::from_str("0x1::whitelist::get_metadata_list").unwrap(),
            type_arguments: vec![],
            arguments: vec![serde_json::Value::String(
                "3c9124028c90111d7cfd47a28fae30612e397d115c7b78f69713fb729347a77e".to_string(),
            )],
        };

        let view_function_res = state_view.as_converter(db_reader.clone(), indexer_reader.clone())
            .convert_view_function(request)
            .map_err(|err| { 
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
            let wh: Vec<TypeTag> = Vec::new();
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


        let mut whitelist = Self::parse_stablecoins(move_values.clone());
        whitelist.push(CedraCoinType::type_tag());

        whitelist
    }

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

    fn parse_stablecoins(vec: Vec<MoveValue>) -> Vec<TypeTag> {
        let mut stablecoins = Vec::new();

        for value in vec {
            if let MoveValue::Vector(move_value) = value {
                for value in move_value {
                      if let MoveValue::Struct(move_struct) = value {
                        let mut addr = String::new();
                        let mut module_name = String::new();
                        let mut symbol = String::new();

                        for (key, val) in move_struct.0 {
                            let key_str = key.0; // Identifier
                            match (key_str.as_str(), val) {
                                ("addr", Value::String(s)) => addr = s,
                                ("module_name", Value::String(s)) => module_name = decode_hex_string(&s),
                                ("symbol", Value::String(s)) => symbol = decode_hex_string(&s),
                                _ => {}
                            }
                        }

                        
                        let address = addr + "::" + &module_name + "::" + &symbol;

                        let fa_address = TypeTag::from_str(&address).unwrap();
                        stablecoins.push(fa_address);
                      }
                }
            }
        }

        stablecoins
    }
}
