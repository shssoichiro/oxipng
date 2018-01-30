### Version 0.19.1 (unreleased)
 - Refactor of internal code.
 - Implement unix-specific permissions copying for `-p` option

### Version 0.19.0
 - [SEMVER_MAJOR] Default to overwriting the input file if `out_file` is not set.
 This does not affect the CLI, but with the library, it was easy to forget to set the `out_file`,
 and there was no warning that no output file would be written.
 - Bump dependencies, reduces binary size by a considerable amount
 - Hide all modules from documentation, and only export the specific structures that should be public.
 Previously there were too many implementation details made public. The modules are still public for the purposes of our integration tests,
 but we strongly advise against using undocumented modules. These may become private in the future.
 - Internal refactoring and code cleanup
 - Fix an error message that was displaying the wrong file path
 - Fix an issue where the output file would not be written if the input was already optimized,
 even if the output path was different from the input path

### Version 0.18.3
 - Return exit code of 1 if an error occurred while processing a file using the CLI app ([#93](https://github.com/shssoichiro/oxipng/issues/93))

### Version 0.18.2
 - Bump `image` to 0.18
 - Fix unfiltering of scan lines in interlaced images ([#92](https://github.com/shssoichiro/oxipng/issues/92))

### Version 0.18.1
 - Bump `rayon` to 0.9
 - Fix failure to optimize on certain grayscale images ([#89](https://github.com/shssoichiro/oxipng/issues/89))

### Version 0.18.0
 - Bump `itertools` to 0.7
 - Bump `image` to 0.17
 - [SEMVER_MAJOR] Bump minimum rustc version to 1.20.0
 - Fix parsing of glob paths on Windows ([#90](https://github.com/shssoichiro/oxipng/issues/90))

### Version 0.17.2
 - Bump `image` to 0.16
 - Quickly pass over files that do not have a PNG header ([#85](https://github.com/shssoichiro/oxipng/issues/85) @emielbeinema)
 - Return an error instead of crashing on APNG files ([#83](https://github.com/shssoichiro/oxipng/issues/83) @emielbeinema)

### Version 0.17.1
 - Remove VC++ download requirement for Windows users

### Version 0.17.0
 - [SEMVER_MAJOR] Bump minimum required rustc version to 1.19.0
 - [SEMVER_MINOR] Oxipng will now, by default, attempt to change all transparent pixels to `rgba(0, 0, 0, 0)` to improve compression.
    It does fast trials with filters 0 and 5 to see if this is an improvement over
    the existing alpha channel.
 - [SEMVER_MINOR] Add a `-a` option to the command line (`alphas` in the struct) which enables 6 different
    trials for optimizing the alpha channel, using the previously mentioned fast heuristic.
    This option will make optimization of images with transparency somewhat slower,
    but may improve compression.
 - Fixed a bug in reducing palettes for images with bit depth of two ([#80](https://github.com/shssoichiro/oxipng/issues/80))
 - Fixed another bug in reducing palettes for images with bit depth less than eight ([#82](https://github.com/shssoichiro/oxipng/issues/82))
 - Code cleanup
 - Bump `image` to 0.15

### Version 0.16.3
 - Fix command-line help text ([#70](https://github.com/shssoichiro/oxipng/issues/70))

### Version 0.16.2
 - Publicly export `error` module

### Version 0.16.1
 - Fix rayon's breaking changes that they made in a point release

### Version 0.16.0
 - [SEMVER_MAJOR] Bump minimum rustc version to 1.17.0
 - Bump `image` to 0.14
 - Bump `rayon` to 0.8

### Version 0.15.2
 - Bump `image` to 0.13 ([#65](https://github.com/shssoichiro/oxipng/pull/65))
 - Bump `rayon` to 0.7
 - Bump `itertools` to 0.6

### Version 0.15.1
 - Ignore color reductions that would increase file size ([#61](https://github.com/shssoichiro/oxipng/issues/61))

### Version 0.15.0
 - [SEMVER_MINOR] Check images for correctness before writing result ([#60](https://github.com/shssoichiro/oxipng/issues/60))
 - Fix invalid output when reducing image to a different color type but file size does not improve ([#60](https://github.com/shssoichiro/oxipng/issues/60))
 - Don't write new file if moving from interlaced to non-interlaced if new file would be larger

### Version 0.14.4
 - Fix bug when reducing RGBA to Indexed if image has 256 colors plus a background color

### Version 0.14.3
 - Fix multiple bugs when reducing transparency palettes

### Version 0.14.2
 - Fix a bug when reducing palette in images with bit depth less than 8
 - Fix a bug when reducing palette in images with transparency

### Version 0.14.1
 - Remove zlib dependency and switch entirely to miniz, since zlib 1.2.11 was not working with oxipng. This costs some performance, but is better than having a broken application.

### Version 0.14.0
 - Performance optimizations
 - [SEMVER_MAJOR] Bump minimum rustc version to 1.13.0
 - Add categories on crates.io

### Version 0.13.1
 - Bump regex dependency to 0.2
 - Bump byteorder dependency to 1.0
 - Bump rayon dependency to 0.6

### Version 0.13.0
 - Fix bug in certain PNG headers when reducing color type ([#52](https://github.com/shssoichiro/oxipng/issues/52))
 - [SEMVER_MAJOR] Reduction functions now take `&mut PngData` and return a `bool` indicating whether the image was reduced
 - [SMEVER_MAJOR] Bump minimum required rustc version to 1.12.0

### Version 0.12.0
 - Performance optimizations
 - Fix processing filenames that contain commas (@aliceatlas [#50](https://github.com/shssoichiro/oxipng/pull/50))
 - [SEMVER_MINOR] Add zopfli option (-Z), disabled by default. Gives about 10% better compression, but is currently 50-100x slower.

### Version 0.11.0
 - [SEMVER_MAJOR] Bump minimum rustc version to 1.9.0, required by dependencies
 - [SEMVER_MINOR] Allow calling optimization presets via crate using `Options::from_preset`
 - [SEMVER_MAJOR] Return proper `PngError` type which implements `std::error::Error` from `Result`s
 - [SEMVER_MAJOR] Rename module `deflate::deflate` to `deflate`
 - Performance optimizations

### Version 0.10.0
 - [SEMVER_MINOR] Make clap and regex dependencies optional
   - Enabled by default, needed for executable build; can be disabled for use in crates
 - Remove reduction from palette to grayscale, which was not working and provided minimal benefit

### Version 0.9.0
 - [SEMVER_MAJOR] Significant refactoring of modules
 - Use `itertools` to cleanup areas of code
 - Use multiple threads for filtering trials

### Version 0.8.2
 - Fix issue where images smaller than 4px width would crash on interlacing ([#42](https://github.com/shssoichiro/oxipng/issues/42))

### Version 0.8.1
 - Minor optimizations
 - Fix issue where interlaced images with certain widths would fail to optimize

### Version 0.8.0
 - [SEMVER_MINOR] Add support for optimizing PNGs already loaded into memory via library function

### Version 0.7.0
 - Minor compression improvement on interlaced images
 - Performance optimizations
 - [SEMVER_MINOR] Move default Options into a Default impl
 - [SEMVER_MINOR] Add option for setting number of threads ([#39](https://github.com/shssoichiro/oxipng/issues/39))

### Version 0.6.0
 - Fix issue where output directory would not be created if it did not exist
 - Use miniz for compression strategies where it outperforms zlib
 - [SEMVER_MINOR] Partially implement -p / --preserve, as far as stable Rust will allow for now
 - [SEMVER_MINOR] Implement --fix to ignore CRC errors and recalculate correct CRC in output

### Version 0.5.0
 - [SEMVER_MINOR] Palette entries can now reduced, on by default ([#11](https://github.com/shssoichiro/oxipng/issues/11))
 - Don't report that we are in pretend mode if verbosity is set to none
 - Add cargo bench suite ([#7](https://github.com/shssoichiro/oxipng/issues/7))

### Version 0.4.0
 - Performance optimizations
 - [SEMVER_MAJOR] `-s` automatically infers `--strip safe` ([#31](https://github.com/shssoichiro/oxipng/issues/31))
 - Update byteorder and clap crates
 - Fix issue where interlaced images incorrectly applied filters on the first line of a pass

### Version 0.3.0
 - Properly decode interlaced images
 - [SEMVER_MINOR] Allow converting between progressive and interlaced images ([#3](https://github.com/shssoichiro/oxipng/issues/3))
 - Fix a bug that would cause oxipng to crash on very small images

### Version 0.2.2
 - Limit number of threads to 1.5x number of cores
 - Significantly improve memory usage, especially with high optimization levels. ([#32](https://github.com/shssoichiro/oxipng/issues/32))
 - Refactor output code ([#19](https://github.com/shssoichiro/oxipng/issues/19))

### Version 0.2.1
 - Add rustdoc for public methods and structs
 - Improve filter mode 5 heuristic ([#16](https://github.com/shssoichiro/oxipng/issues/16))
 - Add tests for edge-case images with subtitles ([#29](https://github.com/shssoichiro/oxipng/issues/29))

### Version 0.2.0
 - Fix program version that is displayed when running `oxipng -V`
 - Ensure `--quiet` mode is actually quiet (@SethDusek [#20](https://github.com/shssoichiro/oxipng/pull/20))
 - Write status/debug information to stderr instead of stdout
 - Use heuristics to determine best combination for `-o1` ([#21](https://github.com/shssoichiro/oxipng/issues/21))
 - [SEMVER_MAJOR] Allow 'safe', 'all', or comma-separated list as options for `--strip`
 - [SEMVER_MINOR] Add `-s` alias for `--strip`

### Version 0.1.1
 - Fix `oxipng *` writing all input files to one output file ([#15](https://github.com/shssoichiro/oxipng/issues/15))

### Version 0.1.0
 - Initial beta release
 - Reduce color type and bit depth
 - Recompress with zlib
 - Multithreading
 - Strip headers option
 - Backup file before writing option
 - Write to stdout option
