// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use cedra_api_types::AsConverter;
use cedra_block_executor::txn_provider::default::DefaultTxnProvider;
use cedra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    hash::HashValue,
    ValidCryptoMaterialStringExt,
};
use cedra_gas_schedule::{InitialGasSchedule, TransactionGasParameters};
use cedra_resource_viewer::CedraValueAnnotator;
use cedra_transaction_simulation::{
    InMemoryStateStore, SimulationStateStore, GENESIS_CHANGE_SET_HEAD,
};
use cedra_types::{
    account_config::{
        cedra_test_root_address, primary_apt_store, AccountResource, FungibleStoreResource,
        ObjectGroupResource,
    },
    block_metadata::BlockMetadata,
    chain_id::ChainId,
    contract_event::ContractEvent,
    state_store::{state_key::StateKey, table::TableHandle, MoveResourceExt, TStateView},
    transaction::{
        signature_verified_transaction::into_signature_verified_block,
        EntryFunction as TransactionEntryFunction, ExecutionStatus, RawTransaction,
        Script as TransactionScript, Transaction, TransactionOutput, TransactionStatus,
    },
};
use cedra_vm::{cedra_vm::CedraVMBlockExecutor, VMBlockExecutor};
use cedra_vm_environment::prod_configs::set_paranoid_type_checks;
use cedra_vm_genesis::GENESIS_KEYPAIR;
use clap::Parser;
use codespan_reporting::{diagnostic::Severity, term::termcolor::Buffer};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_bytecode_verifier::verify_module;
use move_command_line_common::{
    address::ParsedAddress, files::verify_and_create_named_address_mapping, testing::EXP_EXT,
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    move_resource::MoveStructType,
    parser::parse_type_tag,
    transaction_argument::{convert_txn_args, TransactionArgument},
    value::{MoveTypeLayout, MoveValue},
};
use move_model::metadata::LanguageVersion;
use move_symbol_pool::Symbol as MoveSymbol;
use move_transactional_test_runner::{
    framework::{run_test_impl, CompiledState, MoveTestAdapter},
    tasks::{InitCommand, SyntaxChoice, TaskInput},
    vm_test_harness::{PrecompiledFilesModules, TestRunConfig},
};
use move_vm_runtime::move_vm::SerializedReturnValues;
use once_cell::sync::Lazy;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    fmt,
    path::Path,
    string::String,
    sync::Arc,
};
use tempfile::NamedTempFile;

/**
 * Definitions
 */

/// The Cedra transaction test adapter.
///
/// This differs from the SimpleVMTestAdapter in a few ways to ensure that our tests mimics
/// production settings:
///   - It uses a StateView as its storage backend
///   - It executes transactions through CedraVM, instead of MoveVM directly
struct CedraTestAdapter<'a> {
    compiled_state: CompiledState<'a>,
    storage: InMemoryStateStore,
    default_syntax: SyntaxChoice,
    private_key_mapping: BTreeMap<String, Ed25519PrivateKey>,
    run_config: TestRunConfig,
}

/// Parameters *required* to create a transaction.
struct TransactionParameters {
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_timestamp_secs: u64,
}

/// Cedra-specific arguments for the publish command.
#[derive(Parser, Debug)]
struct CedraPublishArgs {
    #[clap(long = "private-key", value_parser = RawPrivateKey::parse)]
    private_key: Option<RawPrivateKey>,

    #[clap(long = "expiration")]
    expiration_time: Option<u64>,

    #[clap(long = "sequence-number")]
    sequence_number: Option<u64>,

    #[clap(long = "gas-price")]
    gas_unit_price: Option<u64>,

    #[clap(long = "override-signer", value_parser= ParsedAddress::parse)]
    override_signer: Option<ParsedAddress>,
}

#[derive(Debug, Clone)]
struct SignerAndKeyPair {
    address: ParsedAddress,
    private_key: Option<RawPrivateKey>,
}

/// Cedra-specifc arguments for the run command.
#[derive(Parser, Debug)]
struct CedraRunArgs {
    #[clap(long = "private-key", value_parser = RawPrivateKey::parse)]
    private_key: Option<RawPrivateKey>,

    #[clap(long = "script")]
    script: bool,

    #[clap(long = "expiration")]
    expiration_time: Option<u64>,

    #[clap(long = "sequence-number")]
    sequence_number: Option<u64>,

