extern crate bit_vec;
extern crate byteorder;
extern crate crc;
extern crate libc;
extern crate libz_sys;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

pub mod deflate {
    pub mod deflate;
    pub mod stream;
}
pub mod png;

pub struct Options {
    pub backup: bool,
    pub out_file: PathBuf,
    pub out_dir: Option<PathBuf>,
    pub stdout: bool,
    pub fix_errors: bool,
    pub pretend: bool,
    pub recursive: bool,
    pub clobber: bool,
    pub create: bool,
    pub force: bool,
    pub preserve_attrs: bool,
    pub verbosity: Option<u8>,
    pub filter: HashSet<u8>,
    pub interlace: Option<u8>,
    pub compression: HashSet<u8>,
    pub memory: HashSet<u8>,
    pub strategies: HashSet<u8>,
    pub window: u8,
    pub bit_depth_reduction: bool,
    pub color_type_reduction: bool,
    pub palette_reduction: bool,
    pub idat_recoding: bool,
    pub strip: bool,
}

pub fn optimize(filepath: &Path, opts: &Options) -> Result<(), String> {
    // Decode PNG from file
    println!("Processing: {}", filepath.to_str().unwrap());
    let in_file = Path::new(filepath);
    let mut png = match png::PngData::new(&in_file) {
        Ok(x) => x,
        Err(x) => return Err(x),
    };

    // Print png info
    let idat_original_size = png.idat_data.len();
    let file_original_size = filepath.metadata().unwrap().len() as usize;
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
                     png.bits_per_pixel_raw(),
                     png.ihdr_data.bit_depth,
                     png.ihdr_data.color_type);
        }
        println!("    IDAT size = {} bytes", idat_original_size);
        println!("    File size = {} bytes", file_original_size);
    }

    let mut something_changed = false;

    if opts.color_type_reduction {
        if png.reduce_color_type() {
            something_changed = true;
            // TODO: Print message to terminal
        };
    }

    if opts.bit_depth_reduction {
        if png.reduce_bit_depth() {
            something_changed = true;
            // TODO: Print message to terminal
        };
    }

    if opts.palette_reduction {
        if png.reduce_palette() {
            something_changed = true;
            // TODO: Print message to terminal
        };
    }

    if let Some(interlacing) = opts.interlace {
        if interlacing != png.ihdr_data.interlaced {
            if png.change_interlacing(interlacing) {
                something_changed = true;
                // TODO: Print message to terminal
            }
        }
    }

    if opts.idat_recoding || something_changed {
        // Go through selected permutations and determine the best
        let mut best: Option<(u8, u8, u8, u8)> = None;
        let combinations = opts.filter.len() * opts.compression.len() * opts.memory.len() *
                           opts.strategies.len();
        println!("Trying: {} combinations", combinations);
        // TODO: Multithreading
        for f in &opts.filter {
            let filtered = png.filter_image(*f);
            for zc in &opts.compression {
                for zm in &opts.memory {
                    for zs in &opts.strategies {
                        let new_idat = match deflate::deflate::deflate(&filtered,
                                                                       *zc,
                                                                       *zm,
                                                                       *zs,
                                                                       opts.window) {
                            Ok(x) => x,
                            Err(x) => return Err(x),
                        };
                        if new_idat.len() < png.idat_data.len() ||
                           (best.is_none() && something_changed) {
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
        }
    }

    if opts.strip {
        // Strip headers
        png.aux_headers = HashMap::new();
    }

    let output_data = png.output();
    if file_original_size <= output_data.len() && !opts.force && opts.interlace.is_none() {
        println!("File already optimized");
        return Ok(());
    }

    if opts.pretend {
        println!("Running in pretend mode, no output");
    } else {
        if opts.backup {
            match fs::copy(in_file,
                           in_file.with_extension(format!("bak.{}",
                                                          in_file.extension()
                                                                 .unwrap()
                                                                 .to_str()
                                                                 .unwrap()))) {
                Ok(x) => x,
                Err(_) => {
                    return Err(format!("Unable to write to backup file at {}",
                                       opts.out_file.display()))
                }
            };
        }

        if opts.stdout {
            let mut buffer = BufWriter::new(io::stdout());
            match buffer.write_all(&output_data) {
                Ok(_) => (),
                Err(_) => return Err(format!("Unable to write to stdout")),
            }
        } else {
            let out_file = match File::create(&opts.out_file) {
                Ok(x) => x,
                Err(_) => {
                    return Err(format!("Unable to write to file {}", opts.out_file.display()))
                }
            };
            let mut buffer = BufWriter::new(out_file);
            match buffer.write_all(&output_data) {
                Ok(_) => println!("Output: {}", opts.out_file.display()),
                Err(_) => {
                    return Err(format!("Unable to write to file {}", opts.out_file.display()))
                }
            }
        }
    }
    if idat_original_size >= png.idat_data.len() {
        println!("    IDAT size = {} bytes ({} bytes decrease)",
                 png.idat_data.len(),
                 idat_original_size - png.idat_data.len());
    } else {
        println!("    IDAT size = {} bytes ({} bytes increate)",
                 png.idat_data.len(),
                 png.idat_data.len() - idat_original_size);
    }
    if file_original_size >= output_data.len() {
        println!("    file size = {} bytes ({} bytes = {:.2}% decrease)",
                 output_data.len(),
                 file_original_size - output_data.len(),
                 (file_original_size - output_data.len()) as f64 / file_original_size as f64);
    } else {
        println!("    file size = {} bytes ({} bytes = {:.2}% increase)",
                 output_data.len(),
                 output_data.len() - file_original_size,
                 (output_data.len() - file_original_size) as f64 / file_original_size as f64);
    }

    Ok(())
}
