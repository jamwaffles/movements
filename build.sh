#!/bin/bash

set -ex

cargo fmt --all -- --check

cargo test --release

cargo bench --no-run

cargo +nightly doc --all-features --document-private-items

linkchecker target/doc/gcode_parser/index.html