    #[clap(long = "gas-price")]
    gas_unit_price: Option<u64>,

    #[clap(long = "show-events")]
    show_events: bool,

    #[clap(long = "secondary-signers", value_parser = SignerAndKeyPair::parse, num_args = 0..)]
    secondary_signers: Option<Vec<SignerAndKeyPair>>,
}

/// Cedra-specifc arguments for the init command.
#[derive(Parser, Debug)]
struct CedraInitArgs {
    #[clap(long = "private-keys", value_parser = parse_named_private_key, num_args = 0..)]
    private_keys: Option<Vec<(Identifier, Ed25519PrivateKey)>>,
    #[clap(long = "initial-coins")]
    initial_coins: Option<u64>,
}

/// A raw private key -- either a literal or an unresolved name.
#[derive(Debug, Clone)]
enum RawPrivateKey {
    Named(Identifier),
    Anonymous(Ed25519PrivateKey),
}

/// Command to initiate a block metadata transaction.
#[derive(Parser, Debug)]
struct BlockCommand {
    #[clap(long = "proposer", value_parser = ParsedAddress::parse)]
    proposer: ParsedAddress,

    #[clap(long = "time")]
    time: u64,
}

/// Command to view a table item.
#[derive(Parser, Debug)]
struct ViewTableCommand {
    #[clap(long = "table_handle")]
    table_handle: AccountAddress,

    #[clap(long = "key_type", value_parser = parse_type_tag)]
    key_type: TypeTag,

    #[clap(long = "value_type", value_parser = parse_type_tag)]
    value_type: TypeTag,

    #[clap(long = "key_value", value_parser = parse_value)]
    key_value: serde_json::Value,
}

fn parse_value(input: &str) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_str(input)
}

/// Custom commands for the transactional test flow.
#[derive(Parser, Debug)]
enum CedraSubCommand {
    #[clap(name = "block")]
    BlockCommand(BlockCommand),

    #[clap(name = "view_table")]
    ViewTableCommand(ViewTableCommand),
}

/**
 * Parsing
 */

fn parse_ed25519_private_key(s: &str) -> Result<Ed25519PrivateKey> {
    Ok(Ed25519PrivateKey::from_encoded_string(s)?)
}

impl RawPrivateKey {
    fn parse(s: &str) -> Result<Self> {
        if let Ok(private_key) = parse_ed25519_private_key(s) {
            return Ok(Self::Anonymous(private_key));
        }
        let name = Identifier::new(s)
            .map_err(|_| format_err!("Failed to parse '{}' as private key.", s))?;
        Ok(Self::Named(name))
    }
}

fn parse_named_private_key(s: &str) -> Result<(Identifier, Ed25519PrivateKey)> {
    let before_after = s.split('=').collect::<Vec<_>>();

    if before_after.len() != 2 {
        bail!("Invalid named private key assignment. Must be of the form <private_key_name>=<private_key>, but found '{}'", s);
    }

    let name = Identifier::new(before_after[0])
        .map_err(|_| format_err!("Invalid private key name '{}'", s))?;
    let private_key = parse_ed25519_private_key(before_after[1])?;

    Ok((name, private_key))
}

impl SignerAndKeyPair {
    fn parse(s: &str) -> Result<Self> {
        if let Ok(address) = ParsedAddress::parse(s) {
            return Ok(Self {
                address,
                private_key: None,
            });
        };

        let before_after = s.split('=').collect::<Vec<_>>();

        if before_after.len() != 2 {
            bail!("Invalid signer and key pair. Must be of the form <signer addr>=<private_key> or <named signer addr>, but found '{}'", s);
        }

        let address = ParsedAddress::parse(before_after[0])?;
        let private_key = RawPrivateKey::parse(before_after[1])?;

        Ok(Self {
            address,
            private_key: Some(private_key),
        })
    }
}

pub struct FakeDbReader {}

impl cedra_storage_interface::DbReader for FakeDbReader {
    fn indexer_enabled(&self) -> bool {
        false
    }
}

/**
 * Helpers
 */

/// Default private key mappings for special Cedra accounts.
fn cedra_framework_private_key_mapping() -> Vec<(String, Ed25519PrivateKey)> {
    vec![("Root".to_string(), GENESIS_KEYPAIR.0.clone())]
}

