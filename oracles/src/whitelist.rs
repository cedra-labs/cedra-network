use serde_json::Value;
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use tokio::time::{sleep, Duration};

use cedra_api_types::{AsConverter, EntryFunctionId, MoveValue, ViewRequest};
use cedra_storage_interface::{
    state_store::state_view::db_state_view::DbStateViewAtVersion, DbReader,
};
use cedra_types::{indexer::indexer_db_reader::IndexerReader, CedraCoinType, CoinType};
use cedra_vm::CedraVM;
use move_core_types::language_storage::TypeTag;

#[derive(Debug, Clone)]
pub struct StablecoinInfo {
    fa_address: String,
    decimals: u8,
}

impl StablecoinInfo {
    pub fn cedra_coin_info() -> Self {
        Self {
            fa_address: CedraCoinType::type_tag().to_string(),
            decimals: 8,
        }
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
    pub fn new(
        db_reader: Arc<dyn DbReader>,
        indexer_reader: Option<Arc<dyn IndexerReader>>,
    ) -> Self {
        let hardcoded_list = vec![
            StablecoinInfo {
                fa_address: "0x1::cedra_coin::CedraCoin".to_string(),
                decimals: 8,
            },
            StablecoinInfo {
                fa_address: "0xc745ffa4f97fa9739fae0cb173996f70bb8e4b0310fa781ccca2f7dc13f7db06::usdct::USDCT".to_string(),
                decimals: 8,
            },
            StablecoinInfo {
                fa_address: "0xca0746f983f7d03891c5e2dab8e321784357e687848b3840ae4cf5cc619dba7a::usdt::USDT".to_string(),
                decimals: 8,
            },
        ];

        Self {
            // stablecoins: RwLock::new(Self::fetch_whitelist(
            //     db_reader.clone(),
            //     indexer_reader.clone(),
            // )),
            stablecoins: RwLock::new(hardcoded_list),
            db_reader,
            indexer_reader,
        }
    }

    pub fn get_whitelist(&self) -> Vec<StablecoinInfo> {
        let list = self.stablecoins.read().unwrap();
        list.clone()
    }

    // use this method only in pare with method exist
    pub fn get_coin_info(&self, fa_address: String) -> StablecoinInfo {
        if fa_address == CedraCoinType::type_tag().to_string() {
            return StablecoinInfo::cedra_coin_info();
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
            let new_list =
                Self::fetch_whitelist(self.db_reader.clone(), self.indexer_reader.clone());
            let mut list = self.stablecoins.write().unwrap();
            *list = new_list;
            sleep(Duration::from_secs(10)).await;
        }
    }

    fn fetch_whitelist(
        db_reader: Arc<dyn DbReader>,
        indexer_reader: Option<Arc<dyn IndexerReader>>,
    ) -> Vec<StablecoinInfo> {
        let latest_version = db_reader.get_latest_ledger_info_version();
        let state_view = db_reader
            .state_view_at_version(Some(latest_version.unwrap()))
            .unwrap();

        let request = ViewRequest {
            function: EntryFunctionId::from_str("0x1::whitelist::get_metadata_list").unwrap(),
            type_arguments: vec![],
            arguments: vec![],
        };

        let view_function_res = state_view
            .as_converter(db_reader.clone(), indexer_reader.clone())
            .convert_view_function(request);

        let view_function = match view_function_res {
            Ok(v) => v,
            Err(err) => {
                eprintln!("Failed to fetch whitelist view function: {:?}", err);
                return vec![StablecoinInfo::cedra_coin_info()];
            },
        };

        let output = CedraVM::execute_view_function(
            &state_view,
            view_function.module.clone(),
            view_function.function.clone(),
            view_function.ty_args.clone(),
            view_function.args.clone(),
            10000,
        );

        let values = match output.values {
            Ok(v) => v,
            Err(err) => {
                eprintln!("fetch_whitelist err: {:?}", err);
                return vec![StablecoinInfo::cedra_coin_info()];
            },
        };

        let Ok(return_types) = state_view
            .as_converter(db_reader.clone(), indexer_reader.clone())
            .function_return_types(&view_function)
            .and_then(|tys| {
                tys.iter()
                    .map(TypeTag::try_from)
                    .collect::<anyhow::Result<Vec<_>>>()
            })
        else {
            eprintln!("Failed to get return types — returning default whitelist");
            return vec![StablecoinInfo::cedra_coin_info()];
        };

        let Ok(move_values) = values
            .into_iter()
            .zip(return_types.into_iter())
            .map(|(v, ty)| {
                state_view
                    .as_converter(db_reader.clone(), indexer_reader.clone())
                    .try_into_move_value(&ty, &v)
            })
            .collect::<anyhow::Result<Vec<_>>>()
        else {
            eprintln!("Failed to convert move values — returning default whitelist");
            return vec![StablecoinInfo::cedra_coin_info()];
        };

        let parsed = Self::parse_stablecoins(move_values);
        if parsed.is_empty() {
            vec![StablecoinInfo::cedra_coin_info()]
        } else {
            parsed
        }
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
                            ("decimals", Value::Number(num)) => {
                                decimals = num.as_u64().unwrap() as u8
                            },
                            ("module_name", Value::String(s)) => module_name = s.to_string(),
                            ("symbol", Value::String(s)) => symbol = s.to_string(),
                            _ => {},
                        }
                    }

                    let address = owner_address + "::" + &module_name + "::" + &symbol;
                    stablecoins.push(StablecoinInfo {
                        fa_address: address,
                        decimals,
                    });
                }
            }
        }

        stablecoins
    }
}
