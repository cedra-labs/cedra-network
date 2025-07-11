// Copyright © Cedra Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

pub mod command_adapter;
pub mod local_fs;

#[cfg(test)]
mod test_util;
#[cfg(test)]
mod tests;

use crate::storage::{
    command_adapter::{CommandAdapter, CommandAdapterOpt},
    local_fs::{LocalFs, LocalFsOpt},
};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use clap::{ArgGroup, Parser};
use once_cell::sync::Lazy;
#[cfg(test)]
use proptest::prelude::*;
use regex::Regex;
#[cfg(test)]
use std::convert::TryInto;
use std::{convert::TryFrom, ops::Deref, str::FromStr, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};

/// String returned by a specific storage implementation to identify a backup, probably a folder name
/// which is exactly the same with the backup name we pass into `create_backup()`
/// This is created and returned by the storage when `create_backup()`, passed back to the storage
/// when `create_for_write()` and persisted nowhere (once a backup is created, files are referred to
/// by `FileHandle`s).
pub type BackupHandle = String;
pub type BackupHandleRef = str;

/// URI pointing to a file in a backup storage, like "s3:///bucket/path/file".
/// These are created by the storage when `create_for_write()`, stored in manifests by the backup
/// controller, and passed back to the storage when `open_for_read()` by the restore controller
/// to retrieve a file referred to in the manifest.
pub type FileHandle = String;
pub type FileHandleRef = str;

/// Through this, the backup controller promises to the storage the names passed to
/// `create_backup()` and `create_for_write()` don't contain funny characters tricky to deal with
/// in shell commands.
/// Specifically, names follow the pattern "\A[a-zA-Z0-9][a-zA-Z0-9._-]{2,126}\z"
#[cfg_attr(test, derive(Hash, Eq, PartialEq))]
#[derive(Debug)]
pub struct ShellSafeName(String);

impl ShellSafeName {
    const PATTERN: &'static str = r"\A[a-zA-Z0-9][a-zA-Z0-9._-]{2,126}\z";

    fn sanitize(name: &str) -> Result<()> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(ShellSafeName::PATTERN).unwrap());
        ensure!(RE.is_match(name), "Illegal name: {}", name,);
        Ok(())
    }
}

impl TryFrom<String> for ShellSafeName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Self::sanitize(&value).map(|_| Self(value))
    }
}

impl FromStr for ShellSafeName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::sanitize(s).map(|_| Self(s.to_string()))
    }
}

impl Deref for ShellSafeName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ShellSafeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Arbitrary for ShellSafeName {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (&ShellSafeName::PATTERN[2..(ShellSafeName::PATTERN.len() - 2)]) // remove \A and \z
            .prop_map(|s| s.try_into().unwrap())
            .boxed()
    }
}

#[cfg_attr(test, derive(Debug, Hash, Eq, Ord, PartialEq, PartialOrd))]
#[derive(Clone)]
pub struct TextLine(String);

impl TextLine {
    pub fn new(value: &str) -> Result<Self> {
        let newlines: &[_] = &['\n', '\r'];
        ensure!(value.find(newlines).is_none(), "Newline not allowed.");
        let mut ret = value.to_string();
        ret.push('\n');
        Ok(Self(ret))
    }
}

impl AsRef<str> for TextLine {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Arbitrary for TextLine {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        ("[^\r\n]{0,1024}")
            .prop_map(|s| TextLine::new(&s).unwrap())
            .boxed()
    }
}

