#!/bin/bash

# This script is for quick building & deploying of the program.
# It also serves as a reference for the commands used for building & deploying Solana programs.
# Run this bad boy with "bash cicd.sh" or "./cicd.sh"

cargo build-bpf --manifest-path=./level0/Cargo.toml --bpf-out-dir=./target/so
solana program deploy ./target/so/level0.so
RUST_BACKTRACE=1 cargo run --manifest-path=./pocs/Cargo.toml --target-dir=./target/
#IF error, replace the program ID in the poc.rs with the correct
solana program show --programs
