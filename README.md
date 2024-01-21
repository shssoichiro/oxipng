# Oxipng

[![Build Status](https://github.com/shssoichiro/oxipng/workflows/oxipng/badge.svg)](https://github.com/shssoichiro/oxipng/actions?query=branch%3Amaster)
[![Version](https://img.shields.io/crates/v/oxipng.svg)](https://crates.io/crates/oxipng)
[![License](https://img.shields.io/crates/l/oxipng.svg)](https://github.com/shssoichiro/oxipng/blob/master/LICENSE)
[![Docs](https://docs.rs/oxipng/badge.svg)](https://docs.rs/oxipng)

## Overview

Oxipng is a multithreaded lossless PNG/APNG compression optimizer. It can be used via a command-line
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

Oxipng is a command-line utility. An example usage, suitable for web, may be the following:

```
oxipng -o 4 --strip safe --alpha *.png
```

The most commonly used options are as follows:

- Optimization: `-o 0` through `-o 6` (or `-o max`), lower is faster, higher is better compression.
  The default (`-o 2`) is quite fast and provides good compression. Higher levels can be notably
  better but generally have increasingly diminishing returns.
- Strip: Used to remove metadata info from processed images. Used via `--strip [safe,all]`.
  Can save a few kilobytes if you don't need the metadata. "Safe" removes only metadata that
  will never affect rendering of the image. "All" removes all metadata that is not critical
  to the image. You can also pass a comma-separated list of specific metadata chunks to remove.
  `-s` can be used as a shorthand for `--strip safe`.
- Alpha: `--alpha` can improve compression of images with transparency, by altering the color
  values of fully transparent pixels. This is generally recommended, but take care as this is
  technically a lossy transformation and may be unsuitable for some specific applications.

More advanced options can be found by running `oxipng --help`, or viewed [here](MANUAL.txt).

Some options have both short (`-a`) and long (`--alpha`) forms. Which form you use is just a
matter of preference. Multiple short options can be combined together, e.g.:
`-savvo6` is equivalent to to `--strip safe --alpha --verbose --verbose --opt 6`.
Note that all options are case-sensitive.

## Git integration via [pre-commit]

Add this to your `.pre-commit-config.yaml`

```yaml
-   repo: https://github.com/shssoichiro/oxipng
    rev: v9.0.0
    hooks:
    -   id: oxipng
        args: ["-o", "4", "--strip", "safe", "--alpha"]
```
[pre-commit]: https://pre-commit.com/

## Git integration via [Trunk]

[Trunk] is an extendable superlinter which can be used to run `oxipng` to automatically optimize `png`s when committing them into a git repo, or to gate any `png`s being added to a git repo on whether they are optimized. The [trunk] oxipng integration is [here](https://github.com/trunk-io/plugins/tree/main/linters/oxipng).

To enable oxipng via [trunk]:

```bash
# to get the latest version:
trunk check enable oxipng

# to get a specific version:
trunk check enable oxipng@9.0.0
```

or modify `.trunk/trunk.yaml` in your repo to contain:

```
lint:
  enabled:
    - oxipng@9.0.0
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
[Options struct](https://docs.rs/oxipng/latest/oxipng/struct.Options.html) and
passing it, along with an input filename, into the
[optimize function](https://docs.rs/oxipng/latest/oxipng/fn.optimize.html).

It is recommended to disable the "binary" feature when including oxipng as a library. Currently, there is
no simple way to just disable one feature in Cargo, it has to be done by disabling default features
and specifying the desired ones, for example:
`oxipng = { version = "9.0", features = ["parallel", "zopfli", "filetime"], default-features = false }`

## History

Oxipng began as a complete rewrite of the OptiPNG project,
which was assumed to be dead as no commit had been made to it since March 2014.
(OptiPNG has since released a new version, after Oxipng was first released.)
The name has been changed to avoid confusion and potential legal issues.

The core goal of rewriting OptiPNG was to implement multithreading,
which would be very difficult to do within the existing C codebase of OptiPNG.
This also served as an opportunity to choose a more modern, safer language (Rust).

Note that, while similar, Oxipng is not a drop-in replacement for OptiPNG.
If you are migrating from OptiPNG, please check the [help](MANUAL.txt) before using.

## Contributing

Any contributions are welcome and will be accepted via pull request on GitHub. Bug reports can be
filed via GitHub issues. Please include as many details as possible. If you have the capability
to submit a fix with the bug report, it is preferred that you do so via pull request,
however you do not need to be a Rust developer to contribute.
Other contributions (such as improving documentation or translations) are also welcome via GitHub.

## License

Oxipng is open-source software, distributed under the MIT license.

## Benchmarks

Tested OxiPNG 9.0.0 (commit `c16519b38b0519988db625913be919d4f0e42f5d`, compiled
on `rustc 1.74.0-nightly (7b4d9e155 2023-09-28)`) against OptiPNG version 0.7.7,
as packaged by Debian unstable, on a Linux 6.5.0-2-amd64 kernel, Intel Core
i7-12700 CPU (8 performance cores, 4 efficiency cores, 20 threads), DDR5-5200
RAM in dual channel configuration.

```

Benchmark 1: ./target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):      59.6 ms ±   7.7 ms    [User: 77.4 ms, System: 3.6 ms]
  Range (min … max):    53.3 ms …  89.9 ms    32 runs

Benchmark 2: optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     132.4 ms ±   0.8 ms    [User: 132.5 ms, System: 0.6 ms]
  Range (min … max):   131.8 ms … 134.4 ms    22 runs

Summary
  ./target/release/oxipng -P ./tests/files/rgb_16_should_be_grayscale_8.png ran
    2.22 ± 0.29 times faster than optipng -simulate ./tests/files/rgb_16_should_be_grayscale_8.png

Benchmark 1: ./target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):      88.7 ms ±   4.3 ms    [User: 270.3 ms, System: 11.0 ms]
  Range (min … max):    86.8 ms … 109.4 ms    26 runs

Benchmark 2: optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png
  Time (mean ± σ):     444.9 ms ±   0.3 ms    [User: 444.8 ms, System: 0.7 ms]
  Range (min … max):   444.4 ms … 445.6 ms    10 runs

Summary
  ./target/release/oxipng -o4 -P ./tests/files/rgb_16_should_be_grayscale_8.png ran
    5.01 ± 0.25 times faster than optipng -o 4 -simulate ./tests/files/rgb_16_should_be_grayscale_8.png

```

<details>
<summary>Older benchmark</summary>

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
</details>
