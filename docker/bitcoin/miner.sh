#!/bin/bash

BLOCKTIME=2

while true
  do
    bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 1 bcrt1q0yszr82fk9q8tu9z9ddxxvwqmlrdycsy378znz
    sleep $BLOCKTIME
  done
