<a href="https://cedra.network">
	<img width="30%" src="https://cedra.network/images/logo.svg" alt="Cedra Banner" />
</a>

---

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
<!-- [![Lint+Test](https://github.com/cedra-labs/cedra-network/actions/workflows/lint-test.yaml/badge.svg)](https://github.com/cedra-labs/cedra-network/actions/workflows/lint-test.yaml) -->

Cedra is the first community-owned blockchain built on the Move language that lets anyone spin up and govern their own sovereign networks. Designed as a public good, Cedra fuses protocol development, funding, and growth into open collaboration among core contributors, a non-profit foundation, and a worldwide guild of builders.

This main repository contains the core components of the Cedra Network ecosystem, including the CLI tools, node, and various modules that power the network.

## Getting Started
Before you begin, ensure you have the following installed:

- **Rust 1.86+** - Required for building from source
- **Operating System**: macOS, Ubuntu 22.04+, or Windows

### Installation

The quickest way to get started with Cedra is by installing the CLI:

**Ubuntu/Debian:**
```bash
sudo add-apt-repository ppa:cedra-network/deps
sudo apt update
sudo apt install cedra-cli
```

**Windows (Chocolatey - recommended):**
```bash
choco install cedra
```

Once installed, verify:
```bash
cedra --version
```

**macOS and any other OS**
1. Visit the Cedra CLI v1.0.1 release page: https://github.com/cedra-labs/cedra-network/releases/tag/cedra-cli-v1.0.1.
2. In the Assets section, choose the file that matches your platform
3. Extract the archive
4. Move the `cedra` (or `cedra.exe` on Windows) executable to a folder that is in your `PATH`:

For detailed installation instructions and troubleshooting, see the [CLI Installation Guide](https://docs.cedra.network/getting-started/cli).

**Build from source:**

If you prefer compiling yourself or contributing to Cedra:

```bash
git clone https://github.com/cedra-labs/cedra-network
cd cedra-network
cargo build --release -p cedra
```

The compiled binary will be at `target/release/cedra` (or `.exe` on Windows). Add it to your PATH and run `cedra --version` to confirm.

For more detailed build instructions and development setup, see our [Development Setup Guide](https://docs.cedra.network/getting-started/libs)

### Resources

- [Documentation](https://docs.cedra.network/)
- [Block Explorer](https://cedrascan.com)
- [DApp Examples](https://docs.cedra.network/real-world-guides)
- [X (Twitter)](https://x.com/cedranetwork)
- [Telegram Network Channel](https://t.me/cedranetwork)
- [Telegram Builders Group](https://t.me/+Ba3QXd0VG9U0Mzky)
- [Discord Server](https://discord.com/invite/cedranetwork)


## Contributing

You can learn more about contributing to the Cedra project by reading our [Contribution Guide](https://github.com/cedra-labs/cedra-network/blob/main/CONTRIBUTING.md) and by viewing our [Code of Conduct](https://github.com/cedra-labs/cedra-network/blob/main/CODE_OF_CONDUCT.md).

Cedra Core is licensed under [Apache 2.0](https://github.com/cedra-labs/cedra-network/blob/main/LICENSE).
