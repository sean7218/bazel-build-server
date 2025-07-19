# Bazel SourceKit BSP

A build server implementation for SourceKit LSP.

This allows you to get SourceKit working for Swift projects that use the Bazel build system.

SourceKit is a language server that provides syntax highlight, jump to definition, auto complete, etc.

## Installation

1. clone the project and run swift build, the server executable will be store in the `.build/arm64-apple-macosx/debug/bazel-build-server`
2. edit the buildServer.json under the TestHarness folder by changing the executable path
3. if you use global_index_store, be sure the change the path to the `.index-db`
4. open TestHarness folder with `code TestHarness`
5. the bazel-build-server.log is stored at the user home directory `~/bazel-build-server.log`

You can use the example buildServer.json inside the TestHarness for your project and make sure you specify the target name `//YourTarget:YourTarget`

### Recommended plugins

- [Swift format](https://github.com/nicklockwood/SwiftFormat). Formats your swift code "on save".
- [Error Lens](https://github.com/usernamehw/vscode-error-lens). Errors and warnings look similar to Xcode.
