#!/bin/bash

export PATH=/bitcoin-0.20.0/bin:$PATH

bitcoind -conf=/etc/bitcoin/bitcoin.conf -rpcport=18443
sleep 8

bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="" importprivkey "cURtxPqTGqaA5oLit5sMszceoEAbiLFsTRz7AHo23piqamtxbzav"
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet user
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="Alice" importprivkey "cUDfdzioB3SqjbN9vutRTUrpw5EH9srrg6RPibacPo1fGHpfPKqL"
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf createwallet signer
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="Bob" importprivkey "cU9PYTnSkcWoAE15U26JJCwtKiYvTCKYdbWt8e7ovidEGDBwJQ5x"
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 1 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 101 bcrt1q0yszr82fk9q8tu9z9ddxxvwqmlrdycsy378znz
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf -rpcwallet="" sendtoaddress bcrt1qfzdcp53u29yt9u5u3d0sx3u2f5xav7sqatfxm2 0.2
bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 101 bcrt1q0yszr82fk9q8tu9z9ddxxvwqmlrdycsy378znz

./miner.sh >/dev/null &

/bin/bash $@
