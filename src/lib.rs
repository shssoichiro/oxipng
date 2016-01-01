extern crate byteorder;
extern crate crc;
extern crate libz_sys;
extern crate libc;

use std::path::Path;
use std::collections::HashSet;

pub mod png;
pub mod deflate {
    pub mod deflate;
    pub mod stream;
}

pub struct Options<'a> {
    pub backup: bool,
    pub out_file: &'a Path,
    pub fix_errors: bool,
    pub force: bool,
    pub clobber: bool,
    pub create: bool,
    pub preserve_attrs: bool,
    pub verbosity: Option<u8>,
    pub filter: HashSet<u8>,
    pub interlaced: u8,
    pub compression: HashSet<u8>,
    pub zm: HashSet<u8>,
    pub zs: HashSet<u8>,
    pub zw: u16,
    pub bit_depth_reduction: bool,
    pub color_type_reduction: bool,
    pub palette_reduction: bool,
    pub idat_recoding: bool,
    pub idat_paranoia: bool,
}

pub fn optimize(filepath: &Path, opts: Options) -> Result<(), String> {
    // Decode PNG from file
    println!("Processing: {}", filepath.to_str().unwrap());
    let in_file = Path::new(filepath);
    let mut png = match png::PngData::new(&in_file) {
        Ok(x) => x,
        Err(x) => return Err(x),
    };

    // Print png info
    let idat_current_size = png.idat_data.len();
    let file_current_size = filepath.metadata().unwrap().len();
    if opts.verbosity.is_some() {
        println!("    {}x{} pixels, PNG format",
                 png.ihdr_data.width,
                 png.ihdr_data.height);
        if let Some(palette) = png.palette.clone() {
            println!("    {} bits/pixel, {} colors in palette",
                     png.ihdr_data.bit_depth,
                     palette.len() / 3);
        } else {
            println!("    {}x{} bits/pixel, {:?}",
                     png.bits_per_pixel(),
                     png.ihdr_data.bit_depth,
                     png.ihdr_data.color_type);
        }
        println!("    IDAT size = {} bytes", idat_current_size);
        println!("    File size = {} bytes", file_current_size);
    }
    // TODO: Bit depth/palette reduction
    //
    // TODO: Apply interlacing changes
    // TODO: Force reencoding if interlacing was changed
    //
    // Go through selected permutations and determine the best
    let mut best: Option<(u8, u8, u8, u8)> = None;
    if opts.idat_recoding {
        let combinations = opts.filter.len() * opts.compression.len() * opts.zm.len() *
                           opts.zs.len();
        println!("Trying: {} combinations", combinations);
        // TODO: Multithreading
        for f in &opts.filter {
            for zc in &opts.compression {
                for zm in &opts.zm {
                    for zs in &opts.zs {
                        let new_idat = match deflate::deflate::deflate(png.raw_data.as_ref(),
                                                                       *zc,
                                                                       *zm,
                                                                       *zs,
                                                                       opts.zw) {
                            Ok(x) => x,
                            Err(x) => return Err(x),
                        };
                        // TODO: Apply filtering
                        if new_idat.len() < png.idat_data.len() {
                            best = Some((*f, *zc, *zm, *zs));
                            png.idat_data = new_idat.clone();
                        }
                        if opts.verbosity == Some(1) {
                            println!("    zc = {}  zm = {}  zs = {}  f = {}        {} bytes",
                                     *zc,
                                     *zm,
                                     *zs,
                                     *f,
                                     new_idat.len());
                        }
                    }
                }
            }
        }

        if let Some(better) = best {
            println!("Found better combination:");
            println!("    zc = {}  zm = {}  zs = {}  f = {}        {} bytes",
                     better.1,
                     better.2,
                     better.3,
                     better.0,
                     png.idat_data.len());
        } else {
            println!("IDAT already optimized");
        }
    }

    // TODO: Backup before writing?

    // TODO: Write output file

    Ok(())
}
