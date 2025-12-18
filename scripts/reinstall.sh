#!/bin/sh

cd ..
cargo clean

rm -rf Cargo.lock
cargo update