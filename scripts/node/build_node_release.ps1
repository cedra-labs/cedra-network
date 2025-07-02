# Copyright © Cedra Foundation
# SPDX-License-Identifier: Apache-2.0

###########################################
# Build and package a release for the Node #
###########################################

# Note: This must be run from the root of the network repository.

param (
    [string]$VERSION_ARG=""
)

# Set up basic variables.
$NAME="cedra-node"
$CARGO_PATH="$NAME\Cargo.toml"
$Env:VCPKG_ROOT = 'D:\vcpkg\'



# Get the version of the Node from its Cargo.toml.
$VERSION = Get-Content $CARGO_PATH | Select-String -Pattern '^\w*version = "(\d*\.\d*.\d*)(-main)?"' | % {"$($_.matches.groups[1])"}
if ( $VERSION -eq "0.0.0" -or $VERSION -eq "0.0.0-main") {
    if  ( $VERSION_ARG -eq "" ) {
        echo "invalid node version"
        exit 1
    }

    $VERSION = $VERSION_ARG
}

echo "node version is:"
echo "$VERSION"

# Install the developer tools
echo "Installing developer tools"
PowerShell -ExecutionPolicy Bypass -File scripts/windows_dev_setup.ps1

# Note: This is required to bypass openssl isssue on Windows.
echo "Installing OpenSSL"
vcpkg install openssl:x64-windows-static-md --clean-after-build

# Build the Node.
echo "Building release $VERSION of $NAME for Windows"
cargo build -p $NAME --profile node

# Compress the Node.
$ZIP_NAME="$NAME-$VERSION-Windows-x86_64.zip"
echo "Compressing Node to $ZIP_NAME"
Compress-Archive -Path target\node\$NAME.exe -DestinationPath $ZIP_NAME

