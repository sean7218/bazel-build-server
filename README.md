# Bazel SourceKit BSP

A Build Server Protocol (BSP) implementation that enables SourceKit LSP integration with Bazel-based Swift projects.

## Overview

This project bridges the gap between Bazel build systems and SourceKit LSP, providing full IDE functionality (syntax highlighting, code completion, jump to definition, error diagnostics) for Swift projects built with Bazel.

**Key Features:**
- 🚀 Real-time IDE features for Bazel Swift projects
- 📋 Build Server Protocol 2.0 compliance
- ⚡ Intelligent caching of Bazel query results
- 📝 Comprehensive logging and debugging support
- 🔄 Background target refresh for optimal performance

## Prerequisites

- macOS 13.0 or later
- Swift 6.1 or later
- Bazel 6.0 or later
- Visual Studio Code with Swift extension (recommended)

## Quick Start

### 1. Build the Server

```bash
# Clone and build
git clone <repository-url>
cd bazel-build-server
swift build --configuration release

# Or use the Makefile
make release
```

### 2. Configure Your Project

Create a `buildServer.json` in your Bazel workspace root:

```json
{
  "name": "bazel-sourcekit-bsp",
  "argv": ["/path/to/bazel-build-server"],
  "version": "1.0.0",
  "bspVersion": "2.0.0",
  "languages": ["swift"],
  "targets": ["//YourApp:YourApp"],
  "indexDatabasePath": "/path/to/your/project/.index-db",
  "aqueryArgs": []
}
```

### 3. IDE Setup (VS Code)

1. Open your Bazel workspace in VS Code
2. Install the Swift extension
3. The build server will automatically be detected via `buildServer.json`

## Configuration

### buildServer.json Options

| Field | Description | Example |
|-------|-------------|---------|
| `argv` | Path to the build server executable | `["/usr/local/bin/bazel-build-server"]` |
| `targets` | Bazel targets to track | `["//App:App", "//Tests:Tests"]` |
| `indexDatabasePath` | SourceKit index database location | `"/workspace/.index-db"` |
| `aqueryArgs` | Additional Bazel aquery arguments | `["--output=jsonproto"]` |

### Installation Options

**Option 1: System-wide Installation**
```bash
make install  # Installs to /usr/local/bin
```

**Option 2: Local Development**
```bash
make test-harness  # Copies binary to TestHarness directory
```

## Development

### Building and Testing

```bash
# Development build
swift build

# Run tests
swift test

# Clean build artifacts
swift package clean
```

### Test Harness

The `TestHarness/` directory contains a complete example Bazel workspace:

```bash
cd TestHarness
code .  # Open in VS Code to test the integration
```

### Debugging

Logs are written to:
- **Main log**: `~/bazel-build-server.log`
- **Activity log**: `~/.bazel-sourcekit-bsp/activity.log`

Set log level in the build server for verbose output.

## Architecture

The build server implements a clean separation of concerns:

- **JSON-RPC Layer**: Handles LSP communication protocol
- **Request Processing**: Manages BSP method implementations  
- **Bazel Integration**: Executes and caches Bazel queries
- **Target Management**: Thread-safe target tracking and updates

## Troubleshooting

**Build server not starting:**
- Verify the `argv` path in `buildServer.json` is correct
- Check file permissions on the executable
- Review logs at `~/bazel-build-server.log`

**No IDE features working:**
- Ensure your Bazel targets build successfully
- Verify `targets` array in `buildServer.json` matches your project
- Check that the index database path is writable

**Performance issues:**
- Review `aqueryArgs` for optimization opportunities
- Monitor cache hit rates in activity logs
- Consider reducing the number of tracked targets

## VS Code Extensions

Recommended extensions for the best experience:

- **[Swift](https://marketplace.visualstudio.com/items?itemName=sswg.swift-lang)** - Core Swift language support
- **[SwiftFormat](https://github.com/nicklockwood/SwiftFormat)** - Code formatting on save
- **[Error Lens](https://github.com/usernamehw/vscode-error-lens)** - Inline error display

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass with `swift test`
5. Submit a pull request

## License

See [LICENSE](LICENSE) for details.
