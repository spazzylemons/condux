#!/bin/bash
set -e
# get output location
OUTPUT=$(pwd)/$1
# make sure we're running where this shell script is
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"
# add linker to path
export PATH=$PATH:$DEVKITARM/bin
# invoke command to build
cargo +nightly 3ds build --release
# invoke elf2dol on compiled binary
cp target/armv6k-nintendo-3ds/release/condux.3dsx $OUTPUT
