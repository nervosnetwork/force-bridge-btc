#!/bin/bash

export PATH=/bitcoin-0.20.0/bin:$PATH

bitcoind -conf=/etc/bitcoin/bitcoin.conf -rpcport=18443
sleep 8

bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet user
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="user" importprivkey "cUDfdzioB3SqjbN9vutRTUrpw5EH9srrg6RPibacPo1fGHpfPKqL"
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet signer
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="signer" importprivkey "cU9PYTnSkcWoAE15U26JJCwtKiYvTCKYdbWt8e7ovidEGDBwJQ5x"
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 201 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf

./miner.sh >/dev/null &

/bin/bash $@
