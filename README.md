# SourceKit Build Server

## Getting Started

1. Creating a buildServer.json at the root of your project shown below. 
    - The `argv` need to point at the buildserver executable
    - The `target` will be used by `aquery` to get all the compiler arguments and targets. 
    - The `sdk` is what will be replaced for `__BAZEL_XCODE_SDKROOT__`

```json
{
  "name": "bazel-build-server",
  "argv": [
    "/Users/sean7218/bazel-build-server/target/debug/buildserver"
  ],
  "version": "1.0.0",  
  "bspVersion": "2.0.0",
  "languages": ["swift"],
  "target": "//Sources/Components:Components",
  "sdk": "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
}
```

2. Compile the build server by running, the executable will be in `Users/sean7218/bazel-build-server/target/debug/buildserver`, 
and change the `argv` in the buildServer.json file as well. 

```bash
cargo build
```

3. Compile your project based on the target specified in the `buildServer.json`

```bash
bazel build //Sources/Components:Components
```

4. Open your project in vscode or neovim, you should be see logs both in sourcekit-lsp and bsp.log

5. Once the server is started Logging is stored in `~/.sourcekit-bsp/bsp.log`, it is recommended to open it and see any issues.

## Debugging

todo!

## Index-Store

The default index store is stored at the root `project-root/.indexstore`, you can specify your bazel rule to output to that location.
This might be helpful to increase sourcekit-lsp performance. 

```python
swift_library(
    name = "Components",
    srcs = ["Button.swift"],
    module_name = "Components",
    visibility = ["//visibility:public"],
    deps = ["//Sources/Utils:Utils"],
    copts = [
        "-index-store-path",
        "./.indexstore",
    ],
)
```





