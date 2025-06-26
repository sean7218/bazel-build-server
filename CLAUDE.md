# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bazel Build Server implementation that provides the Build Server Protocol (BSP) for Bazel projects, primarily designed to work with SourceKit-LSP for Swift development. The server acts as a bridge between Bazel and language servers, providing compiler arguments, source files, and build targets.

## Core Architecture

The codebase is structured as a Rust application with the following key modules:

- **`main.rs`**: Entry point with JSON-RPC request handling loop and the `RequestHandler` struct
- **`aquery/`**: Bazel query functionality to extract build information and compiler arguments  
- **`json_rpc/`**: JSON-RPC protocol implementation for BSP communication
- **`messages/`**: BSP message types and request/response structures
- **`support_types/`**: Core types including `BuildServerConfig` and `BuildTarget`
- **`utils/`**: Logging utilities with custom `log_str!` macro

The server reads configuration from `buildServer.json` files and uses `bazel aquery` to extract compiler arguments and source files for language server integration.

## Development Commands

### Building
```bash
cargo build          # Debug build (executable at target/debug/buildserver)
cargo build --release  # Release build (executable at target/release/buildserver)
```

### Testing
Tests are currently commented out but can be run with:
```bash
cargo test
```

### Configuration Setup
The server requires a `buildServer.json` configuration file at the project root with fields like:
- `target`: Bazel target to query (e.g., "//App:App")
- `sdk`: Path to SDK for `__BAZEL_XCODE_SDKROOT__` replacement
- `indexStorePath`: Location of Bazel's global index store
- `indexDatabasePath`: Output location for index.db
- `aqueryArgs`: Additional arguments for bazel aquery command

### Bazel Requirements
Projects using this build server need `.bazelrc` configuration:
```
build --features swift.use_global_index_store
build --features swift.index_while_building
```

## Key Implementation Details

- **JSON-RPC Communication**: Server reads from stdin and writes to stdout following LSP/BSP protocol
- **Bazel Integration**: Uses `bazel aquery` to extract compiler arguments and source file information
- **Logging**: Custom logging to `~/.sourcekit-bsp/bsp.log` via the `log_str!` macro
- **Configuration Parsing**: `BuildServerConfig::parse()` reads and validates `buildServer.json`
- **Target Resolution**: Converts Bazel targets to BSP `BuildTarget` format with compiler capabilities

## BSP Method Support

The server implements these BSP methods:
- `build/initialize` - Server initialization with capabilities
- `workspace/buildTargets` - Get available build targets  
- `buildTarget/sources` - Get source files for targets
- `textDocument/sourceKitOptions` - Get compiler arguments for files
- `textDocument/registerForChanges` - Legacy push-based file change notifications
- `buildTarget/prepare` - Execute bazel build for targets