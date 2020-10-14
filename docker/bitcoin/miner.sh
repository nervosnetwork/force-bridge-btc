#!/bin/bash

BLOCKTIME=2

while true
  do
    bitcoin-cli -conf=/etc/bitcoin/bitcoin.conf generatetoaddress 1 bcrt1q4r9hqljdpfwxu6gp3x7qqedg77r6408dn4wmnf
    sleep $BLOCKTIME
  done
