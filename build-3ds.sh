#!/bin/bash
set -e
# make sure we're running where the app is
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"/condux-app
# add linker to path
export PATH=$PATH:$DEVKITARM/bin
# invoke command to build
cargo +nightly 3ds $@
