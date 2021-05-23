#!/bin/sh

cd ./submission
cargo equip --exclude-atcoder-crates --rustfmt --remove docs --remove comments
