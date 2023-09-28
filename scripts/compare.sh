#!/bin/bash
cargo build --release
sed -i.bak '/## Benchmarks/,$d' README.md
rm README.md.bak

CORES=$(sysctl -n hw.ncpu 2>/dev/null || grep -c ^processor /proc/cpuinfo)
CPU=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || grep '^model name' /proc/cpuinfo | sed 's/model name.\+: //g' | head -n 1)
OXIPNG_VERSION=$(./target/release/oxipng -V)
OPTIPNG_VERSION=$(optipng -v | head -n 1)
RUST_VERSION=$(rustc -V)
echo -e '## Benchmarks\n' >> README.md
echo "Tested $OXIPNG_VERSION (compiled on $RUST_VERSION) against $OPTIPNG_VERSION on $CPU with $CORES logical cores" >> README.md
echo -e '\n```\n' >> README.md

hyperfine --style basic --warmup 5 './target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png' 'optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png' >> README.md
echo -e '\n\n' >> README.md
hyperfine --style basic --warmup 5 './target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png' 'optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png' >> README.md

echo -e '\n```' >> README.md
