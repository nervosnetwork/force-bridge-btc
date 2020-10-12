### 准备

#### 准备user和signer的私钥及地址

整个跨链过程设计到两个用户：user和signer。user指跨链请求发起者，user会将自己的BTC抵押给signer，换取等额的CKB代币BTC_SUDT，signer则是跨链的服务方。

user和signer都需要准备一个P2WPKH 格式的BTC地址用于接收BTC，以及一个CKB地址用于在CKB网络上进行操作。

本个例子中，user和signer的地址情况如下：

|              | user                                                         | signer                                                       |
| ------------ | ------------------------------------------------------------ | ------------------------------------------------------------ |
| btc 私钥     | cUDfdzioB3SqjbN9vutRTUrpw5EH9srrg6RPibacPo1fGHpfPKqL               | cU9PYTnSkcWoAE15U26JJCwtKiYvTCKYdbWt8e7ovidEGDBwJQ5x         |
| btc 地址     | bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf                       | bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2                 |
| ckb 私钥     | 0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc | 0x63d86723e08f0f813a36ce6aa123bb2289d90680ae1e99d4de8cdb334553f24d |
| ckb 地址     | ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37                     | ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4               |

#### 搭建CKB私链

https://github.com/nervosnetwork/ckb/releases 下载最新版 ckb 执行文件 

```shell
$ cd /path/where/you/want/to/put/ckb-data
$ ckb init -c dev -C . --ba-arg 0x5a7487f529b8b8fd4d4a57c12dc0c70f7958a196
$ ckb run -C .
$ ckb miner -C .
```

#### 启动CKB indexer

```shell
$ git clone https://github.com/nervosnetwork/ckb-indexer.git
$ cd ckb-indexer
$ cargo build --release
$ RUST_LOG=info ./target/release/ckb-indexer -s /tmp/ckb-indexer-test
```

#### 搭建BTC私链

