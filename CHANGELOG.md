**Version 0.2.1**
 - Add rustdoc for public methods and structs

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
