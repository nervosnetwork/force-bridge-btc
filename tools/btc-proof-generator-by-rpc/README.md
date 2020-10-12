# BTC Proof Generator

This is a tool to generate the btc proof using btc node RPC.

## Usage

```bash
$ cd /path/to/toCKB

$ cargo build -p btc-proof-generator-by-rpc --release

$ ./target/release/btc-proof-generator-by-rpc -h
btc-proof-generator-by-rpc 0.1
jacobdenver007 <jacobdenver@163.com>
generate btc proof for toCKB

USAGE:
    btc-proof-generator-by-rpc <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    mint-xt

$ ./target/release/btc-proof-generator-by-rpc mint_xt -h
btc-proof-generator-by-rpc-mint-xt

USAGE:
    btc-proof-generator-by-rpc mint-xt --tx-hash <tx-hash> --funding-input-index <funding-input-index> --funding-output-index <funding-output-index>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --funding-input-index <funding-input-index>
    -o, --funding-output-index <funding-output-index>
    -t, --tx-hash <tx-hash>

$ ./target/release/btc-proof-generator-by-rpc mint-xt --tx-hash 41fb1aeacf96735e56c887c7e8d2f0efed43c4bfe2cda46e4edb99a28ac277ab --funding-input-index 0 --funding-output-index 0

{
  "version": 1,
  "vin": "016fca52ac1b8f18dc7c9747687702fe518e307c940ee432653651757149596769000000000000000000",
  "vout": "01406f7e4a00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced",
  "locktime": 0,
  "tx_id": "ab77c28aa299db4e6ea4cde2bfc443edeff0d2e8c787c8565e7396cfea1afb41",
  "index": 1,
  "headers": "000000203fdffd0b7a43f03f911c51dbaa81a7e01ee2fa2bc9e39eac812d173ca756b73370aefd54923a05abb8bf26c874eb7f8801dae5d34d569d45398376294e1f8457effa725fffff7f2000000000",
  "intermediate_nodes": "3d55cf41d9ee293736caff3210215d3c60ba0b22569609971067248eac82279b",
  "funding_output_index": 0,
  "funding_input_index": 0
}


proof in molecule bytes:

2e0100002c000000300000005e0000008200000086000000a6000000ae00000002010000260100002a010000010000002a000000016fca52ac1b8f18dc7c9747687702fe518e307c940ee4326536517571495967690000000000000000002000000001406f7e4a00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced00000000ab77c28aa299db4e6ea4cde2bfc443edeff0d2e8c787c8565e7396cfea1afb41010000000000000050000000000000203fdffd0b7a43f03f911c51dbaa81a7e01ee2fa2bc9e39eac812d173ca756b73370aefd54923a05abb8bf26c874eb7f8801dae5d34d569d45398376294e1f8457effa725fffff7f2000000000200000003d55cf41d9ee293736caff3210215d3c60ba0b22569609971067248eac82279b0000000000000000
```