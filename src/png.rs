use bit_vec::BitVec;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{Cursor, Read};
use std::iter::Iterator;
use std::path::Path;

#[derive(Debug,PartialEq,Clone,Copy)]
/// The color type used to represent this image
pub enum ColorType {
    /// Grayscale, with one color channel
    Grayscale,
    /// RGB, with three color channels
    RGB,
    /// Indexed, with one byte per pixel representing one of up to 256 colors in the image
    Indexed,
    /// Grayscale + Alpha, with two color channels
    GrayscaleAlpha,
    /// RGBA, with four color channels
    RGBA,
}

impl fmt::Display for ColorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   ColorType::Grayscale => "Grayscale",
                   ColorType::RGB => "RGB",
                   ColorType::Indexed => "Indexed",
                   ColorType::GrayscaleAlpha => "Grayscale + Alpha",
                   ColorType::RGBA => "RGB + Alpha",
               })
    }
}

impl ColorType {
    /// Get the code used by the PNG specification to denote this color type
    fn png_header_code(&self) -> u8 {
        match *self {
            ColorType::Grayscale => 0,
            ColorType::RGB => 2,
            ColorType::Indexed => 3,
            ColorType::GrayscaleAlpha => 4,
            ColorType::RGBA => 6,
        }
    }
}

#[derive(Debug,PartialEq,Clone,Copy)]
/// The number of bits to be used per channel per pixel
pub enum BitDepth {
    /// One bit per channel per pixel
    One,
    /// Two bits per channel per pixel
    Two,
    /// Four bits per channel per pixel
    Four,
    /// Eight bits per channel per pixel
    Eight,
    /// Sixteen bits per channel per pixel
    Sixteen,
}

impl fmt::Display for BitDepth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   BitDepth::One => "1",
                   BitDepth::Two => "2",
                   BitDepth::Four => "4",
                   BitDepth::Eight => "8",
                   BitDepth::Sixteen => "16",
               })
    }
}

impl BitDepth {
    /// Retrieve the number of bits per channel per pixel as a `u8`
    pub fn as_u8(&self) -> u8 {
        match *self {
            BitDepth::One => 1,
            BitDepth::Two => 2,
            BitDepth::Four => 4,
            BitDepth::Eight => 8,
            BitDepth::Sixteen => 16,
        }
    }
    /// Parse a number of bits per channel per pixel into a `BitDepth`
    pub fn from_u8(depth: u8) -> BitDepth {
        match depth {
            1 => BitDepth::One,
            2 => BitDepth::Two,
            4 => BitDepth::Four,
            8 => BitDepth::Eight,
            16 => BitDepth::Sixteen,
            _ => panic!("Unsupported bit depth"),
        }
    }
}

#[derive(Debug,PartialEq,Clone)]
/// Options to use for performing operations on headers (such as stripping)
pub enum Headers {
    /// None
    None,
    /// Some, with a list of 4-character chunk codes
    Some(Vec<String>),
    /// Headers that won't affect rendering (all but cHRM, gAMA, iCCP, sBIT, sRGB, bKGD, hIST, pHYs, sPLT)
    Safe,
    /// All non-critical headers
    All,
}

#[derive(Debug,Clone)]
/// An iterator over the scan lines of a PNG image
pub struct ScanLines<'a> {
    /// A reference to the PNG image being iterated upon
    pub png: &'a PngData,
    start: usize,
    end: usize,
    /// Current pass number, and 0-indexed row within the pass
    pass: Option<(u8, u32)>,
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end == self.png.raw_data.len() {
            None
        } else if self.png.ihdr_data.interlaced == 1 {
            // Scanlines for interlaced PNG files
            if self.pass.is_none() {
                self.pass = Some((1, 0));
            }
            // Handle edge cases for images smaller than 5 pixels in either direction
            if self.png.ihdr_data.width < 5 && self.pass.unwrap().0 == 2 {
                if let Some(pass) = self.pass.as_mut() {
                    pass.0 = 3;
                    pass.1 = 4;
                }
            }
            // Intentionally keep these separate so that they can be applied one after another
            if self.png.ihdr_data.height < 5 && self.pass.unwrap().0 == 3 {
                if let Some(pass) = self.pass.as_mut() {
                    pass.0 = 4;
                    pass.1 = 0;
                }
            }
            let bits_per_pixel = self.png.ihdr_data.bit_depth.as_u8() as u32 *
                                 self.png.channels_per_pixel() as u32;
            let y_steps;
            let pixels_factor;
            match self.pass {
                Some((1, _)) | Some((2, _)) => {
                    pixels_factor = 8;
                    y_steps = 8;
                }
                Some((3, _)) => {
                    pixels_factor = 4;
                    y_steps = 8;
                }
                Some((4, _)) => {
                    pixels_factor = 4;
                    y_steps = 4;
                }
                Some((5, _)) => {
                    pixels_factor = 2;
                    y_steps = 4;
                }
                Some((6, _)) => {
                    pixels_factor = 2;
                    y_steps = 2;
                }
                Some((7, _)) => {
                    pixels_factor = 1;
                    y_steps = 2;
                }
                _ => unreachable!(),
            }
            let mut pixels_per_line = self.png.ihdr_data.width / pixels_factor as u32;
            // Determine whether to add pixels if there is a final, incomplete 8x8 block
            let gap = self.png.ihdr_data.width % pixels_factor;
            if gap > 0 {
                match self.pass.unwrap().0 {
                    1 | 3 | 5 => {
                        pixels_per_line += 1;
                    }
                    2 => {
                        if gap >= 5 {
                            pixels_per_line += 1;
                        }
                    }
                    4 => {
                        if gap >= 3 {
                            pixels_per_line += 1;
                        }
                    }
                    6 => {
                        if gap >= 2 {
                            pixels_per_line += 1;
                        }
                    }
                    _ => (),
                };
            }
            let current_pass = if let Some(pass) = self.pass {
                Some(pass.0)
            } else {
                None
            };
            let bytes_per_line = ((pixels_per_line * bits_per_pixel) as f32 / 8f32).ceil() as usize;
            self.start = self.end;
            self.end = self.start + bytes_per_line + 1;
            if let Some(pass) = self.pass.as_mut() {
                if pass.1 + y_steps >= self.png.ihdr_data.height {
                    pass.0 += 1;
                    pass.1 = match pass.0 {
                        3 => 4,
                        5 => 2,
                        7 => 1,
                        _ => 0,
                    };
                } else {
                    pass.1 += y_steps;
                }
            }
            Some(ScanLine {
                filter: self.png.raw_data[self.start],
                data: self.png.raw_data[(self.start + 1)..self.end].to_owned(),
                pass: current_pass,
            })
        } else {
            // Standard, non-interlaced PNG scanlines
            let bits_per_line = self.png.ihdr_data.width as usize *
                                self.png.ihdr_data.bit_depth.as_u8() as usize *
                                self.png.channels_per_pixel() as usize;
            let bytes_per_line = (bits_per_line as f32 / 8f32).ceil() as usize;
            self.start = self.end;
            self.end = self.start + bytes_per_line + 1;
            Some(ScanLine {
                filter: self.png.raw_data[self.start],
                data: self.png.raw_data[(self.start + 1)..self.end].to_owned(),
                pass: None,
            })
        }
    }
}

