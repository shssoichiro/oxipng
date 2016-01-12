# OxiPng

## Overview

OxiPng is a PNG compression optimizer.

In the typical use case, OxiPng recompresses PNG image files
losslessly and performs PNG integrity checks and corrections.
Future implementation of other features is planned.

## History

OxiPng began as a completely rewrite of the OptiPNG project,
which is assumed to be dead as no commit has been made to it since 2013.
The name has been changed to avoid confusion and potential legal issues.

The core goal of rewriting OptiPNG was to implement multithreading,
which would be very difficult to do within the existing C codebase of OptiPNG.
This also served as an opportunity to choose a more modern, safer language (Rust).

## Building

Building OxiPng can be done using the latest stable or nightly Rust with Cargo installed, as follows:
```
git clone https://github.com/shssoichiro/oxipng.git
cd oxipng
cargo build --release
cp target/release/oxipng /usr/local/bin
```
Please note that zlib is a required build dependency. OxiPng should work with any 1.x version of zlib,
but you are advised to use the latest version (currently 1.2.8) for security and bug fixes.

OxiPng follows Semantic Versioning.

## Usage

OxiPng is a command-line utility. Basic usage looks similar to the following:

```
oxipng -o 4 -i 1 --strip *.png
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
* Strip: Used to remove metadata info from processed images. Used via `--strip` or `-s`.
Can save a few kilobytes if you don't need the metadata.

More advanced options can be found by running `oxipng -h`.

## Contributing

Any contributions are welcome and will be accepted via pull request on GitHub. Bug reports can be
filed via GitHub issues. If you have the capability to submit a fix with the bug report, it is
preferred that you do so via pull request, however you do not need to be a Rust programmer to
submit a bug report. Other contributions (such as improving documentation or translations)
are also welcome via GitHub.

## License

OxiPng is open-source software, distributed under the MIT license.