#[async_trait]
pub trait BackupStorage: Send + Sync {
    /// Hint that a bunch of files are gonna be created related to a backup identified by `name`,
    /// which is unique to the content of the backup, i.e. it won't be the same name unless you are
    /// backing up exactly the same thing.
    /// Storage can choose to take actions like create a dedicated folder or do nothing.
    /// Returns a string to identify this operation in potential succeeding file creation requests.
    async fn create_backup(&self, name: &ShellSafeName) -> Result<BackupHandle>;
    /// Ask to create a file for write, `backup_handle` was returned by `create_backup` to identify
    /// the current backup.
    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &ShellSafeName,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)>;
    /// Open file for reading.
    async fn open_for_read(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>>;
    /// Asks to save a metadata entry and return the File handle of the saved file.
    /// A metadata entry is one line of text.
    /// The backup system doesn't expect a metadata entry to exclusively map to a single file
    /// handle, or the same file handle when accessed later, so there's no need to return one. This
    /// also means a local cache must download each metadata file from remote at least once, to
    /// uncover potential storage glitch sooner.
    /// Behavior on duplicated names is undefined, overwriting the content upon an existing name
    /// is straightforward and acceptable.
    /// See `list_metadata_files`.
    async fn save_metadata_line(
        &self,
        name: &ShellSafeName,
        content: &TextLine,
    ) -> Result<FileHandle> {
        self.save_metadata_lines(name, &[content.clone()]).await
    }
    /// The backup system always asks for all metadata files and cache and build index on top of
    /// the content of them. This means:
    ///   1. The storage is free to reorganise the metadata files, like combining multiple ones to
    /// reduce fragmentation.
    ///   2. But the cache does expect the content stays the same for a file handle, so when
    /// reorganising metadata files, give them new unique names.
    async fn list_metadata_files(&self) -> Result<Vec<FileHandle>>;
    /// Move a metadata file to the metadata file backup folder.
    async fn backup_metadata_file(&self, file_handle: &FileHandleRef) -> Result<()>;
    /// Save a vector of metadata lines to file and return the file handle of saved file.
    /// If the file exists, this will overwrite
    async fn save_metadata_lines(
        &self,
        name: &ShellSafeName,
        lines: &[TextLine],
    ) -> Result<FileHandle>;
}

#[derive(Parser)]
pub enum StorageOpt {
    #[clap(about = "Select the LocalFs backup storage type, which is used mainly for tests.")]
    LocalFs(LocalFsOpt),
    #[clap(
        about = "Select the CommandAdapter backup storage type, which reads shell commands with which \
    it communicates with either a local file system or a remote cloud storage. Compression or other \
    filters can be added as part of the commands. See a sample config here: \
    https://github.com/cedra-labs/cedra-network/tree/main/storage/backup/backup-cli/src/storage/command_adapter/sample_configs/"
    )]
    CommandAdapter(CommandAdapterOpt),
}

impl StorageOpt {
    pub async fn init_storage(self) -> Result<Arc<dyn BackupStorage>> {
        Ok(match self {
            StorageOpt::LocalFs(opt) => Arc::new(LocalFs::new_with_opt(opt)),
            StorageOpt::CommandAdapter(opt) => Arc::new(CommandAdapter::new_with_opt(opt).await?),
        })
    }
}

#[derive(Parser, Clone, Debug)]
#[clap(group(
    ArgGroup::new("storage")
    .required(true)
    .args(&["local_fs_dir", "command_adapter_config"]),
))]
pub struct DBToolStorageOpt {
    #[clap(
        long,
        help = "Select the LocalFs backup storage type, which is used mainly for tests."
    )]
    local_fs_dir: Option<LocalFsOpt>,
    #[clap(
        long,
        help = "Select the CommandAdapter backup storage type, which reads shell commands with which \
    it communicates with either a local file system or a remote cloud storage. Compression or other \
    filters can be added as part of the commands. See a sample config here: \
    https://github.com/cedra-labs/cedra-network/tree/main/testnet/backups "
    )]
    command_adapter_config: Option<CommandAdapterOpt>,
}

impl DBToolStorageOpt {
    pub async fn init_storage(self) -> Result<Arc<dyn BackupStorage>> {
        Ok(if self.local_fs_dir.is_some() {
            Arc::new(LocalFs::new_with_opt(self.local_fs_dir.unwrap()))
        } else {
            Arc::new(CommandAdapter::new_with_opt(self.command_adapter_config.unwrap()).await?)
        })
    }
}
