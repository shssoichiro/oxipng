# Oxipng

[![Build Status](https://github.com/shssoichiro/oxipng/workflows/oxipng/badge.svg)](https://github.com/shssoichiro/oxipng/actions?query=branch%3Amaster)
[![Version](https://img.shields.io/crates/v/oxipng.svg)](https://crates.io/crates/oxipng)
[![License](https://img.shields.io/crates/l/oxipng.svg)](https://github.com/shssoichiro/oxipng/blob/master/LICENSE)
[![Docs](https://docs.rs/oxipng/badge.svg)](https://docs.rs/oxipng)

## Overview

Oxipng is a multithreaded lossless PNG compression optimizer. It can be used via a command-line
interface or as a library in other Rust programs.

## Installing

Oxipng for Windows can be downloaded from the [Releases](https://github.com/shssoichiro/oxipng/releases) link on the GitHub page.

For MacOS or Linux, it is recommended to install from your distro's package repository, if possible.

Alternatively, oxipng can be installed from Cargo, via the following command:
```
cargo install oxipng
```

Oxipng can be built from source using the latest stable or nightly Rust.
This is primarily useful for developing on oxipng.
```
git clone https://github.com/shssoichiro/oxipng.git
cd oxipng
cargo build --release
cp target/release/oxipng /usr/local/bin
```

The current minimum supported Rust version is **1.47.0**.

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
it if you believe the benefits outweigh the costs for your use case.
* Strip: Used to remove metadata info from processed images. Used via `--strip [safe,all]`.
Can save a few kilobytes if you don't need the metadata. "Safe" removes only metadata that
will never affect rendering of the image. "All" removes all metadata that is not critical
to the image. You can also pass a comma-separated list of specific metadata chunks to remove.
`-s` can be used as a shorthand for `--strip safe`.

More advanced options can be found by running `oxipng -h`.

## Library Usage

Although originally intended to be used as an executable, oxipng can also be used as a library in
other Rust projects. To do so, simply add oxipng as a dependency in your Cargo.toml,
then `extern crate oxipng` in your project. You should then have access to all of the library
functions [documented here](https://docs.rs/oxipng). The simplest
method of usage involves creating an
[Options struct](https://docs.rs/oxipng/3.0.1/oxipng/struct.Options.html) and
passing it, along with an input filename, into the
[optimize function](https://docs.rs/oxipng/3.0.1/oxipng/fn.optimize.html).

## History

Oxipng began as a complete rewrite of the OptiPNG project,
which was assumed to be dead as no commit had been made to it since March 2014.
(OptiPNG has since released a new version, after Oxipng was first released.)
The name has been changed to avoid confusion and potential legal issues.

The core goal of rewriting OptiPNG was to implement multithreading,
which would be very difficult to do within the existing C codebase of OptiPNG.
This also served as an opportunity to choose a more modern, safer language (Rust).

## Contributing

Any contributions are welcome and will be accepted via pull request on GitHub. Bug reports can be
filed via GitHub issues. Please include as many details as possible. If you have the capability
to submit a fix with the bug report, it is preferred that you do so via pull request,
however you do not need to be a Rust developer to contribute.
Other contributions (such as improving documentation or translations) are also welcome via GitHub.

## License

Oxipng is open-source software, distributed under the MIT license.

## Benchmarks

Tested oxipng 3.0.1 (compiled on rustc 1.45.0 (5c1f21c3b 2020-07-13)) against OptiPNG version 0.7.7 on Intel(R) Core(TM) i7-6700HQ CPU @ 2.60GHz with 8 logical cores


```

Benchmark #1: ./target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     101.6 ms ±   5.9 ms    [User: 261.6 ms, System: 16.5 ms]
  Range (min … max):    94.7 ms … 113.3 ms    28 runs
 
Benchmark #2: optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     294.0 ms ±   3.5 ms    [User: 293.5 ms, System: 0.6 ms]
  Range (min … max):   290.4 ms … 300.5 ms    10 runs
 
Summary
  './target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png' ran
    2.89 ± 0.17 times faster than 'optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png'



Benchmark #1: ./target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     131.6 ms ±   6.1 ms    [User: 509.8 ms, System: 17.1 ms]
  Range (min … max):   123.5 ms … 143.6 ms    23 runs
 
Benchmark #2: optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     995.7 ms ±   8.0 ms    [User: 993.6 ms, System: 1.7 ms]
  Range (min … max):   987.5 ms … 1012.8 ms    10 runs
 
Summary
  './target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png' ran
    7.57 ± 0.35 times faster than 'optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png'

```

