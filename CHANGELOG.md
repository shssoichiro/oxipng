**Version 0.6.1 (unreleased)**
 - Minor compression improvement on interlaced images

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
