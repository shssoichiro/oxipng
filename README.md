# OptiPNG-Next

## Overview

OptiPNG-Next version 2 is a completely rewrite of the OptiPNG project,
which is assumed to be dead as no commit has been made to it since 2013.

OptiPNG is a PNG compression optimizer.

In the typical use case, OptiPNG recompresses PNG image files
losslessly, converts external formats (BMP, GIF, PNM and TIFF) to
optimized PNG, and performs PNG integrity checks and corrections.
At the user's explicit request, OptiPNG is also capable to alter
image data or remove metadata.

## Building

Building OptiPNG-Next can be done using Rust 1.5.0 or greater with Cargo installed, as follows:
```
git clone https://github.com/shssoichiro/optipng-next.git
cd optipng-next
cargo build --release
cp target/release/optipng /usr/local/bin
```
Please note that zlib is a required build dependency. OptiPNG should work with any 1.x version of zlib,
but you are advised to use the latest version (currently 1.2.8) for security and bug fixes.

## Usage

OptiPNG-Next is a command-line utility. Basic usage looks similar to the following:

```
optipng -o4 -i 1 -strip all *.png
```

The most commonly used options are as follows:
* Optimization: `-o1` through `-o6`, lower is faster, higher is better compression.
The default (`-o2`) is sufficiently fast on a modern CPU and provides 30-50% compression
gains over an unoptimized PNG. `-o4` is 6 times slower than `-o2` but can provide 5-10%
extra compression over `-o2`. Using any setting higher than `-o4` is generally unlikely
to give any extra compression gains and is not recommended.
* Interlacing: `-i 1` will enable [Adam7](https://en.wikipedia.org/wiki/Adam7_algorithm)
PNG interlacing on any images that are processed. `-i 0` will remove interlacing from all
processed images. Not specifying either will keep the same interlacing state as the
input image. Note: Interlacing can add 25-50% to the size of an optimized image. Only use
it if you have a good reason.
* Strip: Used to remove metadata info from processed images. Generally used as `-strip all`.
Can save a few kilobytes if you don't need the metadata.

More advanced options can be found in the man page or by running `optipng -h`.

## Changes

OptiPNG Next version 2 attempts to maintain functionality of the original OptiPNG as much as possible,
although command line usage may have changed (OptiPNG Next follows semantic versioning).

As version 2 is in alpha, there are still some features that are missing. All features
that were available in the original will be implemented in OptiPNG version 2 before it moves into
beta.

## Contributing

Any contributions are welcome and will be accepted via pull request on GitHub. Bug reports can be
filed via GitHub issues. If you have the capability to submit a fix with the bug report, it is
preferred that you do so via pull request, however you do not need to be a Rust programmer to
submit a bug report. Other contributions (such as improving documentation or translations)
are also welcome via GitHub.

## License

OptiPNG Next is open-source software, distributed under the MIT license. (Version 2 is a complete rewrite and shares no
code with the original OptiPNG, which is under the zlib license.)
