# toCKB

![tockb-workflow](https://github.com/nervosnetwork/toCKB/workflows/ci/badge.svg)

> toCKB is under active development and considered to be a work in progress.

toCKB maps assets on other blockchains to [CKB](https://github.com/nervosnetwork/ckb) in a trustless way.

At this stage, we are focusing on [Bitcoin](https://github.com/bitcoin/bitcoin) and [Ethereum](https://github.com/ethereum).

## Run it in your local dev chain

Build the contracts, run unit tests.

```bash
$ git clone --recurse-submodules https://github.com/nervosnetwork/toCKB.git
# If you clone it without `--recurse-submodules`, you can use `git submodule update` to update later.

$ cd toCKB

# build the contracts
$ capsule build --release

# You can run `cargo test` to run the unit tests for contracts.
```

Run local CKB dev chain

```bash
# ckb version used: v0.35.0
# https://github.com/nervosnetwork/ckb/releases/tag/v0.35.0
# Download on Mac: wget https://github.com/nervosnetwork/ckb/releases/download/v0.35.0/ckb_v0.35.0_x86_64-apple-darwin.zip
# unzip and add the binary path to system PATH
$ cd /path/where/you/want/to/put/ckb-data
$ ckb init -c dev -C . --ba-arg 0x5a7487f529b8b8fd4d4a57c12dc0c70f7958a196
$ ckb run -C .
$ ckb miner -C .
```

Use tockb-cli to interact with the chain.

```bash
# install tockb-cli
$ cd tockb-cli 
$ cargo install --path . -f

# prepare the private key for test
# [WARNING] NEVER USE IT IN PRODUCTION ENVIRONMENT!!!
$ mkdir privkeys
$ echo '0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc' > privkeys/0

# This account has CKB with it, you can use it for test.
$ tockb-cli account import --privkey-path ./privkeys/0
Password:
address:
  mainnet: ckb1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts6f6daz
  testnet: ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
lock_arg: c8328aabcd9b9e8e64fbc566c4385c3bdeb219d7

$ tockb-cli wallet get-capacity --address ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
total: 20000000000.0 (CKB)
```

Use tockb-cli to interact with toCKB contracts.

```bash
$ tockb-cli tockb -h
tockb-cli-tockb
tockb cli tools

USAGE:
    tockb-cli tockb [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
        --no-color         Do not highlight(color) output json
        --debug            Display request parameters
        --wait-for-sync    Ensure the index-store synchronizes completely before command being executed
        --no-sync          Don't wait index database sync to tip
    -h, --help             Prints help information
    -V, --version          Prints version information

OPTIONS:
    -c, --config-path <config-path>        tockb config path [default: .tockb-cli]
        --output-format <output-format>    Select output format [default: yaml]  [possible values: yaml, json]

SUBCOMMANDS:
    deploy             deploy toCKB scripts
    deposit_request    create a cell to request deposit
    help               Prints this message or the help of the given subcommand(s)

$ tockb-cli tockb deploy -h
tockb-cli-tockb-deploy
deploy toCKB scripts

USAGE:
    tockb-cli tockb deploy [FLAGS] [OPTIONS] --privkey-path <privkey-path> --tx-fee <tx-fee>

FLAGS:
        --no-color         Do not highlight(color) output json
        --debug            Display request parameters
        --wait-for-sync    Ensure the index-store synchronizes completely before command being executed
        --no-sync          Don't wait index database sync to tip
    -h, --help             Prints help information
    -V, --version          Prints version information

OPTIONS:
        --privkey-path <privkey-path>          Private key file path (only read first line)
        --output-format <output-format>        Select output format [default: yaml]  [possible values: yaml, json]
        --tx-fee <tx-fee>                      The transaction fee capacity (unit: CKB, format: 0.0001)
    -t, --typescript-path <typescript-path>    typescript path [default: ../build/release/toCKB-typescript]
    -l, --lockscript-path <lockscript-path>    lockscript path [default: ../build/release/toCKB-lockscript]

$ tockb-cli tockb deploy --tx-fee 0.1 --privkey-path privkeys/0
scripts config:

[lockscript]
code_hash = "0d665001e9c412712ceb28ea809639400af13fa53df65e36cb46a9cf3c4d4023"

[lockscript.outpoint]
tx_hash = "673950ab271b85eb586389a38c33c21e5cefd84470ad2b4ef8c9f906ffbfaf2f"
index = 1

[typescript]
code_hash = "6b643871862b98c6836e3c4f41d5b42c98bd3dc6da17c0947d655ccf66b20e9c"

[typescript.outpoint]
tx_hash = "673950ab271b85eb586389a38c33c21e5cefd84470ad2b4ef8c9f906ffbfaf2f"
index = 0

scripts config written to ".tockb-cli/scripts.toml"
deploy finished!

$ tockb-cli tockb deposit_request -h
tockb-cli-tockb-deposit_request
create a cell to request deposit

USAGE:
    tockb-cli tockb deposit_request [FLAGS] [OPTIONS] --tx-fee <tx-fee>

FLAGS:
        --no-color         Do not highlight(color) output json
        --debug            Display request parameters
        --wait-for-sync    Ensure the index-store synchronizes completely before command being executed
        --no-sync          Don't wait index database sync to tip
    -h, --help             Prints help information
    -V, --version          Prints version information

OPTIONS:
        --privkey-path <privkey-path>
            Private key file path (only read first line)

        --from-account <from-account>
            The account's lock-arg or sighash address (transfer from this account)

        --output-format <output-format>
            Select output format [default: yaml]  [possible values: yaml, json]

        --from-locked-address <from-locked-address>
            The time locked multisig address to search live cells (which S=0,R=0,M=1,N=1 and have since value)

        --tx-fee <tx-fee>
            The transaction fee capacity (unit: CKB, format: 0.0001)

        --derive-receiving-address-length <derive-receiving-address-length>
            Search derived receiving address length [default: 1000]

        --derive-change-address <derive-change-address>
            Manually specify the last change address (search 10000 addresses max, required keystore password, see: BIP-
            44)
    -p, --pledge <pledge>                                                      pledge
    -l, --lot_size <lot_size>                                                  lot_size
    -k, --kind <kind>                                                          kind

$ tockb-cli tockb deposit_request -l 1 -k 1 -p 10000 --tx-fee 0.1 --privkey-path privkeys/0
0xe0e69db2a07a91a6bb1ab057f4ab0fca58cf1313a0d31cd5b83892d48b50a9f1
```