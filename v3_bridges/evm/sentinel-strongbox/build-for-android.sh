#!/bin/bash

# NOTE: Set `CARGO_BUILD_PROFILE` to release to get a release build, anything else will result in a debug build
cargoBuildProfile="debug"

if [ -v CARGO_BUILD_PROFILE ]
then
   cargoBuildProfile=$CARGO_BUILD_PROFILE
fi

if [ $cargoBuildProfile == "release" ]
then
   echo "building for android with release profile"
   cargoBuildProfile='--release'
else
   echo "building for android with debug profile"
   cargoBuildProfile=''
fi

cargo build $cargoBuildProfile --target=aarch64-linux-android
