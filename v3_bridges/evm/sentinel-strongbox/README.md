### :wrench: Building

This strongbox library needs to target `aarch64-linux-android`. If you try and build it for another target you will get a compilation error telling you to target android instead.

You also need to tell the compiler where to find the android-specific clang compiler, and the llvm archiver, via two environment variables: `TARGET_CC` and `TARGET_AR`, otherwise you will run into compilation issues due to the `ring` crate dependency.

Those two items are part of the the android native development kit (the NDK), whose location will also need to be set correctly under the `$NDK_HOME` environment variable.

With all of that in place, you will be able to successfully compile this library (in this directory) thusly:

```
TARGET_CC="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang" \
TARGET_AR="$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar" \
cargo build --release \
--target=aarch64-linux-android
```

:radioactive: __IMPORTANT:__ If the above script fails to build the binary, it will likewise fail to be built by `react-native`, so make sure the script is working first!
