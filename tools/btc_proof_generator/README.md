# BTC Proof Generator

This is a tool to generate the btc proof in toCKB.

## Usage

```bash
$ cd /path/to/toCKB

$ cargo build -p btc_proof_generator --release

$ ./target/release/btc_proof_generator -h
btc-proof-generator 0.1
Wenchao Hu <me@huwenchao.com>
generate btc proof for toCKB

USAGE:
    btc_proof_generator [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    mint_xt    generate proof for mint_xt

$ ./target/release/btc_proof_generator mint_xt -h
btc_proof_generator-mint_xt
generate proof for mint_xt

USAGE:
    btc_proof_generator mint_xt --block_hash <block_hash> --funding_output_index <funding_output_index> --tx_index <tx_index>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --block_hash <block_hash>                        block hash
    -f, --funding_output_index <funding_output_index>    funding output index
    -t, --tx_index <tx_index>                            tx index


# The block: <https://blockchain.info/block/0000000000000000000d97f0ed2e8c1e2c1eeb92a1853c3a9107cfd2f11e1aa4>
# The tx: <https://www.blockchain.com/btc/tx/c33b712400ac3c262272ea6f0ddbfeab56d7786c84b2419ec25cf1e66a84212b>
$ ./target/release/btc_proof_generator mint_xt -b 0000000000000000000d97f0ed2e8c1e2c1eeb92a1853c3a9107cfd2f11e1aa4 -t 3 -f 0
btc mint xt proof:

{
  "version": "02000000",
  "vin": "015227c5fbad9d9202ade7f02452cf880dac1ed270255ebfe6716e8b3e8956571d0100000017160014085fc2ea0c102fc4db8dbbb10dd6f93684c178c9feffffff",
  "vout": "028c79171300000000160014173ec3a12e289b102f8edcc1d4ecd3b5b893e2dc97b2030000000000160014ef9665bcf82fa83e870a350a6551a09ee819e4a3",
  "locktime": "dbd80900",
  "tx_id": "2b21846ae6f15cc29e41b2846c78d756abfedb0d6fea7222263cac0024713bc3",
  "index": 3,
  "headers": "00000020acf05cadf6d066d01f5aca661690f4e1779a8144b90b070000000000000000006bbb5a7851af48d883e8ac5d6f61c6ad9a4132a9a12531c1b6f085760b3b2e427ba0455fea0710177d792e86",
  "intermediate_nodes": "8546dfccb488115f9c3210255523c0e186fb9b64d16ac68b3d8903bf037dc3ab26069e90c930cc55105d5f8b4ddd798bc33f057641e748fd2e70de0b8747cae802af46fb1e1fccf354b4b46d87f5a85c564fd5284cbe2a5711c16c446fbb6e9e0b3c7beec06a156a8005883b8cf224f665d361a2269b6b21491c1ccbb8160c311b609b5ca21b0a9f708e6124b36871b71c5536d8d556054be435cf0444da70d0814e678eb0e081805d777f9cf84911f9e04b6a80b6cf60dec31527ec73aaa8ba77ec6bff2e04fbb80c8c81b1cc38b415bc21dd732f51a4a903ee265b0eef2c589f751e66e46bb02aa36ed8418ae93317316b84d12f1b1702dd9641ead0ad7f8777526ad7a4ff599946d219a7a932ec8cd2e42649b3d5fa123d2e4532de6d46bddb27a8c02de8fb8fe2c4d88a14132de8cdd7d471bc6a8c8c217aeec600fd295e8925b663332f45bdb6877dd6e0ecd28bfae530ba3ed8bd3959644a82bc418f9c887746e15ae55d82369c3761187ea449c7f7bdff1acaa0b467e1335b3919089d",
  "funding_output_index": 0
}
```