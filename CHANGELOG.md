**Version 0.13.2 (unreleased)**
 - Performance optimizations

**Version 0.13.1**
 - Bump regex dependency to 0.2
 - Bump byteorder dependency to 1.0
 - Bump rayon dependency to 0.6

**Version 0.13.0**
 - Fix bug in certain PNG headers when reducing color type ([#52](https://github.com/shssoichiro/oxipng/issues/52))
 - [SEMVER_MAJOR] Reduction functions now take `&mut PngData` and return a `bool` indicating whether the image was reduced
 - [SMEVER_MAJOR] Bump minimum required rustc version to 1.12.0

**Version 0.12.0**
 - Performance optimizations
 - Fix processing filenames that contain commas (@aliceatlas [#50](https://github.com/shssoichiro/oxipng/pull/50))
 - [SEMVER_MINOR] Add zopfli option (-Z), disabled by default. Gives about 10% better compression, but is currently 50-100x slower.

**Version 0.11.0**
 - [SEMVER_MAJOR] Bump minimum rustc version to 1.9.0, required by dependencies
 - [SEMVER_MINOR] Allow calling optimization presets via crate using `Options::from_preset`
 - [SEMVER_MAJOR] Return proper `PngError` type which implements `std::error::Error` from `Result`s
 - [SEMVER_MAJOR] Rename module `deflate::deflate` to `deflate`
 - Performance optimizations

**Version 0.10.0**
 - [SEMVER_MINOR] Make clap and regex dependencies optional
   - Enabled by default, needed for executable build; can be disabled for use in crates
 - Remove reduction from palette to grayscale, which was not working and provided minimal benefit

**Version 0.9.0**
 - [SEMVER_MAJOR] Significant refactoring of modules
 - Use `itertools` to cleanup areas of code
 - Use multiple threads for filtering trials

**Version 0.8.2**
 - Fix issue where images smaller than 4px width would crash on interlacing ([#42](https://github.com/shssoichiro/oxipng/issues/42))

**Version 0.8.1**
 - Minor optimizations
 - Fix issue where interlaced images with certain widths would fail to optimize

**Version 0.8.0**
 - [SEMVER_MINOR] Add support for optimizing PNGs already loaded into memory via library function

**Version 0.7.0**
 - Minor compression improvement on interlaced images
 - Performance optimizations
 - [SEMVER_MINOR] Move default Options into a Default impl
 - [SEMVER_MINOR] Add option for setting number of threads ([#39](https://github.com/shssoichiro/oxipng/issues/39))

**Version 0.6.0**
 - Fix issue where output directory would not be created if it did not exist
 - Use miniz for compression strategies where it outperforms zlib
 - [SEMVER_MINOR] Partially implement -p / --preserve, as far as stable Rust will allow for now
 - [SEMVER_MINOR] Implement --fix to ignore CRC errors and recalculate correct CRC in output

**Version 0.5.0**
 - [SEMVER_MINOR] Palette entries can now reduced, on by default ([#11](https://github.com/shssoichiro/oxipng/issues/11))
 - Don't report that we are in pretend mode if verbosity is set to none
 - Add cargo bench suite ([#7](https://github.com/shssoichiro/oxipng/issues/7))

**Version 0.4.0**
 - Performance optimizations
 - [SEMVER_MAJOR] `-s` automatically infers `--strip safe` ([#31](https://github.com/shssoichiro/oxipng/issues/31))
 - Update byteorder and clap crates
 - Fix issue where interlaced images incorrectly applied filters on the first line of a pass

**Version 0.3.0**
 - Properly decode interlaced images
 - [SEMVER_MINOR] Allow converting between progressive and interlaced images ([#3](https://github.com/shssoichiro/oxipng/issues/3))
 - Fix a bug that would cause oxipng to crash on very small images

**Version 0.2.2**
 - Limit number of threads to 1.5x number of cores
 - Significantly improve memory usage, especially with high optimization levels. ([#32](https://github.com/shssoichiro/oxipng/issues/32))
 - Refactor output code ([#19](https://github.com/shssoichiro/oxipng/issues/19))

**Version 0.2.1**
 - Add rustdoc for public methods and structs
 - Improve filter mode 5 heuristic ([#16](https://github.com/shssoichiro/oxipng/issues/16))
 - Add tests for edge-case images with subtitles ([#29](https://github.com/shssoichiro/oxipng/issues/29))

**Version 0.2.0**
 - Fix program version that is displayed when running `oxipng -V`
 - Ensure `--quiet` mode is actually quiet (@SethDusek [#20](https://github.com/shssoichiro/oxipng/pull/20))
 - Write status/debug information to stderr instead of stdout
 - Use heuristics to determine best combination for `-o1` ([#21](https://github.com/shssoichiro/oxipng/issues/21))
 - [SEMVER_MAJOR] Allow 'safe', 'all', or comma-separated list as options for `--strip`
 - [SEMVER_MINOR] Add `-s` alias for `--strip`

**Version 0.1.1**
 - Fix `oxipng *` writing all input files to one output file ([#15](https://github.com/shssoichiro/oxipng/issues/15))

**Version 0.1.0**
 - Initial beta release
 - Reduce color type and bit depth
 - Recompress with zlib
 - Multithreading
 - Strip headers option
 - Backup file before writing option
 - Write to stdout option
