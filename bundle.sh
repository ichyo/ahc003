#!/bin/sh

cd ./submission
cargo equip --exclude-atcoder-crates --rustfmt --remove docs --resolve-cfgs | grep -ve '^//!'
