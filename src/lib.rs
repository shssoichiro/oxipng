extern crate png;

use std::path::Path;
use std::fs::File;
use std::io;
use std::collections::HashSet;

mod compress;

pub struct Options<'a> {
    pub backup: bool,
    pub out_file: &'a Path,
    pub fix_errors: bool,
    pub force: bool,
    pub clobber: bool,
    pub create: bool,
    pub preserve_attrs: bool,
    pub verbosity: Option<u8>,
    pub f: HashSet<u8>,
    pub i: u8,
    pub zc: HashSet<u8>,
    pub zm: HashSet<u8>,
    pub zs: HashSet<u8>,
    pub zw: u32,
    pub bit_depth_reduction: bool,
    pub color_type_reduction: bool,
    pub palette_reduction: bool,
    pub idat_recoding: bool,
    pub idat_paranoia: bool,
}

// Processing: /Users/holmerj/Downloads/renpy-6.99.7-sdk/doc/_images/frame_example.png
//    579x354 pixels, PNG format
//    3x8 bits/pixel, RGB
//    IDAT size = 45711 bytes
//    file size = 45881 bytes
// Trying: 8 combinations
// Output:
//    IDAT size = 45711 bytes (no change)
//    file size = 45821 bytes (60 bytes = 0.13% decrease)
pub fn optimize(filepath: &Path, opts: Options) -> Result<(), io::Error> {
    // Decode PNG from file
    println!("Processing: {}", filepath.to_str().unwrap());
    let in_file = try!(File::open(filepath));
    let decoder = png::Decoder::new(in_file);
    let (info, mut reader) = try!(decoder.read_info());
    let mut img_buf = vec![0; info.buffer_size()];
    try!(reader.next_frame(&mut img_buf));

    // Read and print
    let info = reader.info();
    let (width, height) = info.size();
    let depth = (info.bytes_per_pixel(), info.bits_per_pixel() / info.bytes_per_pixel());
    // FIXME
    let idat_current_size = img_buf.len();
    let file_current_size = filepath.metadata().unwrap().len();
    if opts.verbosity.is_some() {
        println!("{}x{} pixels, PNG format", width, height);
        if let Some(palette) = info.palette.clone() {
            println!("{} bits/pixel, {} colors in palette", depth.1, palette.len() / 3);
        } else {
            println!("{}x{} bits/pixel, {:?}", depth.0, depth.1, info.color_type);
        }
        println!("IDAT size = {} bytes", idat_current_size);
        println!("File size = {} bytes", file_current_size);
    }

    // TODO: Bit depth/palette reduction

    // Go through selected permutations and determine the best
    if opts.idat_recoding {
        let combinations = opts.f.len() * opts.zc.len() * opts.zm.len() * opts.zs.len();
        println!("Trying: {} combinations", combinations);
        // TODO: Multithreading
        for f in &opts.f {
            for zc in &opts.zc {
                for zm in &opts.zm {
                    for zs in &opts.zs {
                        // TODO: Test compressions
                    }
                }
            }
        }
    }

    // TODO: Backup before writing?

    // TODO: Write output file

    Ok(())
}
