# Example App

![new_record](https://github.com/user-attachments/assets/73dd4c3c-4ae0-4aa9-a4e6-21734966aa1e)


1. pointing the absolute path to buildserver executable in buildServer.json.
   replace `PATH_TO_YOUR_BAZEL_BUILD_SERVER_REPO` to where you clone this repo

```json
{
  "name": "example-app",
  "argv": [
    "/PATH_TO_YOUR_BAZEL_BUILD_SERVER_REPO/target/debug/buildserver"
  ],
  "version": "1.0.0",  
  "bspVersion": "2.0.0",
  "languages": ["swift"],
  "target": "//App:App",
  "sdk": "/Applications/Xcode.app/Contents/Developer/Platforms/iPhoneSimulator.platform/Developer/SDKs/iPhoneSimulator18.4.sdk"
}

```

2. run cargo build


3. open this folder in VSCode or Neovim

