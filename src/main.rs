extern crate optipng;

use std::env;
use std::path::Path;
use std::collections::HashSet;

fn main() {
    // TODO: Handle wildcards
    let filename = env::args().skip(1).next().unwrap();
    let infile = Path::new(&filename);
    let mut filter = HashSet::new();
    filter.insert(0);
    filter.insert(5);
    let mut compression = HashSet::new();
    compression.insert(9);
    let mut zm = HashSet::new();
    zm.insert(9);
    let mut zs = HashSet::new();
    for i in 0..4 {
        zs.insert(i);
    }

    let default_opts = optipng::Options {
        backup: false,
        out_file: infile,
        fix_errors: false,
        force: false,
        clobber: true,
        create: true,
        preserve_attrs: false,
        verbosity: Some(0),
        filter: filter,
        interlaced: 0,
        compression: compression,
        zm: zm,
        zs: zs,
        zw: 4096,
        bit_depth_reduction: true,
        color_type_reduction: true,
        palette_reduction: true,
        idat_recoding: true,
        idat_paranoia: false,
    };
    // TODO: Handle command line args
    // TODO: Handle optimization presets
    optipng::optimize(infile, default_opts);
}