#[derive(Debug,Clone)]
/// A scan line in a PNG image
pub struct ScanLine {
    /// The filter type used to encode the current scan line (0-4)
    pub filter: u8,
    /// The byte data for the current scan line, encoded with the filter specified in the `filter` field
    pub data: Vec<u8>,
    /// The current pass if the image is interlaced
    pub pass: Option<u8>,
}

#[derive(Debug,Clone)]
/// Contains all data relevant to a PNG image
pub struct PngData {
    /// The filtered and compressed data of the IDAT chunk
    pub idat_data: Vec<u8>,
    /// The headers stored in the IHDR chunk
    pub ihdr_data: IhdrData,
    /// The uncompressed, optionally filtered data from the IDAT chunk
    pub raw_data: Vec<u8>,
    /// The palette containing colors used in an Indexed image
    /// Contains 3 bytes per color (R+G+B), up to 768
    pub palette: Option<Vec<u8>>,
    /// The pixel value that should be rendered as transparent
    pub transparency_pixel: Option<Vec<u8>>,
    /// A map of how transparent each color in the palette should be
    pub transparency_palette: Option<Vec<u8>>,
    /// All non-critical headers from the PNG are stored here
    pub aux_headers: HashMap<String, Vec<u8>>,
}

#[derive(Debug,Clone,Copy)]
/// Headers from the IHDR chunk of the image
pub struct IhdrData {
    /// The width of the image in pixels
    pub width: u32,
    /// The height of the image in pixels
    pub height: u32,
    /// The color type of the image
    pub color_type: ColorType,
    /// The bit depth of the image
    pub bit_depth: BitDepth,
    /// The compression method used for this image (0 for DEFLATE)
    pub compression: u8,
    /// The filter mode used for this image (currently only 0 is valid)
    pub filter: u8,
    /// The interlacing mode of the image (0 = None, 1 = Adam7)
    pub interlaced: u8,
}

impl PngData {
    /// Create a new `PngData` struct by opening a file
    pub fn new(filepath: &Path, fix_errors: bool) -> Result<PngData, String> {
        let mut file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return Err("Failed to open file for reading".to_owned()),
        };
        let mut byte_data: Vec<u8> = Vec::new();
        // Read raw png data into memory
        match file.read_to_end(&mut byte_data) {
            Ok(_) => (),
            Err(_) => return Err("Failed to read from file".to_owned()),
        }

