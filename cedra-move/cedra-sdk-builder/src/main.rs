// Copyright © Cedra Foundation
// SPDX-License-Identifier: Apache-2.0

//! # Code generator for Move script builders
//!
//! '''bash
//! cargo run -p cedra-sdk-builder -- --help
//! '''

use clap::{Parser, ValueEnum};
use serde_generate as serdegen;
use serde_reflection::Registry;
use std::path::PathBuf;

#[derive(ValueEnum, Debug, Clone, Copy)]
enum Language {
    Rust,
    Go,
}

#[derive(Debug, Parser)]
#[clap(name = "Cedra SDK Builder", about = "Generate boilerplate Cedra SDKs")]
struct Options {
    /// Path to the directory containing ABI files in BCS encoding.
    abi_directories: Vec<PathBuf>,

    /// Language for code generation.
    #[clap(long, value_enum, ignore_case = true, default_value_t = Language::Rust)]
    language: Language,

    /// Directory where to write generated modules (otherwise print code on stdout).
    #[clap(long)]
    target_source_dir: Option<PathBuf>,

    /// Also install the cedra types described by the given YAML file, along with the BCS runtime.
    #[clap(long)]
    with_cedra_types: Option<PathBuf>,

    /// Module name for the transaction builders installed in the `target_source_dir`.
    /// * Rust crates may contain a version number, e.g. "test:1.2.0".
    /// * In Java, this is expected to be a package name, e.g. "com.test" to create Java files in `com/test`.
    /// * In Go, this is expected to be of the format "go_module/path/go_package_name",
    /// and `cedra_types` is assumed to be in "go_module/path/cedra_types".
    #[clap(long)]
    module_name: Option<String>,

    /// Optional package name (Python) or module path (Go) of the Serde and BCS runtime dependencies.
    #[clap(long)]
    serde_package_name: Option<String>,

    /// Optional version number for the `cedra_types` module (useful in Rust).
    /// If `--with-cedra-types` is passed, this will be the version of the generated `cedra_types` module.
    #[clap(long, default_value = "0.1.0")]
    cedra_version_number: String,

    /// Optional package name (Python) or module path (Go) of the `cedra_types` dependency.
    #[clap(long)]
    package_name: Option<String>,
}

fn main() {
    let options = Options::parse();
    let abis = cedra_sdk_builder::read_abis(&options.abi_directories)
        .expect("Failed to read ABI in directory");

    let install_dir = match options.target_source_dir {
        None => {
            // Nothing to install. Just print to stdout.
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            match options.language {
                Language::Rust => {
                    cedra_sdk_builder::rust::output(&mut out, &abis, /* local types */ true)
                        .unwrap()
                },
                Language::Go => {
                    cedra_sdk_builder::golang::output(
                        &mut out,
                        options.serde_package_name.clone(),
                        options.package_name.clone(),
                        options.module_name.as_deref().unwrap_or("main").to_string(),
                        &abis,
                    )
                    .unwrap();
                },
            }
            return;
        },
        Some(dir) => dir,
    };

    // Cedra types
    if let Some(registry_file) = options.with_cedra_types {
        let installer: Box<dyn serdegen::SourceInstaller<Error = Box<dyn std::error::Error>>> =
            match options.language {
                Language::Rust => Box::new(serdegen::rust::Installer::new(install_dir.clone())),
                Language::Go => Box::new(serdegen::golang::Installer::new(
                    install_dir.clone(),
                    options.serde_package_name.clone(),
                )),
            };

        let content =
            std::fs::read_to_string(registry_file).expect("registry file must be readable");
        let mut registry = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();
        // update the registry to prevent language keyword being used
        if let Language::Rust = options.language {
            cedra_sdk_builder::rust::replace_keywords(&mut registry)
        }

        let (package_name, _package_path) = match options.language {
            Language::Rust => (
                if options.cedra_version_number == "0.1.0" {
                    "cedra-types".to_string()
                } else {
                    format!("cedra-types:{}", options.cedra_version_number)
                },
                vec!["cedra-types"],
            ),
            Language::Go => ("cedratypes".to_string(), vec!["cedratypes"]),
        };

        let config = serdegen::CodeGeneratorConfig::new(package_name)
            .with_encodings(vec![serdegen::Encoding::Bcs]);

        installer.install_module(&config, &registry).unwrap();
    }

    // Transaction builders
    let installer: Box<dyn cedra_sdk_builder::SourceInstaller<Error = Box<dyn std::error::Error>>> =
        match options.language {
            Language::Rust => Box::new(cedra_sdk_builder::rust::Installer::new(
                install_dir,
                options.cedra_version_number,
            )),
            Language::Go => Box::new(cedra_sdk_builder::golang::Installer::new(
                install_dir,
                options.serde_package_name,
                options.package_name,
            )),
        };

    if let Some(ref name) = options.module_name {
        installer
            .install_transaction_builders(name, abis.as_slice())
            .unwrap();
    }
}

#[test]
fn verify_tool() {
    use clap::CommandFactory;
    Options::command().debug_assert()
}