fn panic_missing_private_key_named(cmd_name: &str, name: &str) -> ! {
    panic!(
        "Missing private key. Either add a `--private-key <priv_key>` argument \
            to the {} command, or associate an address to the \
            name '{}' in the init command.",
        cmd_name, name,
    )
}

fn panic_missing_private_key(cmd_name: &str) -> ! {
    panic!(
        "Missing private key. Try adding a `--private-key <priv_key>` \
            argument to the {} command.",
        cmd_name
    )
}

static cedra_framework_FILES: Lazy<Vec<String>> = Lazy::new(|| {
    cedra_cached_packages::head_release_bundle()
        .files()
        .unwrap()
});

static PRECOMPILED_cedra_framework_V2: Lazy<PrecompiledFilesModules> = Lazy::new(|| {
    let named_address_mapping_strings: Vec<String> = cedra_framework::named_addresses()
        .iter()
        .map(|(string, num_addr)| format!("{}={}", string, num_addr))
        .collect();

    let is_experimental =
        |f: &String| f.contains("cedra-move/framework/cedra-experimental/sources");

    // cedra-experimental can be using latest language features, while others cannot.
    // So we need to compile it twice, once without cedra-experimental, and once with.
    // (in the second pass, we need to provide both, in case cedra-experimental depends on other modules)

    let all_sources = cedra_cached_packages::head_release_bundle()
        .files()
        .unwrap();
    let prod_sources = all_sources
        .iter()
        .filter(|f| !is_experimental(f))
        .cloned()
        .collect::<Vec<_>>();

    let mut options = move_compiler_v2::Options {
        sources: prod_sources,
        dependencies: vec![],
        named_address_mapping: named_address_mapping_strings.clone(),
        known_attributes: cedra_framework::extended_checks::get_all_attribute_names().clone(),
        language_version: None,
        ..move_compiler_v2::Options::default()
    };

    let (_global_env, mut modules) = move_compiler_v2::run_move_compiler_to_stderr(options.clone())
        .expect("framework compilation succeeds");

    options.sources = all_sources;
    options.language_version = Some(LanguageVersion::latest());

    let (experimental_global_env, experimental_modules) =
        move_compiler_v2::run_move_compiler_to_stderr(options)
            .expect("cedra-experimental compilation succeeds");

    for cur_module in experimental_modules {
        let file_name = experimental_global_env.get_file(
            experimental_global_env
                .get_file_id(cur_module.loc().file_hash())
                .unwrap(),
        );
        if is_experimental(&file_name.to_str().unwrap().to_string()) {
            modules.push(cur_module);
        }
    }
    PrecompiledFilesModules::new(cedra_framework_FILES.clone(), modules)
});

/**
 * Test Adapter Implementation
 */

impl<'a> CedraTestAdapter<'a> {
    /// Look up the named private key in the mapping.
    fn resolve_named_private_key(&self, s: &IdentStr) -> Ed25519PrivateKey {
        if let Some(private_key) = self.private_key_mapping.get(s.as_str()) {
            return private_key.clone();
        }
        panic!("Failed to resolve private key '{}'", s)
    }

    /// Resolve a raw private key into a numeric one.
    fn resolve_private_key(&self, private_key: &RawPrivateKey) -> Ed25519PrivateKey {
        match private_key {
            RawPrivateKey::Anonymous(private_key) => private_key.clone(),
            RawPrivateKey::Named(name) => self.resolve_named_private_key(name),
        }
    }

    /// Resolve addresses and private keys for secondary signers.
    fn resolve_secondary_signers(
        &mut self,
        secondary_signers: &[SignerAndKeyPair],
    ) -> (Vec<AccountAddress>, Vec<Ed25519PrivateKey>) {
        let mut addresses = vec![];
        let mut private_keys = vec![];

        for SignerAndKeyPair {
            address,
            private_key,
        } in secondary_signers
        {
            addresses.push(self.compiled_state().resolve_address(address));

            let resolved_private_key = match (private_key, address) {
                (Some(private_key), _) => self.resolve_private_key(private_key),
                (None, ParsedAddress::Named(named_addr)) => {
                    match self.private_key_mapping.get(named_addr) {
                        Some(private_key) => private_key.clone(),
                        None => panic!(
                            "Failed to resolve private key for secondary signer {}.",
                            named_addr
                        ),
                    }
                },
                (None, ParsedAddress::Numerical(addr)) => {
                    panic!("No private key provided for secondary signer {}.", addr)
                },
            };

            private_keys.push(resolved_private_key);
        }

        (addresses, private_keys)
    }