        PngData::from_slice(&byte_data, fix_errors)
    }

    /// Create a new `PngData` struct by reading a slice
    pub fn from_slice(byte_data: &[u8], fix_errors: bool) -> Result<PngData, String> {
        let mut byte_offset: usize = 0;
        // Test that png header is valid
        let header: Vec<u8> = byte_data.iter().take(8).cloned().collect();
        if !file_header_is_valid(header.as_ref()) {
            return Err("Invalid PNG header detected".to_owned());
        }
        byte_offset += 8;
        // Read the data headers
        let mut aux_headers: HashMap<String, Vec<u8>> = HashMap::new();
        let mut idat_headers: Vec<u8> = Vec::new();
        loop {
            let header = parse_next_header(byte_data.as_ref(), &mut byte_offset, fix_errors);
            let header = match header {
                Ok(x) => x,
                Err(x) => return Err(x),
            };
            let header = match header {
                Some(x) => x,
                None => break,
            };
            if header.0 == "IDAT" {
                idat_headers.extend(header.1);
            } else {
                aux_headers.insert(header.0, header.1);
            }
        }
        // Parse the headers into our PngData
        if idat_headers.is_empty() {
            return Err("Image data was empty, skipping".to_owned());
        }
        if aux_headers.get("IHDR").is_none() {
            return Err("Image header data was missing, skipping".to_owned());
        }
        let ihdr_header = match parse_ihdr_header(aux_headers.remove("IHDR").unwrap().as_ref()) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        let raw_data = match super::deflate::deflate::inflate(idat_headers.as_ref()) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        // Handle transparency header
        let mut has_transparency_pixel = false;
        let mut has_transparency_palette = false;
        if aux_headers.contains_key("tRNS") {
            if ihdr_header.color_type == ColorType::Indexed {
                has_transparency_palette = true;
            } else {
                has_transparency_pixel = true;
            }
        }
        let mut png_data = PngData {
            idat_data: idat_headers.clone(),
            ihdr_data: ihdr_header,
            raw_data: raw_data,
            palette: aux_headers.remove("PLTE"),
            transparency_pixel: if has_transparency_pixel {
                aux_headers.remove("tRNS")
            } else {
                None
            },
            transparency_palette: if has_transparency_palette {
                aux_headers.remove("tRNS")
            } else {
                None
            },
            aux_headers: aux_headers,
        };
        png_data.raw_data = png_data.unfilter_image();
        // Return the PngData
        Ok(png_data)
    }
    /// Return the number of channels in the image, based on color type
    pub fn channels_per_pixel(&self) -> u8 {
        match self.ihdr_data.color_type {
            ColorType::Grayscale | ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGB => 3,
            ColorType::RGBA => 4,
        }
    }
    /// Format the `PngData` struct into a valid PNG bytestream
    pub fn output(&self) -> Vec<u8> {
        // PNG header
        let mut output = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR
        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.width).ok();
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.height).ok();
        ihdr_data.write_u8(self.ihdr_data.bit_depth.as_u8()).ok();
        ihdr_data.write_u8(self.ihdr_data.color_type.png_header_code()).ok();
        ihdr_data.write_u8(0).ok(); // Compression -- deflate
        ihdr_data.write_u8(0).ok(); // Filter method -- 5-way adaptive filtering
        ihdr_data.write_u8(self.ihdr_data.interlaced).ok();
        write_png_block(b"IHDR", &ihdr_data, &mut output);
        // Ancillary headers
        for (key, header) in self.aux_headers.iter().filter(|&(ref key, _)| {
            !(**key == "bKGD" || **key == "hIST" || **key == "tRNS")
        }) {
            write_png_block(key.as_bytes(), header, &mut output);
        }
        // Palette
        if let Some(palette) = self.palette.clone() {
            write_png_block(b"PLTE", &palette, &mut output);
            if let Some(transparency_palette) = self.transparency_palette.clone() {
                // Transparency pixel
                write_png_block(b"tRNS", &transparency_palette, &mut output);
            }
        } else if let Some(transparency_pixel) = self.transparency_pixel.clone() {
            // Transparency pixel
            write_png_block(b"tRNS", &transparency_pixel, &mut output);
        }
        // Special ancillary headers that need to come after PLTE but before IDAT
        for (key, header) in self.aux_headers.iter().filter(|&(ref key, _)| {
            **key == "bKGD" || **key == "hIST" || **key == "tRNS"
        }) {
            write_png_block(key.as_bytes(), header, &mut output);
        }
        // IDAT data
        write_png_block(b"IDAT", &self.idat_data, &mut output);
        // Stream end
        write_png_block(b"IEND", &[], &mut output);

        output
    }
    /// Return an iterator over the scanlines of the image
    pub fn scan_lines(&self) -> ScanLines {
        ScanLines {
            png: self,
            start: 0,
            end: 0,
            pass: None,
        }
    }
    /// Reverse all filters applied on the image, returning an unfiltered IDAT bytestream
    pub fn unfilter_image(&self) -> Vec<u8> {
        let mut unfiltered = Vec::with_capacity(self.raw_data.len());
        let bpp = (((self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel()) as f32) /
                   8f32)
                      .ceil() as usize;
        let mut last_line: Vec<u8> = Vec::new();
        for line in self.scan_lines() {
            let unfiltered_line = unfilter_line(line.filter, bpp, &line.data, &last_line);
            unfiltered.push(0);
            unfiltered.extend_from_slice(&unfiltered_line);
            last_line = unfiltered_line;
        }
        unfiltered
    }
    /// Apply the specified filter type to all rows in the image
    /// 0: None
    /// 1: Sub
    /// 2: Up
    /// 3: Average
    /// 4: Paeth
    /// 5: All (heuristically pick the best filter for each line)
    pub fn filter_image(&self, filter: u8) -> Vec<u8> {
        let mut filtered = Vec::with_capacity(self.raw_data.len());
        let bpp = (((self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel()) as f32) /
                   8f32)
                      .ceil() as usize;
        let mut last_line: Vec<u8> = Vec::new();
        let mut last_pass: Option<u8> = None;
        for line in self.scan_lines() {
            match filter {
                0 | 1 | 2 | 3 | 4 => {
                    if last_pass == line.pass || filter <= 1 {
                        filtered.push(filter);
                        filtered.extend_from_slice(&filter_line(filter,
                                                                bpp,
                                                                &line.data,
                                                                &last_line));
                    } else {
                        // Avoid vertical filtering on first line of each interlacing pass
                        filtered.push(0);
                        filtered.extend_from_slice(&filter_line(0, bpp, &line.data, &last_line));
                    }
                }
                5 => {
                    // Heuristically guess best filter per line
                    // Uses MSAD algorithm mentioned in libpng reference docs
                    // http://www.libpng.org/pub/png/book/chapter09.html
                    let mut trials: HashMap<u8, Vec<u8>> = HashMap::with_capacity(5);
                    // Avoid vertical filtering on first line of each interlacing pass
                    for filter in if last_pass == line.pass {
                        0..5
                    } else {
                        0..2
                    } {
                        trials.insert(filter, filter_line(filter, bpp, &line.data, &last_line));
                    }
                    let (best_filter, best_line) = trials.iter()
                                                         .min_by_key(|x| {
                                                             x.1.iter().fold(0u64, |acc, &x| {
                                                                 let signed = x as i8;
                                                                 acc + (signed as i16).abs() as u64
                                                             })
                                                         })
                                                         .unwrap();
                    filtered.push(*best_filter);
                    filtered.extend_from_slice(best_line);
                }
                _ => unreachable!(),
            }
            last_line = line.data.clone();
            last_pass = line.pass;
        }
        filtered
    }
    /// Attempt to reduce the bit depth of the image
    /// Returns true if the bit depth was reduced, false otherwise
    pub fn reduce_bit_depth(&mut self) -> bool {
        if self.ihdr_data.bit_depth != BitDepth::Sixteen {
            if self.ihdr_data.color_type == ColorType::Indexed ||
               self.ihdr_data.color_type == ColorType::Grayscale {
                return match reduce_bit_depth_8_or_less(self) {
                    Some((data, depth)) => {
                        self.raw_data = data;
                        self.ihdr_data.bit_depth = BitDepth::from_u8(depth);
                        true
                    }
                    None => false,
                };
            }
            return false;
        }

        // Reduce from 16 to 8 bits per channel per pixel
        let mut reduced =
            Vec::with_capacity((self.ihdr_data.width * self.ihdr_data.height *
                                self.channels_per_pixel() as u32 +
                                self.ihdr_data.height) as usize);
        let mut high_byte = 0;

        for line in self.scan_lines() {
            reduced.push(line.filter);
            for (i, byte) in line.data.iter().enumerate() {
                if i % 2 == 0 {
                    // High byte
                    high_byte = *byte;
                } else {
                    // Low byte
                    if high_byte != *byte {
                        // Can't reduce, exit early
                        return false;
                    }
                    reduced.push(*byte);
                }
            }
        }

        self.ihdr_data.bit_depth = BitDepth::Eight;
        self.raw_data = reduced;
        true
    }
    /// Attempt to reduce the number of colors in the palette
    /// Returns true if the palette was reduced, false otherwise
    pub fn reduce_palette(&mut self) -> bool {
        if self.ihdr_data.color_type != ColorType::Indexed {
            // Can't reduce if there is no palette
            return false;
        }
        if self.ihdr_data.bit_depth == BitDepth::One {
            // Gains from 1-bit images will be at most 1 byte
            // Not worth the CPU time
            return false;
        }

        // A palette with RGB slices
        let palette = self.palette.clone().unwrap();
        let mut indexed_palette: Vec<&[u8]> = palette.chunks(3).collect();
        // A map of old indexes to new ones, for any moved
        let mut index_map: HashMap<u8, u8> = HashMap::new();

        // A list of (original) indices that are duplicates and no longer needed
        let mut duplicates: Vec<u8> = Vec::new();
        {
            // Find duplicate entries in the palette
            let mut seen: HashMap<&[u8], u8> = HashMap::with_capacity(indexed_palette.len());
            for (i, color) in indexed_palette.iter().enumerate() {
                if seen.contains_key(color) {
                    let index = seen.get(color).unwrap();
                    duplicates.push(i as u8);
                    index_map.insert(i as u8, *index);
                } else {
                    seen.insert(*color, i as u8);
                }
            }
        }

        // Remove duplicates from the data
        if !duplicates.is_empty() {
            self.do_palette_reduction(&mut duplicates, &mut index_map, &mut indexed_palette);
        }

        // A list of unused palette indices
        let mut unused: Vec<u8> = Vec::new();
        {
            // Find palette entries that are never used
            let mut seen = HashSet::with_capacity(indexed_palette.len());
            for line in self.scan_lines() {
                match self.ihdr_data.bit_depth {
                    BitDepth::Eight => {
                        for byte in &line.data {
                            seen.insert(*byte);
                        }
                    }
                    BitDepth::Four => {
                        let bitvec = BitVec::from_bytes(&line.data);
                        let mut current = 0u8;
                        for (i, bit) in bitvec.iter().enumerate() {
                            let mod_i = i % 4;
                            if bit {
                                current += 2u8.pow(3u32 - mod_i as u32);
                            }
                            if mod_i == 3 {
                                seen.insert(current);
                                current = 0;
                            }
                        }
                    }
                    BitDepth::Two => {
                        let bitvec = BitVec::from_bytes(&line.data);
                        let mut current = 0u8;
                        for (i, bit) in bitvec.iter().enumerate() {
                            let mod_i = i % 2;
                            if bit {
                                current += 2u8.pow(1u32 - mod_i as u32);
                            }
                            if mod_i == 1 {
                                seen.insert(current);
                                current = 0;
                            }
                        }
                    }
                    _ => unreachable!(),
                }

                if seen.len() == indexed_palette.len() {
                    // Exit early if no further possible optimizations
                    // Check at the end of each line
                    // Checking after every pixel would be overly expensive
                    return !duplicates.is_empty();
                }
            }
            for i in 0..indexed_palette.len() as u8 {
                if !seen.contains(&i) {
                    unused.push(i);
                }
            }
        }

        // Remove unused palette indices
        self.do_palette_reduction(&mut unused, &mut index_map, &mut indexed_palette);

        true
    }
    fn do_palette_reduction(&mut self,
                            indices: &mut Vec<u8>,
                            index_map: &mut HashMap<u8, u8>,
                            indexed_palette: &mut Vec<&[u8]>) {
        let mut new_data = Vec::with_capacity(self.raw_data.len());
        let mut alpha_palette = self.aux_headers.get("tRNS").cloned();
        let original_len = indexed_palette.len();
        indices.sort_by(|a, b| b.cmp(a));
        for idx in indices {
            for i in (*idx as usize + 1)..original_len {
                let existing = index_map.entry(i as u8).or_insert(i as u8);
                if *existing >= *idx {
                    *existing -= 1;
                }
            }
            indexed_palette.remove(*idx as usize);
            if let Some(ref mut alpha) = alpha_palette {
                alpha.remove(*idx as usize);
            }
        }
        if alpha_palette.is_some() {
            let alpha_header = self.aux_headers.get_mut("tRNS");
            if let Some(alpha_hdr) = alpha_header {
                *alpha_hdr = alpha_palette.unwrap();
            }
        }
        // Reassign data bytes to new indices
        for line in self.scan_lines() {
            new_data.push(line.filter);
            match self.ihdr_data.bit_depth {
                BitDepth::Eight => {
                    for byte in &line.data {
                        if let Some(new_idx) = index_map.get(byte) {
                            new_data.push(*new_idx);
                        } else {
                            new_data.push(*byte);
                        }
                    }
                }
                BitDepth::Four => {
                    for byte in &line.data {
                        let upper = *byte >> 4;
                        let lower = *byte & 0b00001111;
                        let mut new_byte = 0u8;
                        if let Some(new_idx) = index_map.get(&upper) {
                            new_byte &= *new_idx << 4;
                        } else {
                            new_byte &= upper << 4;
                        }
                        if let Some(new_idx) = index_map.get(&lower) {
                            new_byte &= *new_idx;
                        } else {
                            new_byte &= lower;
                        }
                        new_data.push(new_byte);
                    }
                }
                BitDepth::Two => {
                    for byte in &line.data {
                        let one = *byte >> 6;
                        let two = (*byte >> 4) & 0b00000011;
                        let three = (*byte >> 2) & 0b00000011;
                        let four = *byte & 0b00000011;
                        let mut new_byte = 0u8;
                        if let Some(new_idx) = index_map.get(&one) {
                            new_byte &= *new_idx << 6;
                        } else {
                            new_byte &= one << 6;
                        }
                        if let Some(new_idx) = index_map.get(&two) {
                            new_byte &= *new_idx << 4;
                        } else {
                            new_byte &= two << 4;
                        }
                        if let Some(new_idx) = index_map.get(&three) {
                            new_byte &= *new_idx << 2;
                        } else {
                            new_byte &= three << 2;
                        }
                        if let Some(new_idx) = index_map.get(&four) {
                            new_byte &= *new_idx;
                        } else {
                            new_byte &= four;
                        }
                        new_data.push(new_byte);
                    }
                }
                _ => unreachable!(),
            }
        }
        index_map.clear();
        self.raw_data = new_data;
        let mut new_palette = Vec::with_capacity(indexed_palette.len() * 3);
        for color in indexed_palette {
            new_palette.extend_from_slice(color);
        }
        self.palette = Some(new_palette);
    }
    /// Attempt to reduce the color type of the image
    /// Returns true if the color type was reduced, false otherwise
    pub fn reduce_color_type(&mut self) -> bool {
        let mut changed = false;
        let mut should_reduce_bit_depth = false;

        // Go down one step at a time
        // Maybe not the most efficient, but it's safe
        if self.ihdr_data.color_type == ColorType::RGBA {
            if let Some(data) = reduce_rgba_to_grayscale_alpha(self) {
                self.raw_data = data;
                self.ihdr_data.color_type = ColorType::GrayscaleAlpha;
                changed = true;
            } else if let Some(data) = reduce_rgba_to_rgb(self) {
                self.raw_data = data;
                self.ihdr_data.color_type = ColorType::RGB;
                changed = true;
            } else if let Some((data, palette, trans)) = reduce_rgba_to_palette(self) {
                self.raw_data = data;
                self.palette = Some(palette);
                if trans.iter().any(|x| *x != 255) {
                    self.transparency_palette = Some(trans);
                } else {
                    self.transparency_palette = None;
                }
                self.ihdr_data.color_type = ColorType::Indexed;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::GrayscaleAlpha {
            if let Some(data) = reduce_grayscale_alpha_to_grayscale(self) {
                self.raw_data = data;
                self.ihdr_data.color_type = ColorType::Grayscale;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::RGB {
            if let Some(data) = reduce_rgb_to_grayscale(self) {
                self.raw_data = data;
                self.ihdr_data.color_type = ColorType::Grayscale;
                changed = true;
                should_reduce_bit_depth = true;
            } else if let Some((data, palette)) = reduce_rgb_to_palette(self) {
                self.raw_data = data;
                self.palette = Some(palette);
                self.ihdr_data.color_type = ColorType::Indexed;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::Indexed && self.transparency_palette.is_none() &&
           self.palette.as_ref().map(|x| x.len()).unwrap() > 16 {
            if let Some(data) = reduce_palette_to_grayscale(self) {
                self.raw_data = data;
                self.palette = None;
                self.ihdr_data.color_type = ColorType::Grayscale;
                changed = true;
                should_reduce_bit_depth = false;
            }
        } else if self.ihdr_data.color_type == ColorType::Grayscale {
            if let Some((data, palette)) = reduce_grayscale_to_palette(self) {
                self.raw_data = data;
                self.palette = Some(palette);
                self.ihdr_data.color_type = ColorType::Indexed;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if should_reduce_bit_depth {
            // Some conversions will allow us to perform bit depth reduction that
            // wasn't possible before
            if let Some((data, depth)) = reduce_bit_depth_8_or_less(self) {
                self.raw_data = data;
                self.ihdr_data.bit_depth = BitDepth::from_u8(depth);
            }
        }

        changed
    }
    /// Convert the image to the specified interlacing type
    /// Returns true if the interlacing was changed, false otherwise
    /// The `interlace` parameter specifies the *new* interlacing mode
    /// Assumes that the data has already been de-filtered
    pub fn change_interlacing(&mut self, interlace: u8) -> bool {
        if interlace == self.ihdr_data.interlaced {
            return false;
        }

        if interlace == 1 {
            // Convert progressive to interlaced data
            interlace_image(self);
        } else {
            // Convert interlaced to progressive data
            deinterlace_image(self);
        }
        true
    }
}

fn interlace_image(png: &mut PngData) {
    let mut passes: Vec<BitVec> = Vec::with_capacity(7);
    for _ in 0..7 {
        passes.push(BitVec::new());
    }
    let bits_per_pixel = png.ihdr_data.bit_depth.as_u8() * png.channels_per_pixel();
    for (index, line) in png.scan_lines().enumerate() {
        match index % 8 {
            // Add filter bytes to appropriate lines
            0 => {
                passes[0].extend(BitVec::from_elem(8, false));
                passes[3].extend(BitVec::from_elem(8, false));
                passes[5].extend(BitVec::from_elem(8, false));
                if png.ihdr_data.width > 4 {
                    passes[1].extend(BitVec::from_elem(8, false));
                }
            }
            4 => {
                passes[3].extend(BitVec::from_elem(8, false));
                passes[5].extend(BitVec::from_elem(8, false));
                passes[2].extend(BitVec::from_elem(8, false));
            }
            2 | 6 => {
                passes[4].extend(BitVec::from_elem(8, false));
                passes[5].extend(BitVec::from_elem(8, false));
            }
            _ => {
                passes[6].extend(BitVec::from_elem(8, false));
            }
        }
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            // Avoid moving padded 0's into new image
            if i >= (png.ihdr_data.width * bits_per_pixel as u32) as usize {
                break;
            }
            // Copy pixels into interlaced passes
            let pix_modulo = (i / bits_per_pixel as usize) % 8;
            match index % 8 {
                0 => {
                    match pix_modulo {
                        0 => passes[0].push(bit),
                        4 => passes[1].push(bit),
                        2 | 6 => passes[3].push(bit),
                        _ => passes[5].push(bit),
                    }
                }
                4 => {
                    match pix_modulo {
                        0 | 4 => passes[2].push(bit),
                        2 | 6 => passes[3].push(bit),
                        _ => passes[5].push(bit),
                    }
                }
                2 | 6 => {
                    match pix_modulo % 2 {
                        0 => passes[4].push(bit),
                        _ => passes[5].push(bit),
                    }
                }
                _ => {
                    passes[6].push(bit);
                }
            }
        }
        // Pad end of line on each pass to get 8 bits per byte
        for pass in &mut passes {
            while pass.len() % 8 != 0 {
                pass.push(false);
            }
        }
    }
    let mut output = Vec::new();
    for pass in &passes {
        output.extend(pass.to_bytes());
    }
    png.raw_data = output;
}

fn deinterlace_image(png: &mut PngData) {
    let bits_per_pixel = png.ihdr_data.bit_depth.as_u8() * png.channels_per_pixel();
    let mut lines: Vec<BitVec> = Vec::with_capacity(png.ihdr_data.height as usize);
    for _ in 0..png.ihdr_data.height {
        // Initialize each output line with a starting filter byte of 0
        // as well as some blank data
        lines.push(BitVec::from_elem(8 + bits_per_pixel as usize * png.ihdr_data.width as usize,
                                     false));
    }
    let mut current_pass = 1;
    let mut pass_constants = interlaced_constants(current_pass);
    let mut current_y: usize = pass_constants.y_shift as usize;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        let bits_in_line = ((png.ihdr_data.width - pass_constants.x_shift as u32) as f32 /
                            pass_constants.x_step as f32)
                               .ceil() as usize *
                           bits_per_pixel as usize;
        for (i, bit) in bit_vec.iter().enumerate() {
            // Avoid moving padded 0's into new image
            if i >= bits_in_line {
                break;
            }
            let current_x: usize = pass_constants.x_shift as usize +
                                   (i / bits_per_pixel as usize) * pass_constants.x_step as usize;
            // Copy this bit into the output line, offset by 8 because of filter byte
            let index = 8 + (i % bits_per_pixel as usize) + current_x * bits_per_pixel as usize;
            lines[current_y].set(index, bit);
        }
        // Calculate the next line and move to next pass if necessary
        current_y += pass_constants.y_step as usize;
        if current_y >= png.ihdr_data.height as usize {
            if current_pass == 7 {
                break;
            }
            current_pass += 1;
            if current_pass == 2 && png.ihdr_data.width <= 4 {
                current_pass += 1;
            }
            if current_pass == 3 && png.ihdr_data.height <= 4 {
                current_pass += 1;
            }
            pass_constants = interlaced_constants(current_pass);
            current_y = pass_constants.y_shift as usize;
        }
    }
    let mut output = Vec::new();
    for line in &mut lines {
        while line.len() % 8 != 0 {
            line.push(false);
        }
        output.extend(line.to_bytes());
    }
    png.raw_data = output;
}

struct InterlacedConstants {
    x_shift: u8,
    y_shift: u8,
    x_step: u8,
    y_step: u8,
}

fn interlaced_constants(pass: u8) -> InterlacedConstants {
    match pass {
        1 => {
            InterlacedConstants {
                x_shift: 0,
                y_shift: 0,
                x_step: 8,
                y_step: 8,
            }
        }
        2 => {
            InterlacedConstants {
                x_shift: 4,
                y_shift: 0,
                x_step: 8,
                y_step: 8,
            }
        }
        3 => {
            InterlacedConstants {
                x_shift: 0,
                y_shift: 4,
                x_step: 4,
                y_step: 8,
            }
        }
        4 => {
            InterlacedConstants {
                x_shift: 2,
                y_shift: 0,
                x_step: 4,
                y_step: 4,
            }
        }
        5 => {
            InterlacedConstants {
                x_shift: 0,
                y_shift: 2,
                x_step: 2,
                y_step: 4,
            }
        }
        6 => {
            InterlacedConstants {
                x_shift: 1,
                y_shift: 0,
                x_step: 2,
                y_step: 2,
            }
        }
        7 => {
            InterlacedConstants {
                x_shift: 0,
                y_shift: 1,
                x_step: 1,
                y_step: 2,
            }
        }
        _ => unreachable!(),
    }
}

fn filter_line(filter: u8, bpp: usize, data: &[u8], last_line: &[u8]) -> Vec<u8> {
    let mut filtered = Vec::with_capacity(data.len());
    match filter {
        0 => {
            filtered.extend_from_slice(data);
        }
        1 => {
            filtered.extend_from_slice(&data[0..bpp]);
            filtered.extend_from_slice(&data.iter()
                                            .skip(bpp)
                                            .zip(data.iter())
                                            .map(|(cur, last)| cur.wrapping_sub(*last))
                                            .collect::<Vec<u8>>());
        }
        2 => {
            if last_line.is_empty() {
                filtered.extend_from_slice(data);
            } else {
                filtered.extend_from_slice(&data.iter()
                                                .zip(last_line.iter())
                                                .map(|(cur, last)| cur.wrapping_sub(*last))
                                                .collect::<Vec<u8>>());
            };
        }
        3 => {
            for (i, byte) in data.iter().enumerate() {
                if last_line.is_empty() {
                    filtered.push(match i.checked_sub(bpp) {
                        Some(x) => byte.wrapping_sub(data[x] >> 1),
                        None => *byte,
                    });
                } else {
                    filtered.push(match i.checked_sub(bpp) {
                        Some(x) => {
                            byte.wrapping_sub(((data[x] as u16 + last_line[i] as u16) >> 1) as u8)
                        }
                        None => byte.wrapping_sub(last_line[i] >> 1),
                    });
                };
            }
        }
        4 => {
            for (i, byte) in data.iter().enumerate() {
                if last_line.is_empty() {
                    filtered.push(match i.checked_sub(bpp) {
                        Some(x) => byte.wrapping_sub(data[x]),
                        None => *byte,
                    });
                } else {
                    filtered.push(match i.checked_sub(bpp) {
                        Some(x) => {
                            byte.wrapping_sub(paeth_predictor(data[x], last_line[i], last_line[x]))
                        }
                        None => byte.wrapping_sub(last_line[i]),
                    });
                };
            }
        }
        _ => unreachable!(),
    }
    filtered
}

fn unfilter_line(filter: u8, bpp: usize, data: &[u8], last_line: &[u8]) -> Vec<u8> {
    let mut unfiltered = Vec::with_capacity(data.len());
    match filter {
        0 => {
            unfiltered.extend_from_slice(data);
        }
        1 => {
            for (i, byte) in data.iter().enumerate() {
                match i.checked_sub(bpp) {
                    Some(x) => {
                        let b = unfiltered[x];
                        unfiltered.push(byte.wrapping_add(b));
                    }
                    None => {
                        unfiltered.push(*byte);
                    }
                };
            }
        }
        2 => {
            if last_line.is_empty() {
                unfiltered.extend_from_slice(data);
            } else {
                unfiltered.extend_from_slice(&data.iter()
                                                  .zip(last_line.iter())
                                                  .map(|(cur, last)| cur.wrapping_add(*last))
                                                  .collect::<Vec<u8>>());
            };
        }
        3 => {
            for (i, byte) in data.iter().enumerate() {
                if last_line.is_empty() {
                    match i.checked_sub(bpp) {
                        Some(x) => {
                            let b = unfiltered[x];
                            unfiltered.push(byte.wrapping_add(b >> 1));
                        }
                        None => {
                            unfiltered.push(*byte);
                        }
                    };
                } else {
                    match i.checked_sub(bpp) {
                        Some(x) => {
                            let b = unfiltered[x];
                            unfiltered.push(byte.wrapping_add(((b as u16 + last_line[i] as u16) >> 1) as u8));
                        }
                        None => {
                            unfiltered.push(byte.wrapping_add(last_line[i] >> 1));
                        }
                    };
                };
            }
        }
        4 => {
            for (i, byte) in data.iter().enumerate() {
                if last_line.is_empty() {
                    match i.checked_sub(bpp) {
                        Some(x) => {
                            let b = unfiltered[x];
                            unfiltered.push(byte.wrapping_add(b));
                        }
                        None => {
                            unfiltered.push(*byte);
                        }
                    };
                } else {
                    match i.checked_sub(bpp) {
                        Some(x) => {
                            let b = unfiltered[x];
                            unfiltered.push(byte.wrapping_add(paeth_predictor(b,
                                                                              last_line[i],
                                                                              last_line[x])));
                        }
                        None => {
                            unfiltered.push(byte.wrapping_add(last_line[i]));
                        }
                    };
                };
            }
        }
        _ => unreachable!(),
    }
    unfiltered
}

fn reduce_bit_depth_8_or_less(png: &PngData) -> Option<(Vec<u8>, u8)> {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let mut allowed_bits = 1;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            let bit_index = bit_depth - (i % bit_depth);
            if bit && bit_index > allowed_bits {
                allowed_bits = bit_index.next_power_of_two();
                if allowed_bits == bit_depth {
                    // Not reducable
                    return None;
                }
            }
        }
    }

    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            let bit_index = bit_depth - (i % bit_depth);
            if bit_index <= allowed_bits {
                reduced.push(bit);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
        }
    }

    Some((reduced.to_bytes(), allowed_bits as u8))
}

fn reduce_rgba_to_rgb(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 4 * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp >= colored_bytes {
                if *byte != 255 {
                    return None;
                }
            } else {
                reduced.push(*byte);
            }
        }
    }

    Some(reduced)
}

