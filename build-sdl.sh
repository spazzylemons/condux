#!/bin/bash
set -e
# make sure we're running where the app is
cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"/condux-app
# invoke command to build
cargo $@
