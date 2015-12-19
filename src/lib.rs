extern crate byteorder;
extern crate crc;

use std::path::Path;
use std::collections::HashSet;

mod png;

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
pub fn optimize(filepath: &Path, opts: Options) -> Result<(), String> {
    // Decode PNG from file
    println!("Processing: {}", filepath.to_str().unwrap());
    let in_file = Path::new(filepath);
    let png = match png::PngData::new(&in_file) {
        Ok(x) => x,
        Err(x) => return Err(x)
    };

    // Print png info
    let idat_current_size = png.idat_data.len();
    let file_current_size = filepath.metadata().unwrap().len();
    if opts.verbosity.is_some() {
        println!("    {}x{} pixels, PNG format", png.ihdr_data.width, png.ihdr_data.height);
        if let Some(palette) = png.palette.clone() {
            println!("    {} bits/pixel, {} colors in palette", png.ihdr_data.bit_depth, palette.len() / 3);
        } else {
            println!("    {}x{} bits/pixel, {:?}", png.bits_per_pixel(), png.ihdr_data.bit_depth, png.ihdr_data.color_type);
        }
        println!("    IDAT size = {} bytes", idat_current_size);
        println!("    File size = {} bytes", file_current_size);
    }
    //
    // // TODO: Bit depth/palette reduction
    //
    // // Go through selected permutations and determine the best
    // let mut best: (Option<(u8, u8, u8, u8)>, usize) = (None, idat_current_size.clone());
    // if opts.idat_recoding {
    //     let combinations = opts.f.len() * opts.zc.len() * opts.zm.len() * opts.zs.len();
    //     println!("Trying: {} combinations", combinations);
    //     // TODO: Multithreading
    //     for f in &opts.f {
    //         for zc in &opts.zc {
    //             for zm in &opts.zm {
    //                 for zs in &opts.zs {
    //                     // TODO: Test compressions
    //                     let new_idat = compress::compress(img_buf.clone(), *f, opts.i, *zc, *zm, *zs, opts.zw);
    //                     // TODO: Force reencoding if interlacing was changed
    //                     if new_idat.len() < best.1 {
    //                         best = (Some((*f, *zc, *zm, *zs)), new_idat.len());
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // TODO: Backup before writing?

    // TODO: Write output file

    Ok(())
}
