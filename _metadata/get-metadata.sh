#!/usr/bin/env bash
set -e
subxt metadata --url "wss://rpc.helikon.io/polkadot" --pallets Utility,Proxy,ConvictionVoting,Referenda,Preimage -o ./polkadot-metadata.scale
subxt metadata --url "wss://rpc.helikon.io/kusama" --pallets Utility,Proxy,ConvictionVoting,Referenda,Preimage -o ./kusama-metadata.scale