fn reduce_rgba_to_grayscale_alpha(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 4 * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut low_bytes = Vec::with_capacity(4);
        let mut high_bytes = Vec::with_capacity(4);
        let mut trans_bytes = Vec::with_capacity(byte_depth as usize);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp < colored_bytes {
                if byte_depth == 1 || i % 2 == 1 {
                    low_bytes.push(*byte);
                } else {
                    high_bytes.push(*byte);
                }
            } else {
                trans_bytes.push(*byte);
            }

            if i % bpp == bpp - 1 {
                low_bytes.sort();
                low_bytes.dedup();
                if low_bytes.len() > 1 {
                    return None;
                }
                if byte_depth == 2 {
                    high_bytes.sort();
                    high_bytes.dedup();
                    if high_bytes.len() > 1 {
                        return None;
                    }
                    reduced.push(high_bytes[0]);
                    high_bytes.clear();
                }
                reduced.push(low_bytes[0]);
                low_bytes.clear();
                reduced.extend_from_slice(&trans_bytes);
                trans_bytes.clear();
            }
        }
    }

    Some(reduced)
}

fn reduce_rgba_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = Vec::with_capacity(256);
    let bpp: usize = (4 * png.ihdr_data.bit_depth.as_u8() as usize) >> 3;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if palette.contains(&cur_pixel) {
                    let idx = palette.iter().enumerate().find(|&x| x.1 == &cur_pixel).unwrap().0;
                    reduced.push(idx as u8);
                } else {
                    let len = palette.len();
                    if len == 256 {
                        return None;
                    }
                    palette.push(cur_pixel.clone());
                    reduced.push(len as u8);
                }
                cur_pixel.clear();
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    let mut trans_palette = Vec::with_capacity(palette.len());
    for color in &palette {
        for (i, byte) in color.iter().enumerate() {
            if i < 3 {
                color_palette.push(*byte);
            } else {
                trans_palette.push(*byte);
            }
        }
    }

    Some((reduced, color_palette, trans_palette))
}

