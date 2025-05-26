# SourceKit Build Server

## Getting Started

1. creating a buildServer.json at the root of your project such as. The target will be used by `aquery` to 
get all the compiler arguments and targets. the sdk is what will be replaced for `__BAZEL_XCODE_SDKROOT__`

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

2. compile the build server by running, the executable will be in `./target/debug/buildserver`

```
cargo build
```

3. compile your project based on the target specified in the `buildServer.json`

4. Logging is stored in `~/.sourcekit-bsp/bsp.log`, it is recommended to open it and see any issues.

5. open your project in vscode or neovim, you should be see logs both in sourcekit-lsp and bsp.log

## Debugging

todo!





