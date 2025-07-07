# Bazel Build Server

Bazel implemention for [build server protocol](https://build-server-protocol.github.io/) and mainly working with [sourcekit-lsp](https://github.com/swiftlang/sourcekit-lsp).

![Screen Recording 2025-05-26 at 9 10 28 AM](https://github.com/user-attachments/assets/a5d7e248-9f5a-4149-bfe3-065a592d5fba)

## Requirements

- Xcode 16.3
- Swift toolchain 6.1.0
- Enable global_index_store in .bazelrc
- Disable `build --features=swift.use_explicit_swift_module_map`

## Getting Started

1. Creating a buildServer.json at the root of your project shown below.
   - The `argv` need to point at the buildserver executable
   - The `target` will be used by `aquery` to get all the compiler arguments and targets.
   - The `sdk` is what will be replaced for `__BAZEL_XCODE_SDKROOT__`
   - The `indexStorePath` is the location where all indexstore files are.
   - The `indexDatabasePath` is the location for index.db output
   - The `aqueryArgs` are additional arguments you can supply to the bazel aquery command
   - The `extraIncludes` are adding to the compiler arguments with option `-I` for addtional swiftmodule search paths
   - The `extraFrameworks` are adding to the copmiler arguments with option `-F` for framework search apth

```json
{
  "name": "example-app",
  "argv": ["/Users/sean7218/bazel-build-server/target/debug/buildserver"],
  "version": "1.0.0",
  "bspVersion": "2.0.0",
  "languages": ["swift"],
  "target": "//App:App",
  "sdk": "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator18.4.sdk",
  "indexStorePath": "/Users/sean7218/bazel/buildserver/example-app/bazel-out/_global_index_store",
  "indexDatabasePath": "/Users/sean7218/bazel/buildserver/example-app/.index-db",
  "aqueryArgs": [],
  "extraIncludes": [],
  "extraFrameworks": []
}
```

2. Compile the build server by running `cargo build`, and the executable will be in `bazel-build-server/target/debug/buildserver`,
   and change the `argv` in the buildServer.json file as well.

```bash
cargo build
```

3. Before compile your project with bazel, you need to set the global index in `.bazelrc` file, then compile your project based on the target specified in the `buildServer.json` such as `bazel build //App:App`

```bash
# either this
build --features swift.use_global_index_store
build --features swift.index_while_building
# or
build --features swift.use_global_index_store
build --swiftcopt=-index-store-path
build --swiftcopt=$(OUTPUT_BASE)/indexstore
```

4. Open your project in vscode or neovim, you should be see logs both in sourcekit-lsp and bsp.log

5. Once the server is started Logging is stored in `~/.sourcekit-bsp/bsp.log`, it is recommended to open it and see any issues.

## Debugging

Here are some of the common errors:

1. `standard library failed to load`
  -  this means your path to sdk is wrong

2. `execution_root not found`
  - when build server first start, it will run `bazel info execution_root` 
  if the command fails, the build server will shutdown.

3. `👻 handle_initialize_request failed -> JsonError`
  - this usually means your buildServer.json is malformatted