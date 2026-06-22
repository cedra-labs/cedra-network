use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use cedra_logger::error;
use serde_json::Value;

use cedra_api_types::{AsConverter, EntryFunctionId, MoveValue, ViewRequest};
use cedra_storage_interface::{
    state_store::state_view::db_state_view::DbStateViewAtVersion, DbReader,
};
use cedra_types::{
    account_address::AccountAddress, indexer::indexer_db_reader::IndexerReader,
    whitelist::FungibleAssetStruct,
};
use cedra_vm::CedraVM;
use move_core_types::language_storage::TypeTag;

const WHITELIST_ADDRESS: &'static str =
    "0x3c9124028c90111d7cfd47a28fae30612e397d115c7b78f69713fb729347a77e";
const WHITELIST_FUNCTION: &'static str = "0x1::whitelist::get_asset_list";
const VIEW_FUNCTION_GAS_BUDGET: u64 = 10000;

// In-memory whitelist
pub struct Whitelist {
    assets: RwLock<Vec<FungibleAssetStruct>>,
    db_reader: Arc<dyn DbReader>,
    indexer_reader: Option<Arc<dyn IndexerReader>>,
}

impl Whitelist {
    pub fn new(
        db_reader: Arc<dyn DbReader>,
        indexer_reader: Option<Arc<dyn IndexerReader>>,
    ) -> Self {
        Self {
            assets: RwLock::new(vec![]),
            db_reader,
            indexer_reader,
        }
    }

    pub fn get_whitelist(&self) -> Vec<FungibleAssetStruct> {
        self.assets.read().unwrap().clone()
    }

    pub async fn update_whitelist(&self) {
        let new_list = Self::fetch_whitelist(self.db_reader.clone(), self.indexer_reader.clone());
        {
            let mut list = self.assets.write().unwrap();
            *list = new_list;
        }
    }

    fn fetch_whitelist(
        db_reader: Arc<dyn DbReader>,
        indexer_reader: Option<Arc<dyn IndexerReader>>,
    ) -> Vec<FungibleAssetStruct> {
        let admin = AccountAddress::from_hex_literal(WHITELIST_ADDRESS).unwrap();

        let move_values = match Self::execute_view(
            db_reader.clone(),
            indexer_reader.clone(),
            WHITELIST_FUNCTION,
            admin,
        ) {
            Ok(values) => values,
            Err(err) => {
                error!("Failed to fetch whitelist view function: {:?}", err);
                return vec![FungibleAssetStruct::cedra_coin_default()];
            },
        };

        let parsed = Self::parse_assets(move_values);
        if parsed.is_empty() {
            vec![FungibleAssetStruct::cedra_coin_default()]
        } else {
            parsed
        }
    }

    fn execute_view(
        db_reader: Arc<dyn DbReader>,
        indexer_reader: Option<Arc<dyn IndexerReader>>,
        function: &str,
        admin: AccountAddress,
    ) -> anyhow::Result<Vec<MoveValue>> {
        let latest_version = db_reader.get_latest_ledger_info_version();
        let state_view = db_reader
            .state_view_at_version(Some(latest_version.unwrap()))?
            ;

        let request = ViewRequest {
            function: EntryFunctionId::from_str(function)?,
            type_arguments: vec![],
            arguments: vec![serde_json::to_value(admin)?],
        };

        let view_function = state_view
            .as_converter(db_reader.clone(), indexer_reader.clone())
            .convert_view_function(request)?;

        let output = CedraVM::execute_view_function(
            &state_view,
            view_function.module.clone(),
            view_function.function.clone(),
            view_function.ty_args.clone(),
            view_function.args.clone(),
            VIEW_FUNCTION_GAS_BUDGET,
        );

        let values = output.values.map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let return_types = state_view
            .as_converter(db_reader.clone(), indexer_reader.clone())
            .function_return_types(&view_function)?
            .iter()
            .map(TypeTag::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?;

        values
            .into_iter()
            .zip(return_types.into_iter())
            .map(|(v, ty)| {
                state_view
                    .as_converter(db_reader.clone(), indexer_reader.clone())
                    .try_into_move_value(&ty, &v)
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    fn parse_assets(metadata_vec: Vec<MoveValue>) -> Vec<FungibleAssetStruct> {
        let mut assets = Vec::new();

        if metadata_vec.is_empty() {
            return assets;
        }

        let metadata = &metadata_vec[0];

        if let MoveValue::Vector(move_value) = metadata {
            for value in move_value {
                if let MoveValue::Struct(move_struct) = value {
                    let mut addr = String::new();
                    let mut module_name = String::new();
                    let mut symbol = String::new();

                    for (key, val) in &move_struct.0 {
                        match (key.0.as_str(), val) {
                            ("addr", Value::String(s)) => addr = s.to_string(),
                            ("module_name", Value::String(s)) => module_name = s.to_string(),
                            ("symbol", Value::String(s)) => symbol = s.to_string(),
                            _ => {},
                        }
                    }

                    assets.push(FungibleAssetStruct::from_legacy(
                        addr,
                        module_name.into(),
                        symbol.into(),
                    ));
                }
            }
        }

        assets
    }
}
