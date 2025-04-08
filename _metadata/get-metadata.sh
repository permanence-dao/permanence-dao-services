#!/usr/bin/env bash
set -e
subxt metadata --url "wss://rpc.helikon.io/polkadot" --pallets Utility,Proxy,ConvictionVoting -o ./polkadot-metadata.scale
subxt metadata --url "wss://rpc.helikon.io/kusama" --pallets Utility,Proxy,ConvictionVoting -o ./kusama-metadata.scale