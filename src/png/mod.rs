use bit_vec::BitVec;
use byteorder::{BigEndian, WriteBytesExt};
use colors::{AlphaOptim, BitDepth, ColorType};
use crc::crc32;
use deflate;
use error::PngError;
use filters::*;
use headers::*;
use interlace::{deinterlace_image, interlace_image};
use itertools::Itertools;
use reduction::bit_depth::*;
use reduction::color::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter::Iterator;
use std::path::Path;

const STD_COMPRESSION: u8 = 8;
const STD_MEMORY: u8 = 9;
const STD_STRATEGY: u8 = 2; // Huffman only
const STD_WINDOW: u8 = 15;
const STD_FILTERS: [u8; 2] = [0, 5];

mod scan_lines;

use self::scan_lines::{ScanLine, ScanLines};

#[derive(Debug, Clone)]
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

impl PngData {
    /// Create a new `PngData` struct by opening a file
    #[inline]
    #[allow(dead_code)]
    pub fn new(filepath: &Path, fix_errors: bool) -> Result<PngData, PngError> {
        let byte_data = PngData::read_file(filepath)?;

        PngData::from_slice(&byte_data, fix_errors)
    }

    pub fn read_file(filepath: &Path) -> Result<Vec<u8>, PngError> {
        let mut file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return Err(PngError::new("Failed to open file for reading")),
        };
        // Check file for PNG header
        let mut header = [0; 8];
        if file.read_exact(&mut header).is_err() {
            return Err(PngError::new("Not a PNG file: too small"));
        }
        if !file_header_is_valid(&header) {
            return Err(PngError::new("Invalid PNG header detected"));
        }
        if file.seek(SeekFrom::Start(0)).is_err() {
            return Err(PngError::new("Failed to read from file"));
        }
        // Read raw png data into memory
        let mut byte_data: Vec<u8> =
            Vec::with_capacity(file.metadata().map(|m| m.len() as usize).unwrap_or(0));
        match file.read_to_end(&mut byte_data) {
            Ok(_) => (),
            Err(_) => return Err(PngError::new("Failed to read from file")),
        }
        Ok(byte_data)
    }

    /// Create a new `PngData` struct by reading a slice
    pub fn from_slice(byte_data: &[u8], fix_errors: bool) -> Result<PngData, PngError> {
        let mut byte_offset: usize = 0;
        // Test that png header is valid
        let header: Vec<u8> = byte_data.iter().take(8).cloned().collect();
        if !file_header_is_valid(header.as_ref()) {
            return Err(PngError::new("Invalid PNG header detected"));
        }
        byte_offset += 8;
        // Read the data headers
        let mut aux_headers: HashMap<String, Vec<u8>> = HashMap::new();
        let mut idat_headers: Vec<u8> = Vec::new();
        loop {
            let header = parse_next_header(byte_data, &mut byte_offset, fix_errors);
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
            } else if header.0 == "acTL" {
                return Err(PngError::new("APNG files are not (yet) supported"));
            } else {
                aux_headers.insert(header.0, header.1);
            }
        }
        // Parse the headers into our PngData
        if idat_headers.is_empty() {
            return Err(PngError::new("Image data was empty, skipping"));
        }
        if aux_headers.get("IHDR").is_none() {
            return Err(PngError::new("Image header data was missing, skipping"));
        }
        let ihdr_header = match parse_ihdr_header(aux_headers.remove("IHDR").unwrap().as_ref()) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        let raw_data = match deflate::inflate(idat_headers.as_ref()) {
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
            idat_data: idat_headers,
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

    #[doc(hidden)]
    pub fn reset_from_original(&mut self, original: &PngData) {
        self.idat_data = original.idat_data.clone();
        self.ihdr_data = original.ihdr_data;
        self.raw_data = original.raw_data.clone();
        self.palette = original.palette.clone();
        self.transparency_pixel = original.transparency_pixel.clone();
        self.transparency_palette = original.transparency_palette.clone();
        self.aux_headers = original.aux_headers.clone();
    }

    /// Return the number of channels in the image, based on color type
    #[inline]
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
        let _ = ihdr_data.write_u32::<BigEndian>(self.ihdr_data.width);
        let _ = ihdr_data.write_u32::<BigEndian>(self.ihdr_data.height);
        let _ = ihdr_data.write_u8(self.ihdr_data.bit_depth.as_u8());
        let _ = ihdr_data.write_u8(self.ihdr_data.color_type.png_header_code());
        let _ = ihdr_data.write_u8(0); // Compression -- deflate
        let _ = ihdr_data.write_u8(0); // Filter method -- 5-way adaptive filtering
        let _ = ihdr_data.write_u8(self.ihdr_data.interlaced);
        write_png_block(b"IHDR", &ihdr_data, &mut output);
        // Ancillary headers
        for (key, header) in self.aux_headers
            .iter()
            .filter(|&(key, _)| !(*key == "bKGD" || *key == "hIST" || *key == "tRNS"))
        {
            write_png_block(key.as_bytes(), header, &mut output);
        }
        // Palette
        if let Some(ref palette) = self.palette {
            write_png_block(b"PLTE", palette, &mut output);
            if let Some(ref transparency_palette) = self.transparency_palette {
                // Transparency pixel
                write_png_block(b"tRNS", transparency_palette, &mut output);
            }
        } else if let Some(ref transparency_pixel) = self.transparency_pixel {
            // Transparency pixel
            write_png_block(b"tRNS", transparency_pixel, &mut output);
        }
        // Special ancillary headers that need to come after PLTE but before IDAT
        for (key, header) in self.aux_headers
            .iter()
            .filter(|&(key, _)| *key == "bKGD" || *key == "hIST" || *key == "tRNS")
        {
            write_png_block(key.as_bytes(), header, &mut output);
        }
        // IDAT data
        write_png_block(b"IDAT", &self.idat_data, &mut output);
        // Stream end
        write_png_block(b"IEND", &[], &mut output);

        output
    }

    /// Return an iterator over the scanlines of the image
    #[inline]
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
        let bpp = ((f32::from(self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel())) / 8f32)
            .ceil() as usize;
        let mut last_line: Vec<u8> = Vec::new();
        let mut last_pass = 1;
        for line in self.scan_lines() {
            if let Some(pass) = line.pass {
                if pass != last_pass {
                    last_line = Vec::new();
                    last_pass = pass;
                }
            }
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
        let bpp = ((f32::from(self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel())) / 8f32)
            .ceil() as usize;
        let mut last_line: Vec<u8> = Vec::new();
        let mut last_pass: Option<u8> = None;
        for line in self.scan_lines() {
            match filter {
                0 | 1 | 2 | 3 | 4 => {
                    if last_pass == line.pass || filter <= 1 {
                        filtered.push(filter);
                        filtered.extend_from_slice(&filter_line(
                            filter,
                            bpp,
                            &line.data,
                            &last_line,
                        ));
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
                    for filter in if last_pass == line.pass { 0..5 } else { 0..2 } {
                        trials.insert(filter, filter_line(filter, bpp, &line.data, &last_line));
                    }
                    let (best_filter, best_line) = trials
                        .iter()
                        .min_by_key(|x| {
                            x.1.iter().fold(0u64, |acc, &x| {
                                let signed = x as i8;
                                acc + i16::from(signed).abs() as u64
                            })
                        })
                        .unwrap();
                    filtered.push(*best_filter);
                    filtered.extend_from_slice(best_line);
                }
                _ => unreachable!(),
            }
            last_line = line.data;
            last_pass = line.pass;
        }
        filtered
    }

    /// Attempt to reduce the bit depth of the image
    /// Returns true if the bit depth was reduced, false otherwise
    pub fn reduce_bit_depth(&mut self) -> bool {
        if self.ihdr_data.bit_depth != BitDepth::Sixteen {
            if self.ihdr_data.color_type == ColorType::Indexed
                || self.ihdr_data.color_type == ColorType::Grayscale
            {
                return reduce_bit_depth_8_or_less(self);
            }
            return false;
        }

        // Reduce from 16 to 8 bits per channel per pixel
        let mut reduced = Vec::with_capacity(
            (self.ihdr_data.width * self.ihdr_data.height * u32::from(self.channels_per_pixel())
                + self.ihdr_data.height) as usize,
        );
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

        // A palette with RGB or RGBA slices
        let palette = if let Some(ref trns) = self.transparency_palette {
            self.palette
                .clone()
                .unwrap()
                .chunks(3)
                .zip(trns.iter().chain([255].iter().cycle()))
                .flat_map(|(pixel, trns)| {
                    let mut pixel = pixel.to_owned();
                    pixel.push(*trns);
                    pixel
                })
                .collect()
        } else {
            self.palette.clone().unwrap()
        };
        let mut indexed_palette: Vec<&[u8]> = palette
            .chunks(if self.transparency_palette.is_some() {
                4
            } else {
                3
            })
            .collect();
        // A map of old indexes to new ones, for any moved
        let mut index_map: HashMap<u8, u8> = HashMap::new();

        // A list of (original) indices that are duplicates and no longer needed
        let mut duplicates: Vec<u8> = Vec::new();
        {
            // Find duplicate entries in the palette
            let mut seen: HashMap<&[u8], u8> = HashMap::with_capacity(indexed_palette.len());
            for (i, color) in indexed_palette.iter().enumerate() {
                if seen.contains_key(color) {
                    let index = &seen[color];
                    duplicates.push(i as u8);
                    index_map.insert(i as u8, *index);
                } else {
                    seen.insert(*color, i as u8);
                }
            }
        }

        // Remove duplicates from the data
        if !duplicates.is_empty() {
            self.do_palette_reduction(&duplicates, &mut index_map, &mut indexed_palette);
        }

        // Find palette entries that are never used
        let mut seen = HashSet::with_capacity(indexed_palette.len());
        for line in self.scan_lines() {
            match self.ihdr_data.bit_depth {
                BitDepth::Eight => for byte in &line.data {
                    seen.insert(*byte);
                },
                BitDepth::Four => {
                    let bitvec = BitVec::from_bytes(&line.data);
                    let mut current = 0u8;
                    for (i, bit) in bitvec.iter().enumerate() {
                        let mod_i = i % 4;
                        if bit {
                            current += 1u8 << (3 - mod_i);
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
                            current += 1u8 << (1 - mod_i);
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

        let unused: Vec<u8> = (0..indexed_palette.len() as u8)
            .filter(|i| !seen.contains(i))
            .collect();

        // Remove unused palette indices
        self.do_palette_reduction(&unused, &mut index_map, &mut indexed_palette);

        true
    }

    fn do_palette_reduction(
        &mut self,
        indices_to_remove: &[u8],
        index_map: &mut HashMap<u8, u8>,
        indexed_palette: &mut Vec<&[u8]>,
    ) {
        let mut new_data = Vec::with_capacity(self.raw_data.len());
        let original_len = indexed_palette.len();
        for idx in indices_to_remove.iter().sorted_by(|a, b| b.cmp(a)) {
            for i in (*idx as usize + 1)..original_len {
                let existing = index_map.entry(i as u8).or_insert(i as u8);
                if *existing >= *idx {
                    *existing -= 1;
                }
            }
            indexed_palette.remove(*idx as usize);
            if let Some(ref mut alpha) = self.transparency_palette {
                if (*idx as usize) < alpha.len() {
                    alpha.remove(*idx as usize);
                }
            }
        }
        if let Some(ref mut alpha) = self.transparency_palette {
            while let Some(255) = alpha.last().cloned() {
                alpha.pop();
            }
        }
        // Reassign data bytes to new indices
        for line in self.scan_lines() {
            new_data.push(line.filter);
            match self.ihdr_data.bit_depth {
                BitDepth::Eight => for byte in &line.data {
                    if let Some(new_idx) = index_map.get(byte) {
                        new_data.push(*new_idx);
                    } else {
                        new_data.push(*byte);
                    }
                },
                BitDepth::Four => for byte in &line.data {
                    let upper = *byte & 0b1111_0000;
                    let lower = *byte & 0b0000_1111;
                    let mut new_byte = 0u8;
                    new_byte |= if let Some(new_idx) = index_map.get(&(upper >> 4)) {
                        *new_idx << 4
                    } else {
                        upper
                    };
                    new_byte |= if let Some(new_idx) = index_map.get(&lower) {
                        *new_idx
                    } else {
                        lower
                    };
                    new_data.push(new_byte);
                },
                BitDepth::Two => for byte in &line.data {
                    let one = *byte & 0b1100_0000;
                    let two = *byte & 0b0011_0000;
                    let three = *byte & 0b0000_1100;
                    let four = *byte & 0b0000_0011;
                    let mut new_byte = 0u8;
                    new_byte |= if let Some(new_idx) = index_map.get(&(one >> 6)) {
                        *new_idx << 6
                    } else {
                        one
                    };
                    new_byte |= if let Some(new_idx) = index_map.get(&(two >> 4)) {
                        *new_idx << 4
                    } else {
                        two
                    };
                    new_byte |= if let Some(new_idx) = index_map.get(&(three >> 2)) {
                        *new_idx << 2
                    } else {
                        three
                    };
                    new_byte |= if let Some(new_idx) = index_map.get(&four) {
                        *new_idx
                    } else {
                        four
                    };
                    new_data.push(new_byte);
                },
                _ => unreachable!(),
            }
        }
        index_map.clear();
        self.raw_data = new_data;
        let new_palette = indexed_palette
            .iter()
            .cloned()
            .flatten()
            .enumerate()
            .filter(|&(i, _)| !(self.transparency_palette.is_some() && i % 4 == 3))
            .map(|(_, x)| *x)
            .collect::<Vec<u8>>();
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
            if reduce_rgba_to_grayscale_alpha(self) || reduce_rgba_to_rgb(self) {
                changed = true;
            } else if reduce_rgba_to_palette(self) {
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::GrayscaleAlpha
            && reduce_grayscale_alpha_to_grayscale(self)
        {
            changed = true;
            should_reduce_bit_depth = true;
        }

        if self.ihdr_data.color_type == ColorType::RGB
            && (reduce_rgb_to_grayscale(self) || reduce_rgb_to_palette(self))
        {
            changed = true;
            should_reduce_bit_depth = true;
        }

        if should_reduce_bit_depth {
            // Some conversions will allow us to perform bit depth reduction that
            // wasn't possible before
            reduce_bit_depth_8_or_less(self);
        }

        changed
    }

    pub fn try_alpha_reduction(&mut self, alphas: &HashSet<AlphaOptim>) {
        assert!(!alphas.is_empty());
        let best = alphas
            .iter()
            .map(|alpha| {
                let mut image = self.clone();
                image.reduce_alpha_channel(*alpha);
                let size = STD_FILTERS
                    .iter()
                    .map(|f| {
                        deflate::deflate(
                            &image.filter_image(*f),
                            STD_COMPRESSION,
                            STD_MEMORY,
                            STD_STRATEGY,
                            STD_WINDOW,
                        ).unwrap()
                            .len()
                    })
                    .min()
                    .unwrap();
                (size, image)
            })
            .min_by_key(|&(size, _)| size)
            .unwrap();

        self.raw_data = best.1.raw_data;
    }

    pub fn reduce_alpha_channel(&mut self, optim: AlphaOptim) -> bool {
        let (bpc, bpp) = match self.ihdr_data.color_type {
            ColorType::RGBA | ColorType::GrayscaleAlpha => {
                let cpp = self.channels_per_pixel();
                let bpc = self.ihdr_data.bit_depth.as_u8() / 8;
                (bpc as usize, (bpc * cpp) as usize)
            }
            _ => {
                return false;
            }
        };

        match optim {
            AlphaOptim::NoOp => {
                return false;
            }
            AlphaOptim::Black => {
                self.raw_data = self.reduce_alpha_to_black(bpc, bpp);
            }
            AlphaOptim::White => {
                self.raw_data = self.reduce_alpha_to_white(bpc, bpp);
            }
            AlphaOptim::Up => {
                self.raw_data = self.reduce_alpha_to_up(bpc, bpp);
            }
            AlphaOptim::Down => {
                self.raw_data = self.reduce_alpha_to_down(bpc, bpp);
            }
            AlphaOptim::Left => {
                self.raw_data = self.reduce_alpha_to_left(bpc, bpp);
            }
            AlphaOptim::Right => {
                self.raw_data = self.reduce_alpha_to_right(bpc, bpp);
            }
        }

        true
    }

    fn reduce_alpha_to_black(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        for line in self.scan_lines() {
            reduced.push(line.filter);
            for pixel in line.data.chunks(bpp) {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    for _ in 0..bpp {
                        reduced.push(0);
                    }
                } else {
                    reduced.extend_from_slice(pixel);
                }
            }
        }
        reduced
    }

    fn reduce_alpha_to_white(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        for line in self.scan_lines() {
            reduced.push(line.filter);
            for pixel in line.data.chunks(bpp) {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    for _ in 0..(bpp - bpc) {
                        reduced.push(255);
                    }
                    for _ in 0..bpc {
                        reduced.push(0);
                    }
                } else {
                    reduced.extend_from_slice(pixel);
                }
            }
        }
        reduced
    }

    fn reduce_alpha_to_up(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut lines = Vec::new();
        let scan_lines = self.scan_lines().collect::<Vec<ScanLine>>();
        let mut last_line = vec![0; scan_lines[0].data.len()];
        let mut current_line = Vec::with_capacity(last_line.len());
        for line in scan_lines.into_iter().rev() {
            current_line.push(line.filter);
            for (pixel, last_pixel) in line.data.chunks(bpp).zip(last_line.chunks(bpp)) {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    current_line.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                    for _ in 0..bpc {
                        current_line.push(0);
                    }
                } else {
                    current_line.extend_from_slice(pixel);
                }
            }
            last_line = current_line.clone();
            lines.push(current_line.clone());
            current_line.clear();
        }
        lines.into_iter().rev().flatten().collect()
    }

    fn reduce_alpha_to_down(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        let mut last_line = vec![0; self.scan_lines().next().unwrap().data.len()];
        for line in self.scan_lines() {
            reduced.push(line.filter);
            for (pixel, last_pixel) in line.data.chunks(bpp).zip(last_line.chunks(bpp)) {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    reduced.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                    for _ in 0..bpc {
                        reduced.push(0);
                    }
                } else {
                    reduced.extend_from_slice(pixel);
                }
            }
            last_line = reduced.clone();
        }
        reduced
    }

    fn reduce_alpha_to_left(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        for line in self.scan_lines() {
            let mut line_bytes = Vec::with_capacity(line.data.len());
            let mut last_pixel = vec![0; bpp];
            for pixel in line.data.chunks(bpp).rev() {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    line_bytes.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                    for _ in 0..bpc {
                        line_bytes.push(0);
                    }
                } else {
                    line_bytes.extend_from_slice(pixel);
                }
                last_pixel = pixel.to_owned();
            }
            reduced.push(line.filter);
            reduced.extend(line_bytes.chunks(bpp).rev().flatten());
        }
        reduced
    }

    fn reduce_alpha_to_right(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        for line in self.scan_lines() {
            reduced.push(line.filter);
            let mut last_pixel = vec![0; bpp];
            for pixel in line.data.chunks(bpp) {
                if pixel.iter().skip(bpp - bpc).fold(0, |sum, i| sum | i) == 0 {
                    reduced.extend_from_slice(&last_pixel[0..(bpp - bpc)]);
                    for _ in 0..bpc {
                        reduced.push(0);
                    }
                } else {
                    reduced.extend_from_slice(pixel);
                }
                last_pixel = pixel.to_owned();
            }
        }
        reduced
    }

    /// Convert the image to the specified interlacing type
    /// Returns true if the interlacing was changed, false otherwise
    /// The `interlace` parameter specifies the *new* interlacing mode
    /// Assumes that the data has already been de-filtered
    #[inline]
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

fn write_png_block(key: &[u8], header: &[u8], output: &mut Vec<u8>) {
    let mut header_data = Vec::with_capacity(header.len() + 4);
    header_data.extend_from_slice(key);
    header_data.extend_from_slice(header);
    output.reserve(header_data.len() + 8);
    let _ = output.write_u32::<BigEndian>(header_data.len() as u32 - 4);
    let crc = crc32::checksum_ieee(&header_data);
    output.append(&mut header_data);
    let _ = output.write_u32::<BigEndian>(crc);
}
