#!/bin/sh

set -eu

./bundle.sh > /tmp/submit.rs

oj submit https://atcoder.jp/contests/ahc003/tasks/ahc003_a /tmp/submit.rs -y