fn reduce_rgb_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth != BitDepth::Eight {
        return None;
    }
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = Vec::with_capacity(256);
    let bpp: usize = (3 * png.ihdr_data.bit_depth.as_u8() as usize) >> 3;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if palette.contains(&cur_pixel) {
                    let idx = palette.iter().enumerate().find(|&x| x.1 == &cur_pixel).unwrap().0;
                    reduced.push(idx as u8);
                } else {
                    let len = palette.len();
                    if len == 256 {
                        return None;
                    }
                    palette.push(cur_pixel.clone());
                    reduced.push(len as u8);
                }
                cur_pixel.clear();
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    for color in &palette {
        color_palette.extend_from_slice(color);
    }

    Some((reduced, color_palette))
}

fn reduce_grayscale_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>)> {
    if png.ihdr_data.bit_depth == BitDepth::Sixteen {
        return None;
    }
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    // Only perform reduction if we can get to 4-bits or less
    let mut palette = Vec::with_capacity(16);
    let bpp: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let bpp_inverse = 8 - bpp;
    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        let mut cur_pixel = BitVec::with_capacity(bpp);
        for (i, bit) in bit_vec.iter().enumerate() {
            cur_pixel.push(bit);
            if i % bpp == bpp - 1 {
                let pix_value = cur_pixel.to_bytes()[0] >> bpp_inverse;
                let pix_slice = vec![pix_value, pix_value, pix_value];
                if palette.contains(&pix_slice) {
                    let index = palette.iter().enumerate().find(|&x| x.1 == &pix_slice).unwrap().0;
                    let idx = BitVec::from_bytes(&[(index as u8) << bpp_inverse]);
                    for b in idx.iter().take(bpp) {
                        reduced.push(b);
                    }
                } else {
                    let len = palette.len();
                    if len == 16 {
                        return None;
                    }
                    palette.push(pix_slice.clone());
                    let idx = BitVec::from_bytes(&[(len as u8) << bpp_inverse]);
                    for b in idx.iter().take(bpp) {
                        reduced.push(b);
                    }
                }
                cur_pixel = BitVec::with_capacity(bpp);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * 3);
    for color in &palette {
        color_palette.extend_from_slice(color);
    }

    Some((reduced.to_bytes(), color_palette))
}

fn reduce_palette_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let mut cur_pixel = Vec::with_capacity(3);
    let palette = png.palette.clone().unwrap();
    // Iterate through palette and determine if all colors are grayscale
    for byte in &palette {
        cur_pixel.push(*byte);
        if cur_pixel.len() == 3 {
            cur_pixel.sort();
            cur_pixel.dedup();
            if cur_pixel.len() > 1 {
                return None;
            }
            cur_pixel.clear();
        }
    }

    // Iterate through scanlines and assign grayscale value to each pixel
    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    let bit_depth_inverse = 8 - bit_depth;
    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        let mut cur_pixel = BitVec::with_capacity(bit_depth);
        for bit in bit_vec {
            // Handle bit depths less than 8-bits
            // At the end of each pixel, push its grayscale value onto the reduced image
            cur_pixel.push(bit);
            if cur_pixel.len() == bit_depth {
                // `to_bytes` gives us e.g. 10000000 for a 1-bit pixel, when we would want 00000001
                let padded_pixel = cur_pixel.to_bytes()[0] >> bit_depth_inverse;
                let palette_idx: usize = padded_pixel as usize * 3;
                reduced.extend(BitVec::from_bytes(&[palette[palette_idx]]));
                // BitVec's clear function doesn't set len to 0
                cur_pixel = BitVec::with_capacity(bit_depth);
            }
        }
        // Pad end of line to get 8 bits per byte
        while reduced.len() % 8 != 0 {
            reduced.push(false);
        }
    }

    Some(reduced.to_bytes())
}

