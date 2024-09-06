#!/usr/bin/env bash

set -e
while :
do
	cargo clean
	cargo test --release --features loadjvm
done
