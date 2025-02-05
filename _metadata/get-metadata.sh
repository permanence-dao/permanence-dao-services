#!/usr/bin/env bash
set -e
subxt metadata --url "wss://rpc.helikon.io/polkadot" --pallets Proxy,ConvictionVoting -o ./polkadot-metadata.scale
subxt metadata --url "wss://rpc.helikon.io/kusama" --pallets Proxy,ConvictionVoting -o ./kusama-metadata.scale