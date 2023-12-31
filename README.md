# :closed_lock_with_key: pTokens Core

The Provable __pTokens__ core which manages the cross-chain conversions between native and host blockchains.

&nbsp;

## :earth_africa: Core Overview

The __pToken__ core is a set of libraries implementing light-clients for various block-chains, and a set of bridges which each contain two of those light clients, working in tandem to move assets from one to the other.

The v1 & v2 bridges have zero network connectivity and make no network requests. They are a push-only model, requiring external tools to gather & feed them the blocks from the chains with which the bridges interact.

In order to initialize the light-clients inside the core, an initial block from each desired chain is required. These will be the only trusted blocks in the system. Thereafter, subsequent blocks pushed to the core will undergo all the usual validation checks w/r/t to that block's veracity before appending it to the small piece of chain the light client(s) in the bridges hold.

The length of these small pieces of chain held by the bridge are governed by their __`canon-to-tip`__ lengths, which lengths can also be thought of as the number of __`confirmations + 1`__ required before the bridge will sign a transaction.

Once a submitted block reaches __`canon-to-tip`__ number of blocks away from the tip of the chain, it becomes the __`canon-block`__. At this point, it is searched for any relevant peg-ins or peg-outs and any required transactions are then signed and returned to the caller in __`JSON`__ format.

Behind the __`canon-block`__ lies some configurable number of further blocks, which here after are referred to the __`tail`__. This tail allow for the light client to hold more than just the __`cannon-to-tip-length`__ number of blocks, which allows the bridge to handle larger re-organisations if the chain in   question is particularly unstable.

In order to keep the light-clients thin, blocks behind the __`tail`__  are removed. In order to do that whilst retaining the integrity of the chain, the block to be removed is first _linked_ to the initial trusted block (the __`anchor-block`__) by hashing it together with the so-called __`linker-hash`__ (where an arbitrary constant is used for the first linkage) and the block to be removed. This way the small piece of chain inside then core can always be proven to have originated from the original trusted blocks that were used to initialize the bridge.

And so thusly the brige remains synced with the each blockchain, writing relevant transactions as it does so.

## :lock_with_ink_pen: Security:

The bridges herein are designed to be imported by an application that leverages an HSM in order to implement a secure database that adheres to the __`DatabseInterface`__ as defined in __`./common/common/src/traits.rs`__.

The example applications inside the __`vanilla`__ directories for both __`v1`__ and __`v2`__ bridges implement no such protections, and are there simple as examples as to how to implement a bridge from the library modules. These should not be used in production as their databasing is not protected by any means whatsoever.

&nbsp;

## :wrench: Build

You need to ensure you have both __`clang`__ & __`llvm`__ (or later versions) installed on your system. Then, build your desired bridge like so:

__`❍ cargo build --release --package=<bridge-name-here>`__

You can see the available app names by inspecting the __`vanilla`__ directories.

#### Versions

 - __`llvm:`__ version 6.0.0 or later.
 - __`clang:`__ version 6.0.0-1ubuntu2 or later.
 - __`rustc & cargo:`__ version 1.56.0 or later.

&nbsp;

## :floppy_disk: Database Interface

The `core` implements a generic database whose interface follows:

```
pub trait DatabaseInterface {
    fn end_transaction(&self) -> Result<()>;
    fn start_transaction(&self) -> Result<()>;
    fn delete(&self, key: Bytes) -> Result<()>;
    fn get(&self, key: Bytes, data_sensitivity: Option<u8>) -> Result<Bytes>;
    fn put(&self, key: Bytes, value: Bytes, data_sensitivity: Option<u8>) -> Result<()>;
}

```

The `start_transaction` and `end_transaction` are used by the core algorithms to signal when databasing actions begin and end, allowing a consumer of the `core` to implement atomic databasing however they wish.

Further, the `sensitivity` parameter provides a way for the `core` to signal to the consumer how sensitive the data being transmitted is, giving flexibility for the `core` consumer to handle different levels of sensitive data in different ways, where `0` signifies the _least_ sensitive data, and `255` the _most_.

&nbsp;

## :label: Metadata Chain IDs

The `v2` bridges use metadata chain IDs to route peg-ins and peg-outs to their correct destinations. The byte encodings of those metadata chain IDs are as follows:

```
EthereumMainnet,  // 0x005fe7f9
EthereumRopsten,  // 0x0069c322
EthereumRinkeby,  // 0x00f34368
BitcoinMainnet,   // 0x01ec97de
BitcoinTestnet,   // 0x018afeb2
EosMainnet,       // 0x02e7261c
TelosMainnet,     // 0x028c7109
BscMainnet,       // 0x00e4b170
EosJungleTestnet, // 0x0282317f
XDaiMainnet,      // 0x00f1918e
PolygonMainnet,   // 0x0075dd4c
UltraMainnet,     // 0x02f9337d
FioMainnet,       // 0x02174f20
UltraTestnet,     // 0x02b5a4d6
EthUnknown,       // 0x00000000
BtcUnknown,       // 0x01000000
EosUnknown,       // 0x02000000
InterimChain,     // 0xffffffff
ArbitrumMainnet,  // 0x00ce98c4
LuxochainMainnet, // 0x00d5beb0
FantomMainnet,    // 0x0022af98
AlgorandMainnet,  // 0x03c38e67
PhoenixTestnet,   // 0x02a75f2c
PhoenixMainnet,   // 0x026776fa
EthereumGoerli,   // 0x00b4f6c5
EthereumSepolia,  // 0x0030d6b5
LitecoinMainnet,  // 0x01840435
```

&nbsp;

## :black_nib: Notes

- The maximum __`confs`__ possible during initialization is 255.

- There are hardcoded "safe" addresses for each chain which are used as destinations for transactions whose actual destinations are absent or malformed when being parsed from their originating transactions.

- When initializing a bridge, the merkle-roots for transactions in blocks are __NOT__ verified - only the block headers are checked. For smaller initialiazation material, feel free to provide empty arrays for the transactions. Ensure not relevant transactions took place in the blocks used to initialize the core.

- The light __BTC__ client implemented herein currently accepts only `p2sh` deposits made to addresses generated via the __`deposit-address-generator`__ run with the private-key emitted by the core upon BTC initialization.

:warning: Neither `p2pk`, `p2pkh` nor `segwit` deposit transactions are currently supported. Deposits made via such transactions will result in lost funds! :warning:

- The library follows semantic versioning specification ([SemVer](https://semver.org)).

&nbsp;

## :mag: Features

When building the vanilla apps, you can enable features like so:

__`❍ cargo b -r -p <packageName> --features=<featureName>[,<featureName>]`__

Currently supported features include:

 - __`non-validating`__ Build a bridge with block & receipt validation skipped.
 - __`disable-fees`__ Build a v1 bridge with fees disabled. Note that v2 bridges handle fees differently, and this flag doesn't exist/does nothing.

 - __`ltc`__ Build a Litecoin bridge. Note this flag only exists for:

`pbtc-on-eth` (v1 vanilla binary)
`pbtc-on-eos` (v1 vanilla binary)
`pbtc-on-int` (v2 vanilla binary)

`common/bitcoin` (library crate)
`common/safe_addresses` (library crate)

&nbsp;

## :guardsman: Tests

To run the tests simply run:

__`❍ cargo test --features='<chosen-feature>'`__

&nbsp;

## :black_nib: To Do:
