extern crate optipng;

use std::env;
use std::path::Path;
use std::collections::HashSet;

fn main() {
    // TODO: Handle wildcards
    let filename = env::args().skip(1).next().unwrap();
    let infile = Path::new(&filename);
    let mut f = HashSet::new();
    f.insert(0);
    f.insert(5);
    let mut zc = HashSet::new();
    zc.insert(9);
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
        f: f,
        i: 0,
        zc: zc,
        zm: zm,
        zs: zs,
        zw: 4096,
        bit_depth_reduction: true,
        color_type_reduction: true,
        palette_reduction: true,
        idat_recoding: true,
        idat_paranoia: false,
    };
    optipng::optimize(infile, default_opts);
}
