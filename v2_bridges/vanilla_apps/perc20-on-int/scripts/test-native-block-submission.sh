#!/bin/bash
set -e
cd "$(dirname -- $0)"

. ./get-binary-name.sh

BINARY_NAME=$(getBinaryName)
BINARY_PATH="../../../../target/release/$BINARY_NAME"

echo [+] Testing NATIVE block submission to \'$BINARY_NAME\' core...

../../scripts/clean-up.sh $BINARY_NAME

getCoreLatestNativeBlockNumber() {
	$BINARY_PATH getEnclaveState | jq .eth.eth_latest_block_number
}

getExpectedBlockNumber() {
	cat ./eth-subsequent-block-3.json | jq .block.number
}

./initialize-eth.sh
./initialize-int.sh

if [[ $(getCoreLatestNativeBlockNumber) == null ]];then
	../../scripts/clean-up.sh $BINARY_NAME
	echo [-] Something went wrong with core initalization!
	exit 1
fi

# Let's submit our sample blocks...
$BINARY_PATH submitEthBlock --file=./eth-subsequent-block-1.json
$BINARY_PATH submitEthBlock --file=./eth-subsequent-block-2.json
$BINARY_PATH submitEthBlock --file=./eth-subsequent-block-3.json

[[ $(getCoreLatestNativeBlockNumber) == $(getExpectedBlockNumber) ]] && result=true || result=false

../../scripts/clean-up.sh $BINARY_NAME

if [[ $result == true ]]; then
	echo [+] NATIVE block submission test to \'$BINARY_NAME\' core test PASSED!
else
	echo [-] NATIVE block submission test to \'$BINARY_NAME\' core test FAILED!
	exit 1
fi
