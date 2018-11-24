use std::collections::hash_map::Entry::*;
use rgb::RGBA8;
use rgb::ComponentSlice;
use atomicmin::AtomicMin;
use byteorder::{BigEndian, WriteBytesExt};
use colors::{AlphaOptim, BitDepth, ColorType};
use crc::crc32;
use deflate;
use error::PngError;
use filters::*;
use headers::*;
use interlace::{deinterlace_image, interlace_image};
use itertools::flatten;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use reduction::bit_depth::*;
use reduction::color::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::iter::Iterator;
use std::path::Path;

const STD_COMPRESSION: u8 = 8;
/// Must use normal compression, as faster ones (Huffman/RLE-only) are not representative
const STD_STRATEGY: u8 = 0;
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
    pub palette: Option<Vec<RGBA8>>,
    /// The pixel value that should be rendered as transparent
    pub transparency_pixel: Option<Vec<u8>>,
    /// All non-critical headers from the PNG are stored here
    pub aux_headers: HashMap<[u8; 4], Vec<u8>>,
}

impl PngData {
    /// Create a new `PngData` struct by opening a file
    #[inline]
    pub fn new(filepath: &Path, fix_errors: bool) -> Result<Self, PngError> {
        let byte_data = Self::read_file(filepath)?;

        Self::from_slice(&byte_data, fix_errors)
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
    pub fn from_slice(byte_data: &[u8], fix_errors: bool) -> Result<Self, PngError> {
        let mut byte_offset: usize = 0;
        // Test that png header is valid
        let header = byte_data.get(0..8).ok_or(PngError::TruncatedData)?;
        if !file_header_is_valid(header) {
            return Err(PngError::NotPNG);
        }
        byte_offset += 8;
        // Read the data headers
        let mut aux_headers: HashMap<[u8; 4], Vec<u8>> = HashMap::new();
        let mut idat_headers: Vec<u8> = Vec::new();
        while let Some(header) = parse_next_header(byte_data, &mut byte_offset, fix_errors)? {
            match &header.name {
                b"IDAT" => idat_headers.extend(header.data),
                b"acTL" => return Err(PngError::APNGNotSupported),
                _ => {
                    aux_headers.insert(header.name, header.data.to_owned());
                }
            }
        }
        // Parse the headers into our PngData
        if idat_headers.is_empty() {
            return Err(PngError::ChunkMissing("IDAT"));
        }
        let ihdr = match aux_headers.remove(b"IHDR") {
            Some(ihdr) => ihdr,
            None => return Err(PngError::ChunkMissing("IHDR")),
        };
        let ihdr_header = parse_ihdr_header(&ihdr)?;
        let raw_data = deflate::inflate(idat_headers.as_ref())?;

        let (palette, transparency_pixel) = Self::palette_to_rgba(ihdr_header.color_type, aux_headers.remove(b"PLTE"), aux_headers.remove(b"tRNS"))?;

        let mut png_data = Self {
            idat_data: idat_headers,
            ihdr_data: ihdr_header,
            raw_data,
            palette,
            transparency_pixel,
            aux_headers,
        };
        png_data.raw_data = png_data.unfilter_image();
        // Return the PngData
        Ok(png_data)
    }

    /// Handle transparency header
    fn palette_to_rgba(color_type: ColorType, palette_data: Option<Vec<u8>>, trns_data: Option<Vec<u8>>) -> Result<(Option<Vec<RGBA8>>, Option<Vec<u8>>), PngError> {
        if color_type == ColorType::Indexed {
            let palette_data = palette_data.ok_or_else(|| PngError::new("no palette in indexed image"))?;
            let mut palette: Vec<_> = palette_data.chunks(3)
                .map(|color| RGBA8::new(color[0], color[1], color[2], 255))
                .collect();

            if let Some(trns_data) = trns_data {
                for (color, trns) in palette.iter_mut().zip(trns_data) {
                    color.a = trns;
                }
            }
            Ok((Some(palette), None))
        } else {
            Ok((None, trns_data))
        }
    }

    pub(crate) fn reset_from_original(&mut self, original: &Self) {
        self.idat_data = original.idat_data.clone();
        self.ihdr_data = original.ihdr_data;
        self.raw_data = original.raw_data.clone();
        self.palette = original.palette.clone();
        self.transparency_pixel = original.transparency_pixel.clone();
        self.aux_headers = original.aux_headers.clone();
    }

    /// Return the number of channels in the image, based on color type
    #[inline]
    pub fn channels_per_pixel(&self) -> u8 {
        self.ihdr_data.color_type.channels_per_pixel()
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
        for (key, header) in self
            .aux_headers
            .iter()
            .filter(|&(key, _)| !(key == b"bKGD" || key == b"hIST" || key == b"tRNS"))
        {
            write_png_block(key, header, &mut output);
        }
        // Palette
        if let Some(ref palette) = self.palette {
            let mut palette_data = Vec::with_capacity(palette.len()*3);
            for px in palette {
                palette_data.extend_from_slice(px.rgb().as_slice());
            }
            write_png_block(b"PLTE", &palette_data, &mut output);
            let num_transparent = palette.iter().enumerate().fold(0, |prev, (index, px)| {
                if px.a != 255 {index+1} else {prev}
            });
            if num_transparent > 0 {
                let trns_data: Vec<_> = palette[0..num_transparent].iter().map(|px| px.a).collect();
                write_png_block(b"tRNS", &trns_data, &mut output);
            }
        } else if let Some(ref transparency_pixel) = self.transparency_pixel {
            // Transparency pixel
            write_png_block(b"tRNS", transparency_pixel, &mut output);
        }
        // Special ancillary headers that need to come after PLTE but before IDAT
        for (key, header) in self
            .aux_headers
            .iter()
            .filter(|&(key, _)| key == b"bKGD" || key == b"hIST" || key == b"tRNS")
        {
            write_png_block(key, header, &mut output);
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
        ScanLines::new(self)
    }

    /// Reverse all filters applied on the image, returning an unfiltered IDAT bytestream
    pub fn unfilter_image(&self) -> Vec<u8> {
        let mut unfiltered = Vec::with_capacity(self.raw_data.len());
        let bpp = ((self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel() + 7) / 8) as usize;
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
        let bpp = ((self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel() + 7) / 8) as usize;
        let mut last_line: &[u8] = &[];
        let mut last_pass: Option<u8> = None;
        for line in self.scan_lines() {
            match filter {
                0 | 1 | 2 | 3 | 4 => {
                    let filter = if last_pass == line.pass || filter <= 1 {
                        filter
                    } else {
                        0
                    };
                    filtered.push(filter);
                    filtered.extend_from_slice(&filter_line(filter, bpp, &line.data, last_line));
                }
                5 => {
                    // Heuristically guess best filter per line
                    // Uses MSAD algorithm mentioned in libpng reference docs
                    // http://www.libpng.org/pub/png/book/chapter09.html
                    let mut trials: Vec<(u8, Vec<u8>)> = Vec::with_capacity(5);
                    // Avoid vertical filtering on first line of each interlacing pass
                    for filter in if last_pass == line.pass { 0..5 } else { 0..2 } {
                        trials.push((filter, filter_line(filter, bpp, &line.data, last_line)));
                    }
                    let (best_filter, best_line) = trials
                        .iter()
                        .min_by_key(|(_, line)| {
                            line.iter().fold(0u64, |acc, &x| {
                                let signed = x as i8;
                                acc + i16::from(signed).abs() as u64
                            })
                        }).unwrap();
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
            for (i, &byte) in line.data.iter().enumerate() {
                if i % 2 == 0 {
                    // High byte
                    high_byte = byte;
                } else {
                    // Low byte
                    if high_byte != byte {
                        // Can't reduce, exit early
                        return false;
                    }
                    reduced.push(byte);
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

        let mut palette_map = [0u8; 256];
        let mut used = [false; 256];
        {
            let palette = match self.palette {
                Some(ref p) => p,
                None => return false,
            };

            // Find palette entries that are never used
            for line in self.scan_lines() {
                match self.ihdr_data.bit_depth {
                    BitDepth::Eight => for &byte in line.data {
                        used[byte as usize] = true;
                    },
                    BitDepth::Four => for &byte in line.data {
                        used[(byte & 0x0F) as usize] = true;
                        used[(byte >> 4) as usize] = true;
                    },
                    BitDepth::Two => for &byte in line.data {
                        used[(byte & 0x03) as usize] = true;
                        used[((byte >> 2) & 0x03) as usize] = true;
                        used[((byte >> 4) & 0x03) as usize] = true;
                        used[(byte >> 6) as usize] = true;
                    },
                    _ => unreachable!(),
                }
            }

            let mut next_index = 0;
            let mut seen = HashMap::with_capacity(palette.len());
            for (i, (used, palette_map)) in used.iter().cloned().zip(palette_map.iter_mut()).enumerate() {
                if !used {
                    continue;
                }
                // There are invalid files that use pixel indices beyond palette size
                let color = palette.get(i).cloned().unwrap_or(RGBA8::new(0,0,0,255));
                match seen.entry(color) {
                    Vacant(new) => {
                        *palette_map = next_index;
                        new.insert(next_index);
                        next_index += 1;
                    },
                    Occupied(remap_to) => {
                        *palette_map = *remap_to.get();
                    },
                }
            }
            if (0..palette.len()).all(|i| palette_map[i] == i as u8) {
                return false;
            }
        }

        self.do_palette_reduction(&palette_map, &used);
        true
    }

    fn do_palette_reduction(&mut self, palette_map: &[u8; 256], used: &[bool; 256]) {
        let mut new_data = Vec::with_capacity(self.raw_data.len());
        let mut byte_map = *palette_map;

        // low bit-depths can be pre-computed for every byte value
        match self.ihdr_data.bit_depth {
            BitDepth::Four => for byte in 0..=255 {
                byte_map[byte as usize] = palette_map[(byte & 0x0F) as usize] |
                    (palette_map[(byte >> 4) as usize] << 4);
            },
            BitDepth::Two => for byte in 0..=255 {
                byte_map[byte as usize] = palette_map[(byte & 0x03) as usize] |
                    (palette_map[((byte >> 2) & 0x03) as usize] << 2) |
                    (palette_map[((byte >> 4) & 0x03) as usize] << 4) |
                    (palette_map[((byte >> 6)) as usize] << 6);
            },
            _ => {}
        }

        // Reassign data bytes to new indices
        for line in self.scan_lines() {
            new_data.push(line.filter);
            for &byte in line.data {
                new_data.push(byte_map[byte as usize])
            }
        }

        self.raw_data = new_data;
        self.transparency_pixel = None;
        if let Some(palette) = self.palette.take() {
            let max_index = palette_map.iter().max().cloned().unwrap_or(0) as usize;
            let mut new_palette = vec![RGBA8::new(0,0,0,255); max_index+1];
            for (color, (map_to, used)) in palette.into_iter().zip(palette_map.iter().cloned().zip(used.iter().cloned())) {
                if used {
                    new_palette[map_to as usize] = color;
                }
            }
            self.palette = Some(new_palette);
        }
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
            } else if reduce_color_to_palette(self) {
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
            && (reduce_rgb_to_grayscale(self) || reduce_color_to_palette(self))
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

    pub fn try_alpha_reduction(&mut self, alphas: &HashSet<AlphaOptim>) -> bool {
        assert!(!alphas.is_empty());
        let alphas = alphas.iter().collect::<Vec<_>>();
        let best_size = AtomicMin::new(None);
        #[cfg(feature = "parallel")]
        let alphas_iter = alphas.par_iter().with_max_len(1);
        #[cfg(not(feature = "parallel"))]
        let alphas_iter = alphas.iter();
        let best = alphas_iter
            .filter_map(|&alpha| {
                let image = match self.reduced_alpha_channel(*alpha) {
                    Some(image) => image,
                    None => return None,
                };
                #[cfg(feature = "parallel")]
                let filters_iter = STD_FILTERS.par_iter().with_max_len(1);
                #[cfg(not(feature = "parallel"))]
                let filters_iter = STD_FILTERS.iter();
                filters_iter
                    .filter_map(|f| {
                        deflate::deflate(
                            &image.filter_image(*f),
                            STD_COMPRESSION,
                            STD_STRATEGY,
                            STD_WINDOW,
                            &best_size,
                        ).ok()
                        .as_ref()
                        .map(|l| {
                            best_size.set_min(l.len());
                            l.len()
                        })
                    }).min()
                    .map(|size| (size, image))
            }).min_by_key(|&(size, _)| size);

        if let Some(best) = best {
            self.raw_data = best.1.raw_data;
            return true;
        }
        false
    }

    /// It doesn't recompress `idat_data`, so this field is out of date
    /// after the reduction.
    pub fn reduced_alpha_channel(&self, optim: AlphaOptim) -> Option<Self> {
        let (bpc, bpp) = match self.ihdr_data.color_type {
            ColorType::RGBA | ColorType::GrayscaleAlpha => {
                let cpp = self.channels_per_pixel();
                let bpc = self.ihdr_data.bit_depth.as_u8() / 8;
                (bpc as usize, (bpc * cpp) as usize)
            }
            _ => {
                return None;
            }
        };

        let raw_data = match optim {
            AlphaOptim::NoOp => return None,
            AlphaOptim::Black => self.reduced_alpha_to_black(bpc, bpp),
            AlphaOptim::White => self.reduced_alpha_to_white(bpc, bpp),
            AlphaOptim::Up => self.reduced_alpha_to_up(bpc, bpp),
            AlphaOptim::Down => self.reduced_alpha_to_down(bpc, bpp),
            AlphaOptim::Left => self.reduced_alpha_to_left(bpc, bpp),
            AlphaOptim::Right => self.reduced_alpha_to_right(bpc, bpp),
        };

        Some(Self {
            raw_data,
            idat_data: vec![],
            ihdr_data: self.ihdr_data,
            palette: self.palette.clone(),
            transparency_pixel: self.transparency_pixel.clone(),
            aux_headers: self.aux_headers.clone(),
        })
    }

    fn reduced_alpha_to_black(&self, bpc: usize, bpp: usize) -> Vec<u8> {
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

    fn reduced_alpha_to_white(&self, bpc: usize, bpp: usize) -> Vec<u8> {
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

    fn reduced_alpha_to_up(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut lines = Vec::new();
        let mut scan_lines = self.scan_lines().collect::<Vec<ScanLine>>();
        scan_lines.reverse();
        let mut last_line = Vec::new();
        let mut current_line = Vec::with_capacity(last_line.len());
        for line in scan_lines {
            if line.data.len() != last_line.len() {
                last_line = vec![0; line.data.len()];
            }
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
        flatten(lines.into_iter().rev()).collect()
    }

    fn reduced_alpha_to_down(&self, bpc: usize, bpp: usize) -> Vec<u8> {
        let mut reduced = Vec::with_capacity(self.raw_data.len());
        let mut last_line = Vec::new();
        for line in self.scan_lines() {
            if line.data.len() != last_line.len() {
                last_line = vec![0; line.data.len()];
            }
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

    fn reduced_alpha_to_left(&self, bpc: usize, bpp: usize) -> Vec<u8> {
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
            reduced.extend(flatten(line_bytes.chunks(bpp).rev()));
        }
        reduced
    }

    fn reduced_alpha_to_right(&self, bpc: usize, bpp: usize) -> Vec<u8> {
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
