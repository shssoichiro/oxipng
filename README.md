# Oxipng

[![Build Status](https://travis-ci.org/shssoichiro/oxipng.svg?branch=master)](https://travis-ci.org/shssoichiro/oxipng)
[![Version](https://img.shields.io/crates/v/oxipng.svg)](https://crates.io/crates/oxipng)
[![License](https://img.shields.io/crates/l/oxipng.svg)](https://github.com/shssoichiro/oxipng/blob/master/LICENSE)

## Overview

Oxipng is a lossless PNG compression optimizer.

**Note:** This package should be considered a beta. Although there are many tests in place,
there is still a chance of data loss or corruption. You should backup your files before
using this tool, unless you are very brave.

If you encounter an issue, please report it via the GitHub issues tab. Include as many details
as possible.

## Installing

Oxipng can be installed from Cargo, via the following command:
```
cargo install oxipng
```

Alternatively, oxipng can be built from source using the latest stable or nightly Rust:
```
git clone https://github.com/shssoichiro/oxipng.git
cd oxipng
cargo build --release
cp target/release/oxipng /usr/local/bin
```

Please note that zlib is a required build dependency. Oxipng should work with any 1.x version of zlib,
but you are advised to use the latest version (currently 1.2.8) for security and bug fixes.

The current minimum supported Rust version is **1.6.0**. Oxipng may compile on earlier versions of Rust,
but there is no guarantee.

Oxipng follows Semantic Versioning.

## Usage

Oxipng is a command-line utility. Basic usage looks similar to the following:

```
oxipng -o 4 -i 1 --strip safe *.png
```

The most commonly used options are as follows:
* Optimization: `-o 1` through `-o 6`, lower is faster, higher is better compression.
The default (`-o 2`) is sufficiently fast on a modern CPU and provides 30-50% compression
gains over an unoptimized PNG. `-o 4` is 6 times slower than `-o 2` but can provide 5-10%
extra compression over `-o 2`. Using any setting higher than `-o 4` is unlikely
to give any extra compression gains and is not recommended.
* Interlacing: `-i 1` will enable [Adam7](https://en.wikipedia.org/wiki/Adam7_algorithm)
PNG interlacing on any images that are processed. `-i 0` will remove interlacing from all
processed images. Not specifying either will keep the same interlacing state as the
input image. Note: Interlacing can add 25-50% to the size of an optimized image. Only use
it if you have a good reason.
* Strip: Used to remove metadata info from processed images. Used via `--strip [safe,all]`.
Can save a few kilobytes if you don't need the metadata. "Safe" removes only metadata that
will never affect rendering of the image. "All" removes all metadata that is not critical
to the image. You can also pass a comma-separated list of specific metadata chunks to remove.

More advanced options can be found by running `oxipng -h`.

## History

Oxipng began as a complete rewrite of the OptiPNG project,
which is assumed to be dead as no commit has been made to it since March 2014.
The name has been changed to avoid confusion and potential legal issues.

The core goal of rewriting OptiPNG was to implement multithreading,
which would be very difficult to do within the existing C codebase of OptiPNG.
This also served as an opportunity to choose a more modern, safer language (Rust).

## Contributing

Any contributions are welcome and will be accepted via pull request on GitHub. Bug reports can be
filed via GitHub issues. If you have the capability to submit a fix with the bug report, it is
preferred that you do so via pull request, however you do not need to be a Rust programmer to
contribute. Other contributions (such as improving documentation or translations) are also
welcome via GitHub.

## License

Oxipng is open-source software, distributed under the MIT license.
