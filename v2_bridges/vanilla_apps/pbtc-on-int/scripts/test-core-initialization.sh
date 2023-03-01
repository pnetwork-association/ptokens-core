#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)

echo ✔ Testing \'$BINARY_NAME\' core initialization...

../../scripts/clean-up.sh $BINARY_NAME
./initialize-int.sh
./initialize-btc.sh
../../../../target/release/$BINARY_NAME getEnclaveState
../../scripts/clean-up.sh

echo ✔ \'$BINARY_NAME\' core initialization test successful!
