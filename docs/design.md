# toCKB: An Asset Cross-Chain System

## Abstract

The toCKB is an asset cross-chain system, which consists of a bunch of protocols, CKB contracts and off-chain modules, to support decentralized, redeemable token issued on `CKB CHAIN` supply-pegged to any asset on any blockchain.

As long as you can transfer an asset on a blockchain and construct a spv proof to verify the transaction, you can implement the spv verification logic in the toCKB contract. Then anyone can use the contract to mint a token on CKB pegged to the asset by locking the asset. They can transfer the token and burn it to redeem the asset on original blockchain as well. The peg is implemented using an approch mixed with bonding and spv proof verification, which involves a new role called signer. Signers act as a guard on the original blockchain to ensure the asset is locked when minting token and send the asset back to user when redeeming. Anyone can be signer by bonding an amount of CKB that ensures their behavior in the system remains honest, at risk of losing their bond in case of dishonesty or undercollateralization.

The CKB contracts mediates the cross-chain's lifecycle, including request, bonding,  redemption and failure-handle. The off-chain modules help users use the whole system painlessly, including constructing transactions, monitoring CKB as well as other blockchains to act automatically, generating spv proof and so on.

We will support BTC and ETH in this stage, and provide a guide document to support other assets.

## Overview

### A Note on Naming

The system, in its entirety, is called "toCKB". The original blockchain is refered as "XChain" and the asset as "XAsset". The minted crosschain token on `CKB CHAIN` pegged to XAsset is refered as "XToken", sometimes it can be shorten as "XT".

For BTC and ETH, the names mapping is like below:

| XChain   | XAsset | XToken(XT) |
|----------|--------|------------|
| Bitcoin  | BTC    | cBTC       |
| Ethereum | ETH    | cETH       |

### Prior Work

Prior work towards asset cross-chain includes centralized or multisig notary schemes and spv based bridge.

#### Centralized or Multisig Notary Schemes

A party or a group of parties act as notary. When they receive asset on one chain, they mint token on the counterparty chain and send it to users. When they receive mirror asset on one chain, they burn it and send back associated asset to user on the counterparty chain.