    /// Obtain a Rust representation of the account resource from storage, which is used to derive
    /// a few default transaction parameters.
    fn fetch_account_resource(&self, signer_addr: &AccountAddress) -> AccountResource {
        self.storage
            .get_state_value_bytes(
                &StateKey::resource_typed::<AccountResource>(signer_addr).unwrap(),
            )
            .unwrap()
            .map(|bytes| bcs::from_bytes(&bytes).unwrap())
            .unwrap_or(AccountResource::new(
                0,
                signer_addr.to_vec(),
                cedra_types::event::EventHandle::new(
                    cedra_types::event::EventKey::new(0, *signer_addr),
                    0,
                ),
                cedra_types::event::EventHandle::new(
                    cedra_types::event::EventKey::new(1, *signer_addr),
                    0,
                ),
            ))
    }

    /// Obtain the CedraCoin amount under address `signer_addr`
    fn fetch_account_balance(&self, signer_addr: &AccountAddress) -> Result<u64> {
        Ok(FungibleStoreResource::fetch_move_resource_from_group(
            &self.storage,
            &primary_apt_store(*signer_addr),
            &ObjectGroupResource::struct_tag(),
        )?
        .unwrap()
        .balance())
    }

    /// Derive the default transaction parameters from the account and balance resources fetched
    /// from storage. In the future, we are planning to allow the user to override these using
    /// command arguments.
    fn fetch_transaction_parameters(
        &self,
        signer_addr: &AccountAddress,
        sequence_number: Option<u64>,
        expiration_time: Option<u64>,
        gas_unit_price: Option<u64>,
        max_gas_amount: Option<u64>,
    ) -> Result<TransactionParameters> {
        let account_resource = self.fetch_account_resource(signer_addr);

        let sequence_number = sequence_number.unwrap_or_else(|| account_resource.sequence_number());
        let max_number_of_gas_units =
            TransactionGasParameters::initial().maximum_number_of_gas_units;
        let gas_unit_price = gas_unit_price.unwrap_or(1000);
        let max_gas_amount = max_gas_amount.unwrap_or_else(|| {
            if gas_unit_price == 0 {
                u64::from(max_number_of_gas_units)
            } else {
                let account_balance = self.fetch_account_balance(signer_addr).unwrap();
                std::cmp::min(
                    u64::from(max_number_of_gas_units),
                    account_balance / gas_unit_price,
                )
            }
        });
        let expiration_timestamp_secs = expiration_time.unwrap_or(40000);

        Ok(TransactionParameters {
            sequence_number,
            gas_unit_price,
            max_gas_amount,
            expiration_timestamp_secs,
        })
    }

    /// Perform a single transaction.
    ///
    /// Should error if the transaction ends up being discarded, or having a status other than
    /// EXECUTED.
    fn run_transaction(&mut self, txn: Transaction) -> Result<TransactionOutput> {
        let txn_block = vec![txn];
        let sig_verified_block = into_signature_verified_block(txn_block);
        let txn_provider = DefaultTxnProvider::new(sig_verified_block);
        let mut outputs = CedraVMBlockExecutor::new()
            .execute_block_no_limit(&txn_provider, &self.storage.clone())?;

        assert_eq!(outputs.len(), 1);

        let output = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(kept_vm_status) => {
                self.storage.apply_write_set(output.write_set())?;
                match kept_vm_status {
                    ExecutionStatus::Success => Ok(output),
                    _ => {
                        bail!(
                            "Failed to execute transaction. ExecutionStatus: {:?}",
                            kept_vm_status
                        )
                    },
                }
            },
            TransactionStatus::Discard(status_code) => {
                bail!("Transaction discarded. VM status code: {:?}", status_code)
            },
            TransactionStatus::Retry => panic!(),
        }
    }

    fn create_and_fund_account(&mut self, account_addr: AccountAddress, amount: u64) {
        let parameters = self
            .fetch_transaction_parameters(&cedra_test_root_address(), None, None, None, None)
            .unwrap();

        let txn = RawTransaction::new(
            cedra_test_root_address(),
            parameters.sequence_number,
            cedra_cached_packages::cedra_stdlib::cedra_account_create_account(account_addr),
            parameters.max_gas_amount,
            parameters.gas_unit_price,
            parameters.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone())
        .unwrap()
        .into_inner();

        self.run_transaction(Transaction::UserTransaction(txn))
            .expect("Failed to create an account. This should not happen.");

        let txn = RawTransaction::new(
            cedra_test_root_address(),
            parameters.sequence_number + 1,
            cedra_cached_packages::cedra_stdlib::cedra_coin_mint(account_addr, amount),
            parameters.max_gas_amount,
            parameters.gas_unit_price,
            parameters.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone())
        .unwrap()
        .into_inner();

        self.run_transaction(Transaction::UserTransaction(txn))
            .expect("Failed to mint cedra coin. This should not happen.");
    }
}