fn reduce_rgb_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 3 * byte_depth as usize;
    let mut cur_pixel = Vec::with_capacity(bpp);
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if bpp == 3 {
                    cur_pixel.sort();
                    cur_pixel.dedup();
                    if cur_pixel.len() > 1 {
                        return None;
                    }
                    reduced.push(cur_pixel[0]);
                } else {
                    let mut pixel_zip = cur_pixel.iter()
                                                 .enumerate()
                                                 .filter(|&(i, _)| i % 2 == 0)
                                                 .map(|(_, x)| *x)
                                                 .zip(cur_pixel.iter()
                                                               .enumerate()
                                                               .filter(|&(i, _)| i % 2 == 1)
                                                               .map(|(_, x)| *x))
                                                 .collect::<Vec<(u8, u8)>>();
                    pixel_zip.sort();
                    pixel_zip.dedup();
                    if pixel_zip.len() > 1 {
                        return None;
                    }
                    reduced.push(pixel_zip[0].0);
                    reduced.push(pixel_zip[0].1);
                }
                cur_pixel.clear();
            }
        }
    }

    Some(reduced)
}

fn reduce_grayscale_alpha_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 2 * byte_depth as usize;
    let colored_bytes = bpp - byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp >= colored_bytes {
                if *byte != 255 {
                    return None;
                }
            } else {
                reduced.push(*byte);
            }
        }
    }

    Some(reduced)
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn file_header_is_valid(bytes: &[u8]) -> bool {
    let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    bytes.iter().zip(expected_header.iter()).all(|x| x.0 == x.1)
}

