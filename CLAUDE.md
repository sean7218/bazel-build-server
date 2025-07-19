# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bazel Build Server Protocol (BSP) implementation for Swift and iOS projects that integrates with SourceKit LSP. It allows SourceKit to work with Bazel-based Swift projects, providing IDE features like syntax highlighting, jump to definition, auto-complete, etc.

## Build Commands

### Swift Package Manager Commands
- `swift build` - Build the project in debug mode
- `swift build --configuration release` - Build release version
- `swift run bazel-build-server` - Run the build server executable
- `swift test` - Run all tests using Swift Testing framework
- `swift package clean` - Clean build artifacts

### Makefile Commands
- `make build` - Build debug version (equivalent to `swift build`)
- `make release` - Build release version
- `make run` - Build and run the executable
- `make clean` - Clean build artifacts
- `make install` - Install release binary to `/usr/local/bin`
- `make test-harness` - Build and copy binary to TestHarness directory

### Bazel Commands (in TestHarness directory)
- `bazel build //App:App` - Build the test app
- `bazel test //...` - Run all Bazel tests
- `bazel query //...` - Query all targets

## Architecture

### Core Components

**Main Executable (`Sources/BazelBuildServer/`)**
- `BazelBuildServer.swift` - Entry point with CLI argument parsing and logging setup

**Build Server Library (`Sources/BazelBuildServerLib/`)**
- `BuildServer.swift` - JSON-RPC communication handler, reads from stdin/stdout
- `RequestHandler.swift` - Processes BSP requests (initialize, compile, etc.)
- `BSPTypes.swift` - Build Server Protocol type definitions
- `StreamLogHandler.swift` - Custom logging implementation

**Action Query System (`Sources/ActionQuery/`)**
- `ActionQuery.swift` - Executes Bazel aquery commands with caching
- `BazelTarget.swift` - Represents Bazel build targets
- `QueryResult.swift` - Parses Bazel query output

**Supporting Libraries**
- `Sources/BSPError/` - Error handling types
- `Sources/ShellCommand/` - Shell command execution utilities
- `Sources/QueryParser/` - Protobuf-based query parsing

### Key Patterns

1. **JSON-RPC Communication**: The build server communicates via JSON-RPC over stdin/stdout following LSP patterns
2. **Caching Strategy**: Bazel query results are cached in `~/.bazel-sourcekit-bsp/` with background refresh
3. **Concurrent Processing**: Uses DispatchQueue for thread-safe target management
4. **Structured Logging**: Dual logging to file (`~/bazel-build-server.log`) and activity log (`~/.bazel-sourcekit-bsp/activity.log`)

### Configuration

The build server is configured via `buildServer.json` files that specify:
- Executable path and arguments
- Target Bazel targets to track
- Index database path
- Additional aquery arguments

## Test Harness

The `TestHarness/` directory contains a complete Bazel workspace for testing:
- iOS app target (`//App:App`)
- Various Swift libraries with different dependency patterns
- External dependencies managed via `MODULE.bazel`
- Custom BUILD.bazel files for third-party dependencies

## Testing

Tests use the Swift Testing framework (not XCTest). Key test targets:
- `ActionQueryTests` - Tests query parsing with JSON/text fixtures
- `ShellCommandTests` - Tests shell command execution
- `QueryParserTests` - Tests protobuf query parsing

Run tests with `swift test` or target specific tests like `swift test --filter ActionQueryTests`.