impl<'a> MoveTestAdapter<'a> for CedraTestAdapter<'a> {
    type ExtraInitArgs = CedraInitArgs;
    type ExtraPublishArgs = CedraPublishArgs;
    type ExtraRunArgs = CedraRunArgs;
    type ExtraValueArgs = ();
    type Subcommand = CedraSubCommand;

    fn compiled_state(&mut self) -> &mut CompiledState<'a> {
        &mut self.compiled_state
    }

    fn default_syntax(&self) -> SyntaxChoice {
        self.default_syntax
    }

    fn known_attributes(&self) -> &BTreeSet<String> {
        cedra_framework::extended_checks::get_all_attribute_names()
    }

    fn run_config(&self) -> TestRunConfig {
        self.run_config.clone()
    }

    fn init(
        default_syntax: SyntaxChoice,
        run_config: TestRunConfig,
        pre_compiled_deps_v2: &'a PrecompiledFilesModules,
        task_opt: Option<TaskInput<(InitCommand, Self::ExtraInitArgs)>>,
    ) -> (Self, Option<String>) {
        set_paranoid_type_checks(true);
        // Named address mapping
        let additional_named_address_mapping = match task_opt.as_ref().map(|t| &t.command) {
            Some((InitCommand { named_addresses }, _)) => {
                verify_and_create_named_address_mapping(named_addresses.clone()).unwrap()
            },
            None => BTreeMap::new(),
        };

        let mut named_address_mapping = cedra_framework::named_addresses().clone();

        for (name, addr) in additional_named_address_mapping.clone() {
            if named_address_mapping.contains_key(&name) {
                panic!("Invalid init. The named address '{}' already exists.", name)
            }
            named_address_mapping.insert(name, addr);
        }

        // Genesis modules
        let storage = InMemoryStateStore::new();
        storage
            .apply_write_set(GENESIS_CHANGE_SET_HEAD.write_set())
            .unwrap();

        // Builtin private key mapping
        let mut private_key_mapping = BTreeMap::new();
        for (name, private_key) in cedra_framework_private_key_mapping() {
            private_key_mapping.insert(name, private_key);
        }

        // Initial coins to mint, defaults to 5,000,000
        let mut coins_to_mint = 5000000;

        if let Some(TaskInput {
            command: (_, init_args),
            ..
        }) = task_opt
        {
            // Private key mapping
            if let Some(additional_private_key_mapping) = init_args.private_keys {
                for (name, private_key) in additional_private_key_mapping {
                    if private_key_mapping.contains_key(name.as_str()) {
                        panic!(
                            "Invalid init. The named private key '{}' already exists.",
                            name
                        )
                    }
                    private_key_mapping.insert(name.as_str().to_string(), private_key);
                }
            }

            if let Some(initial_coins) = init_args.initial_coins {
                coins_to_mint = initial_coins;
            }
        }

        let mut adapter = Self {
            compiled_state: CompiledState::new(named_address_mapping, pre_compiled_deps_v2, None),
            default_syntax,
            storage,
            private_key_mapping,
            run_config,
        };

        for (_, addr) in additional_named_address_mapping {
            adapter.create_and_fund_account(addr.into_inner(), coins_to_mint);
        }

        (adapter, None)
    }

    fn compile_module(
        &mut self,
        syntax: SyntaxChoice,
        data: Option<NamedTempFile>,
        start_line: usize,
        command_lines_stop: usize,
    ) -> Result<(
        NamedTempFile,
        Option<MoveSymbol>,
        CompiledModule,
        Option<String>,
    )> {
        let (data, named_addr_opt, module, opt_model, warnings_opt) =
            self.compile_module_default(syntax, data, start_line, command_lines_stop)?;
        let warnings_opt = match (syntax, opt_model) {
            (SyntaxChoice::IR, _) => warnings_opt,
            (_, Some(model)) => {
                let _runtime_metadata =
                    cedra_framework::extended_checks::run_extended_checks(&model);
                // TODO(#13327): call inject_runtime_metadata in built_package.rs?  what file?
                if model.diag_count(Severity::Warning) > 0 {
                    let mut error_writer = Buffer::no_color();
                    model.report_diag(&mut error_writer, Severity::Warning);
                    let extended_warnings =
                        String::from_utf8_lossy(&error_writer.into_inner()).to_string();
                    if model.has_errors() {
                        bail!("extended checks failed:\n\n{}", extended_warnings);
                    };
                    match warnings_opt {
                        Some(warnings) => Some(warnings + &extended_warnings),
                        None => Some(extended_warnings),
                    }
                } else {
                    warnings_opt
                }
            },
            (_, None) => {
                bail!(
                    "Cannot run extended checks, no model:\n\n{}",
                    warnings_opt.unwrap_or_else(|| "No compiler warnings".to_string())
                );
            },
        };
        Ok((data, named_addr_opt, module, warnings_opt))
    }

    fn compile_script(
        &mut self,
        syntax: SyntaxChoice,
        data: Option<NamedTempFile>,
        start_line: usize,
        command_lines_stop: usize,
    ) -> Result<(CompiledScript, Option<String>)> {
        let (compiled_script, opt_model, warnings_opt) =
            self.compile_script_default(syntax, data, start_line, command_lines_stop)?;
        let warnings_opt = match (syntax, opt_model) {
            (SyntaxChoice::IR, _) => warnings_opt,
            (_, Some(model)) => {
                let _runtime_metadata =
                    cedra_framework::extended_checks::run_extended_checks(&model);
                // TODO(#13327): call inject_runtime_metadata in built_package.rs?  what file?
                if model.diag_count(Severity::Warning) > 0 {
                    let mut error_writer = Buffer::no_color();
                    model.report_diag(&mut error_writer, Severity::Warning);
                    let extended_warnings =
                        String::from_utf8_lossy(&error_writer.into_inner()).to_string();
                    if model.has_errors() {
                        bail!("extended checks failed:\n\n{}", extended_warnings);
                    };
                    match warnings_opt {
                        Some(warnings) => Some(warnings + &extended_warnings),
                        None => Some(extended_warnings),
                    }
                } else {
                    warnings_opt
                }
            },
            (_, None) => {
                bail!(
                    "Cannot run extended checks, no model:\n\n{}",
                    warnings_opt.unwrap_or_else(|| "No compiler warnings".to_string())
                );
            },
        };
        Ok((compiled_script, warnings_opt))
    }

    fn publish_module(
        &mut self,
        module: CompiledModule,
        mut named_addr_opt: Option<Identifier>,
        _gas_budget: Option<u64>,
        extra_args: Self::ExtraPublishArgs,
    ) -> Result<(Option<String>, CompiledModule)> {
        // TODO: hack to allow the signer to be overridden.
        // See if we can implement it in a cleaner way.
        let address = match extra_args.override_signer {
            Some(addr) => {
                if let ParsedAddress::Named(named_addr) = &addr {
                    named_addr_opt = Some(Identifier::new(named_addr.clone()).unwrap())
                }
                self.compiled_state().resolve_address(&addr)
            },
            None => *module.self_id().address(),
        };
        let module_id = ModuleId::new(address, module.self_id().name().to_owned());

        let mut module_blob = vec![];
        module.serialize(&mut module_blob).unwrap();

        // TODO: Do we still need this?
        let _private_key = match (extra_args.private_key, named_addr_opt) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, Some(named_addr)) => match self.private_key_mapping.get(named_addr.as_str()) {
                Some(private_key) => private_key.clone(),
                None => panic_missing_private_key_named("publish", named_addr.as_str()),
            },
            (None, None) => panic_missing_private_key("publish"),
        };

        // TODO: HACK! This allows us to publish a module without any checks and bypassing publishing
        //  through native context. Implement in a cleaner way, and simply run the bytecode verifier
        //  for now.
        verify_module(&module)?;
        self.storage.add_module_blob(&module_id, module_blob)?;
        Ok((None, module))
    }

    fn execute_script(
        &mut self,
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        signers: Vec<ParsedAddress>,
        txn_args: Vec<MoveValue>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> Result<Option<String>> {
        let signer0 = self.compiled_state().resolve_address(&signers[0]);

        if gas_budget.is_some() {
            panic!("Cannot set gas budget for script.")
        }
        if extra_args.gas_unit_price.is_some() {
            panic!("Cannot set gas price for script.")
        }
        if extra_args.expiration_time.is_some() {
            panic!("Cannot set expiration time for script.")
        }
        if extra_args.secondary_signers.is_some() {
            panic!("Cannot set secondary signers for script.")
        }

        let private_key = match (extra_args.private_key, &signers[0]) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, ParsedAddress::Named(named_addr)) => {
                match self.private_key_mapping.get(named_addr) {
                    Some(private_key) => private_key.clone(),
                    None => panic_missing_private_key_named("run", named_addr.as_str()),
                }
            },
            (None, ParsedAddress::Numerical(_)) => panic_missing_private_key("run"),
        };

        let mut script_blob = vec![];
        script.serialize(&mut script_blob)?;

        let params = self.fetch_transaction_parameters(
            &signer0,
            extra_args.sequence_number,
            None,
            None,
            None,
        )?;

        let txn = RawTransaction::new_script(
            signer0,
            params.sequence_number,
            TransactionScript::new(
                script_blob,
                type_args,
                txn_args
                    .into_iter()
                    .map(|arg| TransactionArgument::try_from(arg).unwrap())
                    .collect(),
            ),
            params.max_gas_amount,
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign(&private_key, Ed25519PublicKey::from(&private_key))
        .unwrap()
        .into_inner();

        let output = self.run_transaction(Transaction::UserTransaction(txn))?;

        let output = if extra_args.show_events {
            render_events(output.events())
        } else {
            None
        };
        Ok(output)
    }

    fn call_function(
        &mut self,
        module: &ModuleId,
        function: &IdentStr,
        type_args: Vec<TypeTag>,
        signers: Vec<ParsedAddress>,
        txn_args: Vec<MoveValue>,
        gas_budget: Option<u64>,
        extra_args: Self::ExtraRunArgs,
    ) -> Result<(Option<String>, SerializedReturnValues)> {
        if extra_args.script {
            panic!("Entry functions are not supported.")
        }

        if signers.len() != 1 {
            panic!("Expected 1 signer, got {}.", signers.len());
        }
        let signer = self.compiled_state().resolve_address(&signers[0]);

        let private_key = match (extra_args.private_key, &signers[0]) {
            (Some(private_key), _) => self.resolve_private_key(&private_key),
            (None, ParsedAddress::Named(named_addr)) => {
                match self.private_key_mapping.get(named_addr) {
                    Some(private_key) => private_key.clone(),
                    None => panic_missing_private_key_named("run", named_addr.as_str()),
                }
            },
            (None, ParsedAddress::Numerical(_)) => panic_missing_private_key("run"),
        };

        let params = self.fetch_transaction_parameters(
            &signer,
            extra_args.sequence_number,
            extra_args.expiration_time,
            extra_args.gas_unit_price,
            gas_budget,
        )?;
        let txn = RawTransaction::new_entry_function(
            signer,
            params.sequence_number,
            TransactionEntryFunction::new(
                module.clone(),
                function.to_owned(),
                type_args,
                convert_txn_args(
                    &txn_args
                        .into_iter()
                        .map(|arg| TransactionArgument::try_from(arg).unwrap())
                        .collect::<Vec<_>>(),
                ),
            ),
            params.max_gas_amount,
            params.gas_unit_price,
            params.expiration_timestamp_secs,
            ChainId::test(),
        );

        let txn = match &extra_args.secondary_signers {
            Some(secondary_signers) => {
                let (secondary_signers, secondary_private_keys) =
                    self.resolve_secondary_signers(secondary_signers);

                txn.sign_multi_agent(
                    &private_key,
                    secondary_signers,
                    secondary_private_keys.iter().collect(),
                )?
                .into_inner()
            },
            None => txn
                .sign(&private_key, Ed25519PublicKey::from(&private_key))?
                .into_inner(),
        };

        let output = self.run_transaction(Transaction::UserTransaction(txn))?;

        let output = if extra_args.show_events {
            render_events(output.events())
        } else {
            None
        };

        //TODO: replace this dummy value with actual txn return value
        let a = SerializedReturnValues {
            mutable_reference_outputs: vec![(0, vec![0], MoveTypeLayout::U8)],
            return_values: vec![(vec![0], MoveTypeLayout::U8)],
        };
        Ok((output, a))
    }

    fn view_data(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        resource: &IdentStr,
        type_args: Vec<TypeTag>,
    ) -> Result<String> {
        let struct_tag = StructTag {
            address: *module.address(),
            module: module.name().to_owned(),
            name: resource.to_owned(),
            type_args,
        };
        let state_key = StateKey::resource(&address, &struct_tag)?;
        match self.storage.get_state_value_bytes(&state_key).unwrap() {
            None => Ok("[No Resource Exists]".to_owned()),
            Some(data) => {
                let annotated =
                    CedraValueAnnotator::new(&self.storage).view_resource(&struct_tag, &data)?;
                Ok(format!("{}", annotated))
            },
        }
    }

    fn handle_subcommand(&mut self, input: TaskInput<Self::Subcommand>) -> Result<Option<String>> {
        match input.command {
            CedraSubCommand::BlockCommand(block_cmd) => {
                let proposer = self.compiled_state().resolve_address(&block_cmd.proposer);
                let metadata = BlockMetadata::new(
                    HashValue::zero(),
                    0,
                    block_cmd.time,
                    proposer,
                    vec![],
                    vec![],
                    block_cmd.time,
                );

                let output = self.run_transaction(Transaction::BlockMetadata(metadata))?;

                Ok(render_events(output.events()))
            },
            CedraSubCommand::ViewTableCommand(view_table_cmd) => {
                let converter = self.storage.as_converter(Arc::new(FakeDbReader {}), None);

                let vm_key = converter
                    .try_into_vm_value(&view_table_cmd.key_type, view_table_cmd.key_value)
                    .unwrap();
                let raw_key = vm_key.undecorate().simple_serialize().unwrap();

                let state_key =
                    StateKey::table_item(&TableHandle(view_table_cmd.table_handle), &raw_key);

                let bytes = self
                    .storage
                    .get_state_value_bytes(&state_key)
                    .unwrap()
                    .ok_or_else(|| format_err!("Failed to fetch table item.",))?;

                let move_value =
                    converter.try_into_move_value(&view_table_cmd.value_type, &bytes)?;

                Ok(Some(serde_json::to_string(&move_value).unwrap()))
            },
        }
    }
}