fn parse_next_header(byte_data: &[u8],
                     byte_offset: &mut usize,
                     fix_errors: bool)
                     -> Result<Option<(String, Vec<u8>)>, String> {
    let mut rdr = Cursor::new(byte_data.iter()
                                       .skip(*byte_offset)
                                       .take(4)
                                       .cloned()
                                       .collect::<Vec<u8>>());
    let length: u32 = match rdr.read_u32::<BigEndian>() {
        Ok(x) => x,
        Err(_) => return Err("Invalid data found--unable to read PNG file".to_owned()),
    };
    *byte_offset += 4;

    let mut header_bytes: Vec<u8> = byte_data.iter().skip(*byte_offset).take(4).cloned().collect();
    let header = match String::from_utf8(header_bytes.clone()) {
        Ok(x) => x,
        Err(_) => return Err("Invalid data found--unable to read PNG file".to_owned()),
    };
    if header == "IEND" {
        // End of data
        return Ok(None);
    }
    *byte_offset += 4;

    let data: Vec<u8> = byte_data.iter()
                                 .skip(*byte_offset)
                                 .take(length as usize)
                                 .cloned()
                                 .collect();
    *byte_offset += length as usize;
    let mut rdr = Cursor::new(byte_data.iter()
                                       .skip(*byte_offset)
                                       .take(4)
                                       .cloned()
                                       .collect::<Vec<u8>>());
    let crc: u32 = match rdr.read_u32::<BigEndian>() {
        Ok(x) => x,
        Err(_) => return Err("Invalid data found--unable to read PNG file".to_owned()),
    };
    *byte_offset += 4;
    header_bytes.extend(data.clone());
    if !fix_errors && crc32::checksum_ieee(header_bytes.as_ref()) != crc {
        return Err(format!("Corrupt data chunk found--CRC Mismatch in {}\nThis may be recoverable by using --fix",
                           header));
    }

    Ok(Some((header, data)))
}