Projects using this schema includes:
- [HBTC](https://huobiglobal.zendesk.com/hc/zh-cn/articles/900000196603-%E5%85%B3%E4%BA%8E%E7%81%AB%E5%B8%81%E5%85%A8%E7%90%83%E7%AB%99%E6%AD%A3%E5%BC%8F%E4%B8%8A%E7%BA%BF%E5%9F%BA%E4%BA%8E%E4%BB%A5%E5%A4%AA%E5%9D%8A%E5%8C%BA%E5%9D%97%E9%93%BE%E7%9A%84%E6%AF%94%E7%89%B9%E5%B8%81%E4%BB%A3%E5%B8%81-HBTC-%E7%9A%84%E5%85%AC%E5%91%8A)
- [elements](https://elementsproject.org)

trade-offs
- It is simple and easy to implement.
- It is not decentralized and insecure. Custodians need to be fully trusted. A malicious custodian can delay or block the withdraw,  even steal your asset.

#### Spv Based Bridge

If the two chains are both capable of verifying the spv proof of the counterparty chain, we can implement a spv based bridge. We setup cross-chain contract on both chains, lock asset in the contract on one chain, construct the spv proof and relay the proof to the contract on counterparty chain, the contract mint the amount of mirror token on counterparty chain. It is similar when you burn the mirror token and get your original asset back.

Projects using this schema includes:
- [Near Rainbow Bridge](https://github.com/near/rainbow-bridge-cli), NEAR <-> Ethereum Decentralized Bridge
- [waterloo](https://blog.kyber.network/waterloo-a-decentralized-practical-bridge-between-eos-and-ethereum-c25b1698f010), Bridge between EOS and Ethereum

trade-offs
- It is fully decentralized and secure. You don't need to trust anyone or anything except the two chains you want to move assets between.
- It depends on the programming capability of both chain to verify the spv proof, which is hard to satisfy in some cases. E.g. Bitcoin's script system is too weak to verify the spv proof of Ethereum.
- It is more complicated to implement. We have to develop contracts on both chains to maintain an on-chain light client of counterparty chain, verify the spv proof and handle the lock/unlock logic and off-chain modules to generate spv proof, relay the proof.

### Core Concepts

The core concepts of toCKB are:
- Because we want to support the chains lack of ability to verify CKB spv proof, we involve a new role called signer to replace the contract in spv based bridge. Users transfer their asset to signer's address instead of the contract to lock.
- Anyone can be signer by bonding an amount of CKB that ensures their behavior in the system remains honest. The exchange rate may changes over time, so the value of CKB have to be more than the locked asset for security. The collateral value percent is configurable, which is 150% in this stage.
- With the help of signer, we can handle both mint and redeem logic on CKB by spv proof verification:
    - When user locked their XAsset on XChain(which means they transfered their XAsset to signer), toCKB contract can verify the proof and mint XToken on CKB, owned by user.
    - When user burns their XToken on CKB, signer has to send the XAsset to user on XChain and submit the proof to CKB. toCKB contract verifies the proof, and redeem the bond of signers.
- There will be additional logics in toCKB contract to handle unexpected situations, e.g. signer incentive, signer fraud, undercollateralization caused by the exchange rate changes.

## Process

The normal process shows below:

![toCKB normal process](https://i.imgur.com/kMPOOOi.png)

<!--
```plantuml
autonumber
actor user
actor signer
database CKB
database XChain

== Mint XToken ==
user -> CKB: deposit request
signer -> CKB: bonding, provide a XChain receive addr
user -> XChain: transfer asset to signer
user -> CKB: use the proof of previous step to mint XToken
user -> user: user can use the XToken

== Redeem XAsset ==
user -> CKB: redeem request
signer -> XChain: signer send asset back to user
signer -> CKB: Withdraw Collateral
```
-->

1. User makes a deposit request on CKB, along with some pledge. If someone bonds as signer but the user does not follw up, the user will lose the pledge to compensate the loss of signer's CKB liquidity.
2. Someone bonds CKB to become a signer, provide a XChain address for user to deposit XAsset.
3. User transfers their XAsset to signer on XChain.
4. User generates the transaction proof and relays it to CKB, mints 1-to-1 CKB token -- XToken. Signer gets some percent of XToken as fee, e.g. 0.1%.
5. User can deal with the XToken as he wishes.
6. When user want to redeem XAsset on XChain, he can make a redeem request on CKB, burn his XToken on CKB via toCKB contract.
7. Signer sends the XAsset back to user on XChain.
8. Signer generates the transaction proof, relays it on CKB, and withdraws his collateral back.

### Handle Failture

There are two kinds of failure: abort and faulty.

Abort:
1. After users make a deposit request, nobody bonds their CKB to become the signer(step 2). Users can withdraw there pledge then.
2. After someone bonds CKB to become the signer, user does not send the XAsset to signer(step 3). Signer can withdraw his collateral and user's pledge.
3. During redeem process, signer does not send asset back to user within a limited time(step 7). Anyone can trigger liquidation.
4. The exchange rate changes, causes the CKB value bonded lower than expected. Anyone can trigger liquidation.

Faulty:
1. During the warranty period, signer spends the locked XAsset. Anyone can relay the proof to CKB to trigger liquidation.
2. During redeem period, signer transfers the XAsset to wrong address. Anyone can relay the proof to CKB to trigger liquidation.


In liquidation period, anyone can start an auction. The auction price decreases over time. People pays XT to buy the CKB bonded in the contract. User gets the XT to redeem XAsset from other signers.

The rules for the distribution of the remaining collateral from the auction are as follows:
1. If the auction is caused by abort, the user who triggers the liquidation shares the remaining collateral with the Signer.
2. If the auction is caused by faulty, the user who triggers the liquidation gets all remaining collateral.

## modules

### On-Chain

#### toCKB Contract

The toCKB contract mediates the lifecycle of the deposit, including:
- Deposit Request
- Bonding
- Withdraw Pledge
- Withdraw Pledge and Collateral
- Pre-term Redeem
- At-term Redeem
- Withdraw Collateral
- Liquidation：SignerTimeout
- Redeem: Pre-Undercollateral
- Liquidation：Undercollateral
- Liquidation: FaultyWhenWarranty
- Liquidation: FaultyWhenRedeeming
- Auction: SignerTimeout
- Auction: FaultyWhenWarranty
- Auction: FaultyWhenRedeeming

#### Price Oracle

The toCKB system relies on a price oracle who provides the CKB/XAsset price to remain security.

We are still working on design and implementation on that. The possible solutions:
- A [MakerDAO's price feed](https://developer.makerdao.com/feeds/) like oracle, which gets price from multiple public APIs.
- Use some decentralized oracle implemented on CKB.
- If there are some uniswap like DEX running on CKB, we can use it as price feed.
- [Reuse the data on Ethereum](https://talk.nervos.org/t/hi-ckb/4890).

#### The X Specific SPV Verification

For different XChain, we have to implement the spv verification logic on CKB. Due to the flexible design of CKB-VM, it will be easy to reuse a lot of previous work.

### Off-Chain

In this stage, we will make some scripts or command line tools to help users and signers construct transactions, monitor the chain events and even trigger the follow up actions automatically.

There are 3 modules in plan right now:
- A user client for users to deal with the deposit request, mint XT and other actions.
- A signer client for signers to deal with bonding, redemption and other actions.
- A guard service for users who want to get profits by watching frand behaviors on CKB and triggering the liquidation.
