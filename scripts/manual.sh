#!/bin/bash
cargo build

./target/debug/oxipng -V > MANUAL.txt
./target/debug/oxipng --help >> MANUAL.txt