/**
 * Misc
 */

struct PrettyEvent<'a>(&'a ContractEvent);

impl<'a> fmt::Display for PrettyEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        match self.0 {
            ContractEvent::V1(v1) => {
                writeln!(f, "    key:     {}", v1.key())?;
                writeln!(f, "    seq_num: {}", v1.sequence_number())?;
            },
            ContractEvent::V2(_v2) => (),
        }
        writeln!(f, "    type:    {}", self.0.type_tag())?;
        writeln!(f, "    data:    {:?}", hex::encode(self.0.event_data()))?;
        write!(f, "}}")
    }
}

struct PrettyEvents<'a>(&'a [ContractEvent]);

impl<'a> fmt::Display for PrettyEvents<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Events:")?;
        for event in self.0.iter() {
            writeln!(f)?;
            write!(f, "{}", PrettyEvent(event))?;
        }
        Ok(())
    }
}

fn render_events(events: &[ContractEvent]) -> Option<String> {
    if events.is_empty() {
        None
    } else {
        Some(format!("{}", PrettyEvents(events)))
    }
}

fn precompiled_v2_stdlib() -> &'static PrecompiledFilesModules {
    &PRECOMPILED_cedra_framework_V2
}

pub fn run_cedra_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_cedra_test_with_config(path, TestRunConfig::CompilerV2 {
        language_version: LanguageVersion::default(),
        experiments: vec![("attach-compiled-module".to_owned(), true)],
    })
}

pub fn run_cedra_test_with_config(
    path: &Path,
    config: TestRunConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let suffix = Some(EXP_EXT.to_owned());
    let v2_lib = precompiled_v2_stdlib();
    set_paranoid_type_checks(true);
    run_test_impl::<CedraTestAdapter>(config, path, v2_lib, &suffix)
}
