#!/bin/bash
cargo build
cargo xstask mangen

./target/debug/oxipng -V > MANUAL.txt
#Redirect all streams to prevent detection of the terminal width and force an internal default of 100
./target/debug/oxipng --help >> MANUAL.txt 2>/dev/null </dev/null
