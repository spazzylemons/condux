#!/bin/bash
set -e
# get output location
OUTPUT=$(pwd)/$1
# make sure we're running where this shell script is
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"
# invoke command to build
cargo build --release
# invoke elf2dol on compiled binary
cp target/release/condux $OUTPUT