[下载bitcoind/bitcoin-cli](https://bitcoin.org/en/download): BTC节点及客户端

编辑/etc/bitcoin/bitcoin.conf文件，内容如下：

```shell
daemon=1
server=1
rpcuser=test
rpcpassword=test
regtest=1
txindex=1
rpcallowip=0.0.0.0/0
discover=0
listen=0
fallbackfee=0.1
```

执行bitcoind启动私链：

```shell
$ bitcoind -conf=/etc/bitcoin/bitcoin.conf
```

创建user和signer的钱包：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet user
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="user" importprivkey "cUDfdzioB3SqjbN9vutRTUrpw5EH9srrg6RPibacPo1fGHpfPKqL"
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet signer
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="signer" importprivkey "cU9PYTnSkcWoAE15U26JJCwtKiYvTCKYdbWt8e7ovidEGDBwJQ5x"
```

使用user的地址打包201个块，这样user就会有BTC余额

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 201 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
```

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="user" getwalletinfo
{
  "walletname": "user",
  "walletversion": 169900,
  "balance": 5050.00000000,
  "unconfirmed_balance": 0.00000000,
  "immature_balance": 3700.00000000,
  "txcount": 201,
  "keypoololdest": 1602489173,
  "keypoolsize": 1000,
  "hdseedid": "80fbe840f7c2f918abae4dd95e2c36671be7f2ba",
  "keypoolsize_hd_internal": 1000,
  "paytxfee": 0.00000000,
  "private_keys_enabled": true,
  "avoid_reuse": false,
  "scanning": false
}
```



### 使用tockb-cli 执行 toCKB 合约

#### 编译toCKB合约及tockb-cli

```shell
$ git clone https://github.com/nervosnetwork/toCKB.git
$ cd toCKB
$ git checkout demo
$ capsule build --release
$ cd cli
$ cargo build
```

#### 导入user和signer的私钥

```shell
$ mkdir privkeys
$ echo '0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc' > privkeys/user
$ echo '0x63d86723e08f0f813a36ce6aa123bb2289d90680ae1e99d4de8cdb334553f24d' > privkeys/signer
```

#### 部署toCKB合约

部署toCKB的lockscript，typescript，sudt合约，构建price_oracle和btc_difficulty的块，执行完成后会将上述几个cell的outpoint和code_hash打印到默认保存路径 ```/tmp/.tockb-cli/config.toml```  中

```shell
$ ../target/debug/tockb-cli dev-init -f -p 10000 -d 0 -k privkeys/user
```

#### user 执行 deposit_request

```shell
$ ../target/debug/tockb-cli contract -k privkeys/user --wait-for-committed deposit-request -l 1 -k 1 -p 10000 --user-lockscript-addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
```

执行结果如下：

```shell
{
  "cell_typescript": "5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f07002500000001cd72937d649c7f4ce53d4a77e43073b9eb5690fe6b80ac9e8861c0de3b17ecd501000000",
  "tx_hash": "3cc5b71d5e3f5bafe1ac37f609ab3895dcb33e4ff7d413d6d31ef918e34e5660"
}
```

执行完此步骤后保存回显中的 cell_typescript ，后面的操作都会用到cell_typescript的值

#### signer 执行 bonding

```shell
$ ../target/debug/tockb-cli contract -k privkeys/signer --wait-for-committed bonding -c 5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f07002500000001cd72937d649c7f4ce53d4a77e43073b9eb5690fe6b80ac9e8861c0de3b17ecd501000000 -l bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 -s ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4
```

#### user 在 BTC 链上向signer提供的BTC地址转账

在CKB网络上执行完上述两步后，user此时要在BTC网络上向signer提供的地址转账，并且生成spv proof。具体执行过程请参考生成spv proof。

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="user" sendtoaddress bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 10
```

此时会得到交易hash：

```shell
7259f8a389b8847860b9394ab98ad22340056204bd1c4dddfc636c17e60159b0
```

将该交易打包：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 100 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
```

使用btc-proof-generator-by-rpc生成spv proof：

```shell
$ cd /toCKB/path
$ ./target/release/btc-proof-generator-by-rpc mint-xt --tx-hash 7259f8a389b8847860b9394ab98ad22340056204bd1c4dddfc636c17e60159b0 -i 0 -o 1
```

结果如下：

```shell
btc mint xt proof:

{
  "version": 2,
  "vin": "014a0b532962e1e2006a4f1d1aa8ebcc53323a561611d96fcf9eb31d20f1eb9a140100000000feffffff",
  "vout": "02ec2dcd1d00000000160014b01eba5de48f8c6f637c1882e19d7e22fdc0eb8a0065cd1d00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced",
  "locktime": 402,
  "tx_id": "7bd96e4f1927c1d3123998ef2498a73a4aee6bb2c37ec38f79ff5d28382dfe9e",
  "index": 1,
  "headers": "000000300767b75998bc0ff87045a59c834deeb4ac606270fb28f16ebbb33e74c115ac75b4f9e27ccd56246fc44b2e47ddc73b5fdc4dd5d6d94b011bda628ae9d6bef5419f5a845fffff7f2000000000",
  "intermediate_nodes": "1b05aca434b82d3e3c3a39070eae3e96b3db1ebd788e2fc21c7e964b0702b77a",
  "funding_output_index": 1,
  "funding_input_index": 0
}


proof in molecule bytes:

4d0100002c000000300000005e000000a1000000a5000000c5000000cd000000210100004501000049010000020000002a000000014a0b532962e1e2006a4f1d1aa8ebcc53323a561611d96fcf9eb31d20f1eb9a140100000000feffffff3f00000002ec2dcd1d00000000160014b01eba5de48f8c6f637c1882e19d7e22fdc0eb8a0065cd1d00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced920100007bd96e4f1927c1d3123998ef2498a73a4aee6bb2c37ec38f79ff5d28382dfe9e010000000000000050000000000000300767b75998bc0ff87045a59c834deeb4ac606270fb28f16ebbb33e74c115ac75b4f9e27ccd56246fc44b2e47ddc73b5fdc4dd5d6d94b011bda628ae9d6bef5419f5a845fffff7f2000000000200000001b05aca434b82d3e3c3a39070eae3e96b3db1ebd788e2fc21c7e964b0702b77a0100000000000000
```

#### user 执行 mint_xt

user 使用上一步交易的spv proof 构造mint-xt交易。

```shell
$ ../target/debug/tockb-cli contract -k privkeys/user --wait-for-committed mint-xt -c 5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f07002500000001cd72937d649c7f4ce53d4a77e43073b9eb5690fe6b80ac9e8861c0de3b17ecd501000000 --spv-proof 2e0100002c000000300000005e0000008200000086000000a6000000ae00000002010000260100002a010000010000002a00000001ce89cb795d1b1c9c942e6d0192c73793d5332a284e5135c5bf99b0f303303caa0000000000000000002000000001e0f57f4a00000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a00000000006fca52ac1b8f18dc7c9747687702fe518e307c940ee432653651757149596769010000000000000050000000000000204bbb22fd5881cecbc6a62463a782261cbfd897974a0b8d5ac57090c6965609032c93a31ba3c9669ef97b19e0a7161c7a47c30505a661894648f0534644f0b016b9e3725fffff7f200000000020000000df91a540455e5b8c6586793e1d52928047f2b07bc19af072e4cde94ec82010020000000000000000
```

至此，user就将自己的BTC跨链到了CKB上的SUDT代币。



#### user 执行 pre-term redeem

如果user想要赎回BTC，可以执行 pre-term-redeem 从而发出赎回btc的请求，并销毁相应的SUDT。

```shell
$	../target/debug/tockb-cli contract -k privkeys/user --wait-for-committed pre-term-redeem -c 5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f07002500000001cd72937d649c7f4ce53d4a77e43073b9eb5690fe6b80ac9e8861c0de3b17ecd501000000 --unlock-address bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf --redeemer-lockscript-addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37

```

#### signer 在 BTC 链上向user提供的BTC地址转账

signer收到user的redeem请求后，在BTC网络上将user抵押的BTC打给user提供的BTC地址，并且生成spv proof。具体执行过程请参考生成spv proof。

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="signer" sendtoaddress bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf 5
```

此时会得到交易hash：

```shell
6e557a9f861200ebbefbe2f4e8ba791100a5606b8660536c79813db0539103e0
```

将该交易打包：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 100 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
```

使用btc-proof-generator-by-rpc生成spv proof：

```shell
$ cd /toCKB/path
$ ./target/release/btc-proof-generator-by-rpc mint-xt --tx-hash 7259f8a389b8847860b9394ab98ad22340056204bd1c4dddfc636c17e60159b0 -i 0 -o 1
```

结果如下：

```shell
btc mint xt proof:

{
  "version": 2,
  "vin": "01b05901e6176c63fcdd4d1cbd0462054023d28ab94a39b9607884b889a3f859720000000000feffffff",
  "vout": "02ec2dcd1d000000001600144aad12ca19ebf7ea9570d2185f7754cbafcb161c0065cd1d00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced",
  "locktime": 804,
  "tx_id": "e0039153b03d81796c5360866b60a5001179bae8f4e2fbbeeb0012869f7a556e",
  "index": 1,
  "headers": "00000020638bf16c82a148220b14b5fbea6a5af1b5ca1f387a10c25255ff930c1c63423eb345045cdefdab06f8cc4eb6914771a3e6313502488a9c139df8cbc31427f9e3635f845fffff7f2000000000",
  "intermediate_nodes": "59e95e473dd84923e3bbe58a641a55cfed456fac44654c07cdf50aff3d493725",
  "funding_output_index": 1,
  "funding_input_index": 0
}


proof in molecule bytes:

4d0100002c000000300000005e000000a1000000a5000000c5000000cd000000210100004501000049010000020000002a00000001b05901e6176c63fcdd4d1cbd0462054023d28ab94a39b9607884b889a3f859720000000000feffffff3f00000002ec2dcd1d000000001600144aad12ca19ebf7ea9570d2185f7754cbafcb161c0065cd1d00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced24030000e0039153b03d81796c5360866b60a5001179bae8f4e2fbbeeb0012869f7a556e01000000000000005000000000000020638bf16c82a148220b14b5fbea6a5af1b5ca1f387a10c25255ff930c1c63423eb345045cdefdab06f8cc4eb6914771a3e6313502488a9c139df8cbc31427f9e3635f845fffff7f20000000002000000059e95e473dd84923e3bbe58a641a55cfed456fac44654c07cdf50aff3d4937250100000000000000
```

#### signer 执行 withdraw_collateral

signer 使用上一步交易的spv proof 构造withdraw-collateral交易。

```shell
$ ../target/debug/tockb-cli contract -k privkeys/signer --wait-for-committed withdraw-collateral -c 5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f07002500000001cd72937d649c7f4ce53d4a77e43073b9eb5690fe6b80ac9e8861c0de3b17ecd501000000 --spv-proof 2e0100002c000000300000005e0000008200000086000000a6000000ae00000002010000260100002a010000010000002a000000016fca52ac1b8f18dc7c9747687702fe518e307c940ee4326536517571495967690000000000000000002000000001406f7e4a00000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced00000000ab77c28aa299db4e6ea4cde2bfc443edeff0d2e8c787c8565e7396cfea1afb41010000000000000050000000000000203fdffd0b7a43f03f911c51dbaa81a7e01ee2fa2bc9e39eac812d173ca756b73370aefd54923a05abb8bf26c874eb7f8801dae5d34d569d45398376294e1f8457effa725fffff7f2000000000200000003d55cf41d9ee293736caff3210215d3c60ba0b22569609971067248eac82279b0000000000000000

```

### 生成spv proof

btc 发送转账交易：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="user" sendtoaddress bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 10
```

发送完交易后，需要将交易打包：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 100 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
```

在btc网络上发送交易后，使用 btc-proof-generator-by-rpc 工具即可生成 spv proof:

```shell
$ cd /toCKB/path
$ cargo build -p btc-proof-generator-by-rpc --release
$ ./target/release/btc-proof-generator-by-rpc mint-xt --tx-hash 8b83cdd5673ae6a77856232948df8d8cd5027d905aeed7456c07ac87c97e12d1 -i 0 -o 0
```

执行结果如下：

```shell
btc mint xt proof:

{
  "version": 2,
  "vin": "013add6ebdbc5209a8e597603d73c6fbbdec905914c826d1b88fce808de1a2db290000000000feffffff",
  "vout": "02ecf06aee000000001600142eb014a336e210cc8397365f599113e37939fd8600ca9a3b00000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a00",
  "locktime": 201,
  "tx_id": "d1127ec987ac076c45d7ee5a907d02d58c8ddf4829235678a7e63a67d5cd838b",
  "index": 1,
  "headers": "0000003004645f210ec2471e7e6834ab4f91d0e9dda9a2fb4daaab2f5d30737c2e6cd604cf0492b4812e0192e3ef4d56b7a1795e295da2bf3eaf462b73d902d5abc235ccec41845fffff7f2000000000",
  "intermediate_nodes": "b62ac217d5b62f1a4948825b6c8576870b2feed55f9b8a0ad9a2e702bef08e8a",
  "funding_output_index": 0,
  "funding_input_index": 0
}


proof in molecule bytes:

4d0100002c000000300000005e000000a1000000a5000000c5000000cd000000210100004501000049010000020000002a000000013add6ebdbc5209a8e597603d73c6fbbdec905914c826d1b88fce808de1a2db290000000000feffffff3f00000002ecf06aee000000001600142eb014a336e210cc8397365f599113e37939fd8600ca9a3b00000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a00c9000000d1127ec987ac076c45d7ee5a907d02d58c8ddf4829235678a7e63a67d5cd838b0100000000000000500000000000003004645f210ec2471e7e6834ab4f91d0e9dda9a2fb4daaab2f5d30737c2e6cd604cf0492b4812e0192e3ef4d56b7a1795e295da2bf3eaf462b73d902d5abc235ccec41845fffff7f200000000020000000b62ac217d5b62f1a4948825b6c8576870b2feed55f9b8a0ad9a2e702bef08e8a0000000000000000
```


