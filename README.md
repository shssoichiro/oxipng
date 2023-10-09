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

The current minimum supported Rust version is **1.66.0**.

Oxipng follows Semantic Versioning.

## Usage

Oxipng is a command-line utility. Basic usage looks similar to the following:

```
oxipng -o 4 -i 1 --strip safe *.png
```

The most commonly used options are as follows:

- Optimization: `-o 1` through `-o 6`, lower is faster, higher is better compression.
  The default (`-o 2`) is sufficiently fast on a modern CPU and provides 30-50% compression
  gains over an unoptimized PNG. `-o 4` is 6 times slower than `-o 2` but can provide 5-10%
  extra compression over `-o 2`. Using any setting higher than `-o 4` is unlikely
  to give any extra compression gains and is not recommended.
- Interlacing: `-i 1` will enable [Adam7](https://en.wikipedia.org/wiki/Adam7_algorithm)
  PNG interlacing on any images that are processed. `-i 0` will remove interlacing from all
  processed images. Not specifying either will keep the same interlacing state as the
  input image. Note: Interlacing can add 25-50% to the size of an optimized image. Only use
  it if you believe the benefits outweigh the costs for your use case.
- Strip: Used to remove metadata info from processed images. Used via `--strip [safe,all]`.
  Can save a few kilobytes if you don't need the metadata. "Safe" removes only metadata that
  will never affect rendering of the image. "All" removes all metadata that is not critical
  to the image. You can also pass a comma-separated list of specific metadata chunks to remove.
  `-s` can be used as a shorthand for `--strip safe`.

More advanced options can be found by running `oxipng -h`.

## Git integration via [Trunk]

[Trunk] is an extendable superlinter which can be used to run `oxipng` to automatically optimize `png`s when committing them into a git repo, or to gate any `png`s being added to a git repo on whether they are optimized. The [trunk] oxipng integration is [here](https://github.com/trunk-io/plugins/tree/main/linters/oxipng).

To enable oxipng via [trunk]:

```bash
# to get the latest version:
trunk check enable oxipng

# to get a specific version:
trunk check enable oxipng@8.0.0
```

or modify `.trunk/trunk.yaml` in your repo to contain:

```
lint:
  enabled:
    - oxipng@8.0.0
```

Then just run:

```bash
# to optimize a png:
trunk fmt <file>

# to check if a png is already optimized:
trunk check <file>
```

You can setup [trunk] to [manage your git hooks](https://docs.trunk.io/docs/actions-git-hooks) and automatically optimize any `png`s you commit to git, _when_ you `git commit`. To enable this, run:

```bash
trunk actions enable trunk-fmt-pre-commit
```

[trunk]: https://docs.trunk.io

## Library Usage

Although originally intended to be used as an executable, oxipng can also be used as a library in
other Rust projects. To do so, simply add oxipng as a dependency in your Cargo.toml,
then `extern crate oxipng` in your project. You should then have access to all of the library
functions [documented here](https://docs.rs/oxipng). The simplest
method of usage involves creating an
[Options struct](https://docs.rs/oxipng/3.0.1/oxipng/struct.Options.html) and
passing it, along with an input filename, into the
[optimize function](https://docs.rs/oxipng/3.0.1/oxipng/fn.optimize.html).

It is recommended to disable the "binary" feature when including oxipng as a library. Currently, there is
no simple way to just disable one feature in Cargo, it has to be done by disabling default features
and specifying the desired ones, for example:
`oxipng = { version = "8.0", features = ["parallel", "zopfli", "filetime"], default-features = false }`

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

Tested oxipng 5.0.0 (compiled on rustc 1.55.0-nightly (7a16cfcff 2021-07-11)) against OptiPNG version 0.7.7 on AMD Ryzen 7 4800H with Radeon Graphics with 16 logical cores

```

Benchmark #1: ./target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     128.8 ms ±  14.2 ms    [User: 296.0 ms, System: 14.3 ms]
  Range (min … max):    98.8 ms … 152.3 ms    21 runs

Benchmark #2: optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     254.2 ms ±  16.0 ms    [User: 252.8 ms, System: 1.2 ms]
  Range (min … max):   208.4 ms … 263.8 ms    14 runs

Summary
  './target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png' ran
    1.97 ± 0.25 times faster than 'optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png'



Benchmark #1: ./target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     141.4 ms ±  14.9 ms    [User: 611.7 ms, System: 21.1 ms]
  Range (min … max):   100.2 ms … 160.4 ms    23 runs

Benchmark #2: optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     730.0 ms ±  25.9 ms    [User: 728.0 ms, System: 1.2 ms]
  Range (min … max):   713.3 ms … 768.2 ms    10 runs

Summary
  './target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png' ran
    5.16 ± 0.58 times faster than 'optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png'

```
