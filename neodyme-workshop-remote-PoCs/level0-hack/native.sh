#!/bin/bash
cargo build-bpf --manifest-path=./Cargo.toml --bpf-out-dir=./target/so
solana program deploy ./target/so/level0.so
RUST_BACKTRACE=1 cargo run --bin native
