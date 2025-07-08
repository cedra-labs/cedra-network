# The Cedra Framework Repo

This repository serves as a mirror for the Cedra Framework packages, including the Move standard library. The contents are synced from [cedra-core](https://github.com/cedra-labs/cedra) on an hourly basis.

By pulling dependencies from this mirror repository, developers can avoid downloading unnecessary data, reducing build time significantly.

## Usage
To use the packages in this repository as dependencies in your Move project, you can include them in your move.toml file by adding them as Git dependencies.

To add a dependency from this repository, include the following in your `move.toml` file:
```
[dependencies]
<package_name> = { git = "https://github.com/cedra-labs/cedra-framework.git", subdir = "<path_to_directory_containing_Move.toml>", rev = "<commit_hash_or_branch_name>" }
```
For example, to add `CedraFramework` from the `mainnet` branch, you would use:
```
CedraFramework = { git = "https://github.com/cedra-labs/cedra-framework.git", subdir = "cedra-framework", rev = "mainnet" }
```
Make sure to replace `subdir` with the appropriate path if you are referencing a different package within the framework.

## Contributing
If you want to contribute to the development of the framework, please submit issues and pull requests to the [cedra-core](https://github.com/cedra-labs/cedra) repository, where active development happens.

Bugs, feature requests, or discussions of enhancements will be tracked in the issue section there as well. This repository is a mirror, and issues will not be tracked here.