fn parse_ihdr_header(byte_data: &[u8]) -> Result<IhdrData, String> {
    let mut rdr = Cursor::new(&byte_data[0..8]);
    Ok(IhdrData {
        color_type: match byte_data[9] {
            0 => ColorType::Grayscale,
            2 => ColorType::RGB,
            3 => ColorType::Indexed,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
            _ => return Err("Unexpected color type in header".to_owned()),
        },
        bit_depth: match byte_data[8] {
            1 => BitDepth::One,
            2 => BitDepth::Two,
            4 => BitDepth::Four,
            8 => BitDepth::Eight,
            16 => BitDepth::Sixteen,
            _ => return Err("Unexpected bit depth in header".to_owned()),
        },
        width: rdr.read_u32::<BigEndian>().unwrap(),
        height: rdr.read_u32::<BigEndian>().unwrap(),
        compression: byte_data[10],
        filter: byte_data[11],
        interlaced: byte_data[12],
    })
}

fn write_png_block(key: &[u8], header: &[u8], output: &mut Vec<u8>) {
    let mut header_data = Vec::with_capacity(header.len() + 4);
    header_data.extend_from_slice(key);
    header_data.extend_from_slice(header);
    output.reserve(header_data.len() + 8);
    output.write_u32::<BigEndian>(header_data.len() as u32 - 4).ok();
    let crc = crc32::checksum_ieee(&header_data);
    output.append(&mut header_data);
    output.write_u32::<BigEndian>(crc).ok();
}
