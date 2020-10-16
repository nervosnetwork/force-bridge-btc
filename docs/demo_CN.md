# BTC <-> CKB 跨链 Demo

[toCKB](./design.md) 是将其他链（eg. Bitcoin, Ethereum）的资产跨到 CKB 上的去中心协议。

本文向读者描述和演示如何将 BTC 跨到 CKB 上生成 cBTC，以及将 cBTC 跨回 Bitcoin 得到 BTC。

## 准备

### 环境搭建

跨链流程包含的组件有：

##### 1. CKB 开发链: 开发友好的 CKB 私链
  > 搭建流程参考[官方文档](https://docs.nervos.org/docs/basics/guides/devchain)
##### 2. CKB Indexer：提供 CKB 链上数据的索引服务，tockb-cli 依赖该服务
  > 搭建流程参考[官方库](https://github.com/nervosnetwork/ckb-indexer)
##### 3. Bitcoin Regtest: 比特币私链
  > 搭建流程参考[文档](https://gist.github.com/System-Glitch/cb4e87bf1ae3fec9925725bb3ebe223a)
我们使用的配置如下：
```shell
// bitcoin.conf
daemon=1
server=1
rpcuser=test
rpcpassword=test
regtest=1
txindex=1
rpcallowip=0.0.0.0/0
discover=0
listen=0
fallbackfee=0.02
```

##### 4. toCKB 仓库：提供 CKB 跨链合约，合约部署及交互工具(tockb-cli)，跨链消息生成工具(proof-generator-by-rpc)

```shell
$ git clone https://github.com/nervosnetwork/toCKB.git
$ cd toCKB
$ git checkout demo
// 编译合约
$ capsule build --release
// 编译 tockb-cli， proof-generator-by-rpc
$ cargo build
```

### 使用 Docker 搭建比特币私链、CKB 私链、CKB Indexer (推荐使用，此 demo 后续步骤均基于 docker 环境)
使用 docker-compose 一键搭建：

```shell
$ git clone https://github.com/nervosnetwork/toCKB.git
$ cd toCKB/docker
$ docker-compose up
```

### 部署 toCKB 合约

部署 toCKB 的 lockscript，typescript，sudt 合约，构建 price_oracle 和 btc_difficulty 的块，执行完成后会将上述几个 cell 的 outpoint 和 code_hash 打印到默认保存路径 ```/tmp/.tockb-cli/config.toml```  中

```shell
$ cd toCKB/cli
$ mkdir -p /tmp/.tockb-cli
$ mkdir -p privkeys
$ echo "0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc" > privkeys/admin
$ ../target/debug/tockb-cli dev-init --force --price 10000 --btc-difficulty 0 --private-key-path privkeys/admin
```

命令行选项解析：
```
--force                     强制覆盖 /tmp/.tockb-cli/config.toml
--price                     CKB/SAT 价格
--btc-difficulty            BTC 区块难度（由于使用私链，此处为0）
--private-key-path          私钥路径，即这笔交易的发起方
```

## BTC->CKB

### 背景

让我们想象这样一种场景：Alice 手中有 1 个 BTC，她想把这个 BTC 转到 CKB 的网络上进行操作，这时候就需要使用 toCKB 跨链工具实现从 BTC->CKB 的过程。

假设 Alice 的 BTC 地址及 CKB 私钥地址如下：
> 本 demo 中所有的 BTC 地址均为 regtest 测试网下的 P2WPKH 格式，且都已经在 [docker](../docker/bitcoin/entrypoint.sh) 中提前配置好

|              | Alice                                                         |
| ------------ | ------------------------------------------------------------ | 
| btc 地址     | bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf                       | 
| ckb 私钥     | 0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc | 
| ckb 地址     | ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37                     | 

### 查询 Alice 的 BTC 余额为 2：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="alice" getbalance
2.00000000
```
> 本 demo 中所有的 bitcoin-cli 操作都是在 docker 中操作，请先 docker attach 再执行

### 导入 Alice 的 CKB 私钥方便后续操作：

```shell
$ echo '0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc' > privkeys/alice
```

### Alice 发起跨链请求：

Alice 向 CKB 网络发起跨链请求，需要提供自己的 CKB 地址，跨链的 BTC 金额以及支付抵押金（10000 CKB）：

```shell
$ ../target/debug/tockb-cli contract --private-key-path privkeys/alice --wait-for-committed deposit-request --lot-size 3 --kind 1 --pledge 10000 --user-lockscript-addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
```

命令行选项解析：
```
--kind                      CKB 跨链对象，1 表示 BTC
--lot-size                  跨链金额，目前对 BTC 跨 CKB 只支持三种情况：lot-size 为 1 表示 0.25 个 BTC，为 2 表示 0.5 个 BTC，为 3 表示 1 个 BTC。因为 Alice 要跨 1 个 BTC，所以此处写 3。
--pledge                    Alice 执行跨链需要缴纳的押金（规定为 10000 CKB）
--user-lockscript-addr      Alice CKB 地址，用于接收跨链资产
```

执行结果如下：

```shell
{
  "cell_typescript": "5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f0700250000000129b3e39a746ff6b95ce486c9aadc1cde6b064b8e79066548db1a1954286a61fc05000000",
  "tx_hash": "e6e36f0cd3b562f2a37c7fbf1900fe4f62a726aab8b97e22a538dcaaf4193ed4"
}
```

```cell_typescript``` 表示 toCKB cell 的 typescript，相当于是本次跨链的唯一标识，后面的操作也会使用到 ```cell_typescript``` 的值，将 ```cell_typescript``` 保存下作为后面命令的参数：

```shell
$ export CELL=5a000000100000003000000031000000b64479991672218d70cb7a34b31a6d74e464b97b20d965e909bd12db5fd83f0700250000000129b3e39a746ff6b95ce486c9aadc1cde6b064b8e79066548db1a1954286a61fc05000000
```

### 担保人 Bob

Alice 执行完上一步后，只是向 CKB 网络提交了跨链的申请，还并没有执行，此时需要一个担保人来协助 Alice 完成跨链操作。
> 担保人需要抵押 CKB，因此必须要有足量的 CKB 才能成为担保人

我们假设现在正好有个担保人 Bob 愿意为 Alice 的这笔跨链交易做担保，且Bob 的 BTC 地址及 CKB 私钥地址如下：

|              | Bob                                                         |
| ------------ | ------------------------------------------------------------ | 
| btc 地址     | bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2                       | 
| ckb 私钥     | 0x63d86723e08f0f813a36ce6aa123bb2289d90680ae1e99d4de8cdb334553f24d | 
| ckb 地址     | ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4                     | 

那么此时的情况是 Alice 请求 1 个 BTC 的跨链，且此时 CKB/SAT 的价格是 1000，那么 Bob 需要抵押的 CKB 数量是：

```
[(1 * 100000000) / 1000] * 1.5 + 2 * 200 
```
> 即 Bob 需要抵押 Alice 跨链的 BTC 金额的 1.5 倍等价的 CKB 金额，外加固定收费的 400 个 CKB。

### 查询 Bob 的 BTC 余额为 0.2：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="bob" getbalance
0.20000000
```

### 导入 Bob 的 CKB 私钥方便后续操作：

```shell
$ echo '0x63d86723e08f0f813a36ce6aa123bb2289d90680ae1e99d4de8cdb334553f24d' > privkeys/bob
```

### Bob 响应 Alice 的跨链请求

Bob 抵押相应数额的 CKB，并提供自己的 BTC 地址用于接收 Alice 的 BTC，提供自己的 CKB 地址用于接收手续费：

```shell
$ ../target/debug/tockb-cli contract --private-key-path privkeys/bob --wait-for-committed bonding --cell $CELL --lock-address bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 --signer-lockscript-addr ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4
```

命令行选项解析：
```
--cell                      toCKB cell 的 typescript，使用 deposit-request 这步所生产的 cell_typescript
--lock-address              Bob 提供的 BTC 锁定地址
--signer-lockscript-addr    Bob 的 CKB 地址
```

### Alice 向 Bob 的 BTC 地址转账 1 BTC

在 CKB 网络上执行完上述两步后，Alice 此时要在 BTC 网络上向 Bob 提供的地址转账，并且生成 spv proof。流程如下：

首先 Alice 向 Bob 提供的 lock-address 转账 1 个 BTC：

```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="alice" sendtoaddress bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 1
```

此时会得到交易 hash ：

```shell
4156e38ea4f47c2c0319332f8ea4396c3b0ca7873e6b8a43c9002b3fb601d817
```

先将交易 hash 保存：

```shell
$ export BTC_TX_HASH=4156e38ea4f47c2c0319332f8ea4396c3b0ca7873e6b8a43c9002b3fb601d817
```

等待片刻待交易打包后，查询交易详情：
```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf getrawtransaction 4156e38ea4f47c2c0319332f8ea4396c3b0ca7873e6b8a43c9002b3fb601d817 true
{
  "txid": "4156e38ea4f47c2c0319332f8ea4396c3b0ca7873e6b8a43c9002b3fb601d817",
  "hash": "39e62d50be282a1fc6a0d3fb473510248bb2db01594e1c3c42e1ec5eeb77996c",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 706,
  "vin": [
    {
      "txid": "4fe8b578d9c7262971de00c394a848873270d5d64b15efcd73711ae080f2c2b1",
      "vout": 1,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "304402200f05ec44326e12d2a92eba2fce902b809593a9fe86c753fb4973d2da6ed159d802205275d9d93ee3aad1bcbd8371b605d3dfa7fb2e781577491d2ad385ddfc7b83f201",
        "0227de674775b35b06fca8ed06a492c817d542cc08b8d4f64d3717d4af70134d80"
      ],
      "sequence": 4294967294
    }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 489b80d23c5148b2f29c8b5f03478a4d0dd67a00",
        "hex": "0014489b80d23c5148b2f29c8b5f03478a4d0dd67a00",
        "reqSigs": 1,
        "type": "witness_v0_keyhash",
        "addresses": [
          "bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2"
        ]
      }
    },
    {
      "value": 0.99718000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 3f9de41a1bb25201e1ca7b713aa2ad5bd79004d5",
        "hex": "00143f9de41a1bb25201e1ca7b713aa2ad5bd79004d5",
        "reqSigs": 1,
        "type": "witness_v0_keyhash",
        "addresses": [
          "bcrt1q87w7gxsmkffqrcw20dcn4g4dt0teqpx4w9fmpy"
        ]
      }
    }
  ],
  "hex": "02000000000101b1c2f280e01a7173cdef154bd6d570328748a894c300de712926c7d978b5e84f0100000000feffffff0200e1f50500000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a007093f105000000001600143f9de41a1bb25201e1ca7b713aa2ad5bd79004d50247304402200f05ec44326e12d2a92eba2fce902b809593a9fe86c753fb4973d2da6ed159d802205275d9d93ee3aad1bcbd8371b605d3dfa7fb2e781577491d2ad385ddfc7b83f201210227de674775b35b06fca8ed06a492c817d542cc08b8d4f64d3717d4af70134d80c2020000",
  "blockhash": "67b9e3b2f53c76f9441b1f29f22b9ff729656f5833f2f6c7b6a7f608c0b84fc8",
  "confirmations": 17,
  "time": 1602748628,
  "blocktime": 1602748628
}
```

我们需要关注下这笔交易的详细信息，这笔交易有 1 个 vin，2 个 vout，而第一个 vout 才是我们真正想要的那个转账给 Bob 1 个 BTC 的 vout。
因此，我们需要的 vin index = 0，vout index = 0。

使用 btc-proof-generator-by-rpc，结合刚刚得到的交易 Hash，vin，vout 信息生成 spv proof：

```shell
$ cd toCKB
$ ./target/debug/btc-proof-generator-by-rpc mint-xt --tx-hash $BTC_TX_HASH --funding-input-index 0 --funding-output-index 0
```

btc-proof-generator-by-rpc 命令行参数解析：

```
--tx-hash                   交易 hash
--funding-input-index       交易的 input utxo 中使用到的地址的 index（本示例中，交易的 input 的第 0 位的 utxo 是 Alice 地址的，则该值为 0）
--funding-output-index      交易的 output utxo 中使用到的地址的 index（本示例中，交易的 output 的第 1 位的 utxo 是给 Bob 地址转账 1 BTC 的，则该值为 1）
```

结果如下：

```shell
btc mint xt proof:

{
  "version": 2,
  "vin": "01b1c2f280e01a7173cdef154bd6d570328748a894c300de712926c7d978b5e84f0100000000feffffff",
  "vout": "0200e1f50500000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a007093f105000000001600143f9de41a1bb25201e1ca7b713aa2ad5bd79004d5",
  "locktime": 706,
  "tx_id": "17d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee35641",
  "index": 1,
  "headers": "00000020bb00dd6e288990fab13e3faa34ac76df62b7284d471938cb8978178f1c1edf72af350b4a7e8d2722a5dd8993ae4284f49114fe4b4845c966a1a54212d86457ced400885fffff7f2000000000",
  "intermediate_nodes": "5146421e3811b8a29e95373c75f5b4c8ddba832cd1fa2e87255f37b8dbc13c05",
  "funding_output_index": 0,
  "funding_input_index": 0
}


proof in molecule bytes:

4d0100002c000000300000005e000000a1000000a5000000c5000000cd000000210100004501000049010000020000002a00000001b1c2f280e01a7173cdef154bd6d570328748a894c300de712926c7d978b5e84f0100000000feffffff3f0000000200e1f50500000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a007093f105000000001600143f9de41a1bb25201e1ca7b713aa2ad5bd79004d5c202000017d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee3564101000000000000005000000000000020bb00dd6e288990fab13e3faa34ac76df62b7284d471938cb8978178f1c1edf72af350b4a7e8d2722a5dd8993ae4284f49114fe4b4845c966a1a54212d86457ced400885fffff7f2000000000200000005146421e3811b8a29e95373c75f5b4c8ddba832cd1fa2e87255f37b8dbc13c050000000000000000
```

将得到的 spv proof 保存：

```
$ export SPV_PROOF=4d0100002c000000300000005e000000a1000000a5000000c5000000cd000000210100004501000049010000020000002a00000001b1c2f280e01a7173cdef154bd6d570328748a894c300de712926c7d978b5e84f0100000000feffffff3f0000000200e1f50500000000160014489b80d23c5148b2f29c8b5f03478a4d0dd67a007093f105000000001600143f9de41a1bb25201e1ca7b713aa2ad5bd79004d5c202000017d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee3564101000000000000005000000000000020bb00dd6e288990fab13e3faa34ac76df62b7284d471938cb8978178f1c1edf72af350b4a7e8d2722a5dd8993ae4284f49114fe4b4845c966a1a54212d86457ced400885fffff7f2000000000200000005146421e3811b8a29e95373c75f5b4c8ddba832cd1fa2e87255f37b8dbc13c050000000000000000
```

### Alice 获取 cBTC

完成上一步后，Alice 相当于锁仓了 1 BTC，此时，Alice 提供上一笔 BTC 交易的 SPV PROOF 就可以获取 CKB 上的代币 cBTC：
> cBTC 的数量和跨链锁仓的 BTC 对应的 SAT 的数量是一致的，以 Alice 本次跨链为例，1 个 BTC 是 100000000 个 SAT，那么就会生成出 100000000 个 cBTC
> 其中 Bob 作为担保人要抽取 2/1000 的手续费，所以最终 Alice 拿到 99800000 个 cBTC，Bob 拿到 200000 个 cBTC，

```shell
$ cd toCKB/cli
$ ../target/debug/tockb-cli contract --private-key-path privkeys/alice --wait-for-committed mint-xt -c $CELL --spv-proof $SPV_PROOF
```

经过上述几个步骤，Alice 就完成了 BTC->CKB 的跨链过程，现在可以查看下 Alice 和 Bob 在跨链后的资产情况。

Alice 查询 BTC 余额：

```
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="alice" getbalance
0.99718000
```

Alice 查询 cBTC 余额：

```
$ ../target/debug/tockb-cli sudt --kind 1 get-balance --addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
99800000
```

Bob 查询 BTC 余额：

```
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="bob" getbalance
1.20000000
```

Bob 查询 cBTC 余额：
```
$ ../target/debug/tockb-cli sudt --kind 1 get-balance --addr ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4
200000
```

金额变动总结：

|              | BTC->CKB 跨链前                                                         | 跨链后 |
| ------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | 
| Alice BTC    | 2                     | 0.99718000 |
| Bob BTC      | 0.2                   | 1.2 |
| Alice cBTC   | 0                     | 99800000 |
| Bob cBTC     | 0                     | 200000 |


## CKB->BTC 跨链

### Alice 归集 cBTC

Alice 现在想赎回自己锁仓的 BTC，于是她先在市场上（如交易所）中购买 200000 个 cBTC：
> 实际场景是 Alice 在市场上购买 cBTC，此处为了简化，直接让 Bob 给 Alice 转账 cBTC

```
$ ../target/debug/tockb-cli sudt --kind 1 transfer --wait-for-committed --private-key-path privkeys/bob --to-addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37 --sudt-amount 200000
```

### Alice 提交赎回 BTC 的请求

Alice 销毁自己的 100000000 个 cBTC，同时发起赎回 BTC 请求，提供自己的 BTC 地址及 CKB 地址： 

```shell
../target/debug/tockb-cli contract --private-key-path privkeys/alice --wait-for-committed pre-term-redeem -c $CELL --unlock-address bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf --redeemer-lockscript-addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
```

命令行参数解析：

```
--unlock-address                 user 提供的 BTC 解锁地址
--redeemer-lockscript-addr       redeemer 的 CKB 地址 （此处即为 Alice 的地址）
```

### Bob 向 Alice 转账

Bob 接收到 Alice 赎回的请求后，要在 BTC 网络上向 Alice 转账，并且生成 SPV PROOF。流程如下：

首先 Bob 向 Alice 提供的 unlock-address 转账：
```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="bob" sendtoaddress bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf 1
```

此时会得到交易 hash ：

```shell
62f0a0a94025b5c3a0f59d2b191dbc99d1d69be2dee3169f580421a9b1197a36
```

先将交易 hash 保存：

```shell
$ export BTC_TX_HASH=62f0a0a94025b5c3a0f59d2b191dbc99d1d69be2dee3169f580421a9b1197a36
```

等待片刻待交易打包后，查询交易详情：
```shell
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf getrawtransaction 62f0a0a94025b5c3a0f59d2b191dbc99d1d69be2dee3169f580421a9b1197a36 true
{
  "txid": "62f0a0a94025b5c3a0f59d2b191dbc99d1d69be2dee3169f580421a9b1197a36",
  "hash": "cc69fe59f727040efac9db7e84599d0b5bc27b2498262859d25ca85e560ecfa6",
  "version": 2,
  "size": 370,
  "vsize": 208,
  "weight": 832,
  "locktime": 1646,
  "vin": [
    {
      "txid": "b5c5b1f76d3b68b7bc1df439d168b1563963223efb1f384e5ed8d2b8070c99fa",
      "vout": 1,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "3044022017e87dee8d8fbe6a09a0a0052a30e47cc08396478b05bd208c8cccae94dedee0022044f1c1eb6ab74d28c45a4d65da7543eff5fc3885896c452c8613d249af98ece301",
        "03c6d39badda2718c1a89ebde024774910499ac67278b6d6b7ed18be9f17a566d9"
      ],
      "sequence": 4294967294
    },
    {
      "txid": "4156e38ea4f47c2c0319332f8ea4396c3b0ca7873e6b8a43c9002b3fb601d817",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "30440220617ede3c3bcdc04b18acdeeca2732c5966670985b8e0aa23e21c268ee9bc2b9a022055ddf12c4b4701991eefda72b98c84953e21b554aae6fc9d52be0251c2c218cc01",
        "03c6d39badda2718c1a89ebde024774910499ac67278b6d6b7ed18be9f17a566d9"
      ],
      "sequence": 4294967294
    }
  ],
  "vout": [
    {
      "value": 0.19584000,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 b22b359c7e106b4673068417752373305fe2f52e",
        "hex": "0014b22b359c7e106b4673068417752373305fe2f52e",
        "reqSigs": 1,
        "type": "witness_v0_keyhash",
        "addresses": [
          "bcrt1qkg4nt8r7zp45vucxssth2gmnxp079afw80ur44"
        ]
      }
    },
    {
      "value": 1.00000000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 a8cb707e4d0a5c6e690189bc0065a8f787aabced",
        "hex": "0014a8cb707e4d0a5c6e690189bc0065a8f787aabced",
        "reqSigs": 1,
        "type": "witness_v0_keyhash",
        "addresses": [
          "bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf"
        ]
      }
    }
  ],
  "hex": "02000000000102fa990c07b8d2d85e4e381ffb3e22633956b168d139f41dbcb7683b6df7b1c5b50100000000feffffff17d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee356410000000000feffffff0200d42a0100000000160014b22b359c7e106b4673068417752373305fe2f52e00e1f50500000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced02473044022017e87dee8d8fbe6a09a0a0052a30e47cc08396478b05bd208c8cccae94dedee0022044f1c1eb6ab74d28c45a4d65da7543eff5fc3885896c452c8613d249af98ece3012103c6d39badda2718c1a89ebde024774910499ac67278b6d6b7ed18be9f17a566d9024730440220617ede3c3bcdc04b18acdeeca2732c5966670985b8e0aa23e21c268ee9bc2b9a022055ddf12c4b4701991eefda72b98c84953e21b554aae6fc9d52be0251c2c218cc012103c6d39badda2718c1a89ebde024774910499ac67278b6d6b7ed18be9f17a566d96e060000",
  "blockhash": "4edea8165c35358b9064ce78e34e268bedd78fca37f5ad63764a428985500107",
  "confirmations": 46,
  "time": 1602750515,
  "blocktime": 1602750515
}
```

我们需要关注下这笔交易的详细信息，这笔交易有 2 个 vin，2 个 vout，系统要求的 vin 必须是 Alice 给 Bob 转账的那笔交易，且 vout 是 Bob 给 Alice 转账 1 BTC 的 vout。
结合之前 Alice 给 Bob转账的 txid 可知，我们需要的 vin index = 1，vout index = 1。

使用 btc-proof-generator-by-rpc，结合刚刚得到的交易 Hash，vin，vout 信息生成 spv proof：

```shell
$ cd toCKB
$ ./target/debug/btc-proof-generator-by-rpc mint-xt --tx-hash $BTC_TX_HASH --funding-input-index 1 --funding-output-index 1
```

结果如下：

```shell
btc mint xt proof:

{
  "version": 2,
  "vin": "02fa990c07b8d2d85e4e381ffb3e22633956b168d139f41dbcb7683b6df7b1c5b50100000000feffffff17d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee356410000000000feffffff",
  "vout": "0200d42a0100000000160014b22b359c7e106b4673068417752373305fe2f52e00e1f50500000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced",
  "locktime": 1646,
  "tx_id": "367a19b1a92104589f16e3dee29bd6d199bc1d192b9df5a0c3b52540a9a0f062",
  "index": 1,
  "headers": "00000020277e9d8cf1e66423269442a2be95bdb197b50baa4d5ab0933baf8aefa0f4af0f09cd28a2a017d0986e877178dc34807c30ce40a0fc079b2cd881f505ba8cea243308885fffff7f2000000000",
  "intermediate_nodes": "b3883cd4b5d3dd99ba18a472f7cc0951fb6c738464ea2c04472d7ffde93eea8f",
  "funding_output_index": 1,
  "funding_input_index": 1
}


proof in molecule bytes:

760100002c0000003000000087000000ca000000ce000000ee000000f60000004a0100006e01000072010000020000005300000002fa990c07b8d2d85e4e381ffb3e22633956b168d139f41dbcb7683b6df7b1c5b50100000000feffffff17d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee356410000000000feffffff3f0000000200d42a0100000000160014b22b359c7e106b4673068417752373305fe2f52e00e1f50500000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced6e060000367a19b1a92104589f16e3dee29bd6d199bc1d192b9df5a0c3b52540a9a0f06201000000000000005000000000000020277e9d8cf1e66423269442a2be95bdb197b50baa4d5ab0933baf8aefa0f4af0f09cd28a2a017d0986e877178dc34807c30ce40a0fc079b2cd881f505ba8cea243308885fffff7f200000000020000000b3883cd4b5d3dd99ba18a472f7cc0951fb6c738464ea2c04472d7ffde93eea8f0100000001000000
```

将得到的 spv proof 保存：

```
$ export SPV_PROOF=760100002c0000003000000087000000ca000000ce000000ee000000f60000004a0100006e01000072010000020000005300000002fa990c07b8d2d85e4e381ffb3e22633956b168d139f41dbcb7683b6df7b1c5b50100000000feffffff17d801b63f2b00c9438a6b3e87a70c3b6c39a48e2f3319032c7cf4a48ee356410000000000feffffff3f0000000200d42a0100000000160014b22b359c7e106b4673068417752373305fe2f52e00e1f50500000000160014a8cb707e4d0a5c6e690189bc0065a8f787aabced6e060000367a19b1a92104589f16e3dee29bd6d199bc1d192b9df5a0c3b52540a9a0f06201000000000000005000000000000020277e9d8cf1e66423269442a2be95bdb197b50baa4d5ab0933baf8aefa0f4af0f09cd28a2a017d0986e877178dc34807c30ce40a0fc079b2cd881f505ba8cea243308885fffff7f200000000020000000b3883cd4b5d3dd99ba18a472f7cc0951fb6c738464ea2c04472d7ffde93eea8f0100000001000000
```

### Bob 取回自己的押金

Bob 把 Alice 锁仓的 BTC 返还之后，即可通过上笔交易的 SPV PROOF 取回自己抵押的 CKB：

```shell
$ cd toCKB/cli
$ ../target/debug/tockb-cli contract --private-key-path privkeys/bob --wait-for-committed withdraw-collateral -c $CELL --spv-proof $SPV_PROOF
```

结果上述两步，就完成了 CKB->BTC 的跨链过程，此时看下 Alice 和 Bob 的资产情况。

Alice 查询 BTC 余额：

```
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="alice" getbalance
1.99718000
```

Alice 查询 cBTC 余额：

```
$ ../target/debug/tockb-cli sudt --kind 1 get-balance --addr ckt1qyqvsv5240xeh85wvnau2eky8pwrhh4jr8ts8vyj37
0
```

Bob 查询 BTC 余额：

```
$ bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="bob" getbalance
0.19584000
```

Bob 查询 cBTC 余额：
```
$ ../target/debug/tockb-cli sudt --kind 1 get-balance --addr ckt1qyqywrwdchjyqeysjegpzw38fvandtktdhrs0zaxl4
0
```

金额变动总结：

|              | CKB->BTC 跨链前                                                         | 跨链后 |
| ------------ | ------------------------------------------------------------ | ------------------------------------------------------------ | 
| Alice BTC    | 0.99718000                     | 1.99718000 |
| Bob BTC      | 1.2                            | 0.19584000 |
| Alice cBTC   | 100000000                      | 0 |
| Bob cBTC     | 0                              | 0 |



