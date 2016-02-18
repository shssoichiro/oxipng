use bit_vec::BitVec;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Cursor;
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::Path;

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum ColorType {
    Grayscale,
    RGB,
    Indexed,
    GrayscaleAlpha,
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
pub enum BitDepth {
    One,
    Two,
    Four,
    Eight,
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
    fn as_u8(&self) -> u8 {
        match *self {
            BitDepth::One => 1,
            BitDepth::Two => 2,
            BitDepth::Four => 4,
            BitDepth::Eight => 8,
            BitDepth::Sixteen => 16,
        }
    }
    fn from_u8(depth: u8) -> BitDepth {
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

#[derive(Debug,Clone)]
struct ScanLines<'a> {
    png: &'a PngData,
    start: usize,
    end: usize,
}

impl<'a> ScanLines<'a> {
    fn len(&mut self) -> usize {
        self.png.raw_data.len()
    }
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end == self.len() {
            None
        } else {
            let bits_per_line = self.png.ihdr_data.width as usize *
                                self.png.ihdr_data.bit_depth.as_u8() as usize *
                                self.png.channels_per_pixel() as usize;
            // This avoids casting to and from floats, which is expensive
            let bytes_per_line = (bits_per_line + bits_per_line % 8) >> 3;
            self.start = self.end;
            self.end = self.start + bytes_per_line + 1;
            Some(ScanLine {
                filter: self.png.raw_data[self.start],
                data: self.png.raw_data[(self.start + 1)..self.end].to_owned(),
            })
        }
    }
}

#[derive(Debug,Clone)]
struct ScanLine {
    filter: u8,
    data: Vec<u8>,
}

#[derive(Debug,Clone)]
pub struct PngData {
    pub idat_data: Vec<u8>,
    pub ihdr_data: IhdrData,
    pub raw_data: Vec<u8>,
    pub palette: Option<Vec<u8>>,
    pub transparency_pixel: Option<Vec<u8>>,
    pub transparency_palette: Option<Vec<u8>>,
    pub aux_headers: HashMap<String, Vec<u8>>,
}

#[derive(Debug,Clone,Copy)]
pub struct IhdrData {
    pub width: u32,
    pub height: u32,
    pub color_type: ColorType,
    pub bit_depth: BitDepth,
    pub compression: u8,
    pub filter: u8,
    pub interlaced: u8,
}

impl PngData {
    pub fn new(filepath: &Path) -> Result<PngData, String> {
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
            let header = parse_next_header(byte_data.as_ref(), &mut byte_offset);
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
    pub fn channels_per_pixel(&self) -> u8 {
        match self.ihdr_data.color_type {
            ColorType::Grayscale | ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGB => 3,
            ColorType::RGBA => 4,
        }
    }
    pub fn output(&self) -> Vec<u8> {
        // FIXME: This code can all be refactored
        // PNG header
        let mut output = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR
        let mut ihdr_data = Vec::with_capacity(17);
        ihdr_data.extend_from_slice(b"IHDR");
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.width).ok();
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.height).ok();
        ihdr_data.write_u8(self.ihdr_data.bit_depth.as_u8()).ok();
        ihdr_data.write_u8(self.ihdr_data.color_type.png_header_code()).ok();
        ihdr_data.write_u8(0).ok(); // Compression -- deflate
        ihdr_data.write_u8(0).ok(); // Filter method -- 5-way adaptive filtering
        ihdr_data.write_u8(self.ihdr_data.interlaced).ok();
        output.reserve(ihdr_data.len() + 8);
        output.write_u32::<BigEndian>(ihdr_data.len() as u32 - 4).ok();
        let crc = crc32::checksum_ieee(&ihdr_data);
        output.append(&mut ihdr_data);
        output.write_u32::<BigEndian>(crc).ok();
        // Ancillary headers
        for (key, header) in &self.aux_headers {
            let mut header_data = Vec::with_capacity(header.len() + 4);
            header_data.extend(key.as_bytes());
            header_data.extend_from_slice(header);
            output.reserve(header_data.len() + 8);
            output.write_u32::<BigEndian>(header_data.len() as u32 - 4).ok();
            let crc = crc32::checksum_ieee(&header_data);
            output.append(&mut header_data);
            output.write_u32::<BigEndian>(crc).ok();
        }
        // Palette
        if let Some(palette) = self.palette.clone() {
            let mut palette_data = Vec::with_capacity(palette.len() + 4);
            palette_data.extend_from_slice(b"PLTE");
            palette_data.extend(palette);
            output.reserve(palette_data.len() + 8);
            output.write_u32::<BigEndian>(palette_data.len() as u32 - 4).ok();
            let crc = crc32::checksum_ieee(&palette_data);
            output.append(&mut palette_data);
            output.write_u32::<BigEndian>(crc).ok();
            if let Some(transparency_palette) = self.transparency_palette.clone() {
                // Transparency pixel
                let mut palette_data = Vec::with_capacity(transparency_palette.len() + 4);
                palette_data.extend_from_slice(b"tRNS");
                palette_data.extend(transparency_palette);
                output.reserve(palette_data.len() + 8);
                output.write_u32::<BigEndian>(palette_data.len() as u32 - 4).ok();
                let crc = crc32::checksum_ieee(&palette_data);
                output.append(&mut palette_data);
                output.write_u32::<BigEndian>(crc).ok();
            }
        } else if let Some(transparency_pixel) = self.transparency_pixel.clone() {
            // Transparency pixel
            let mut pixel_data = Vec::with_capacity(transparency_pixel.len() + 4);
            pixel_data.extend_from_slice(b"tRNS");
            pixel_data.extend(transparency_pixel);
            output.reserve(pixel_data.len() + 8);
            output.write_u32::<BigEndian>(pixel_data.len() as u32 - 4).ok();
            let crc = crc32::checksum_ieee(&pixel_data);
            output.append(&mut pixel_data);
            output.write_u32::<BigEndian>(crc).ok();
        }
        // IDAT data
        let mut idat_data = Vec::with_capacity(self.idat_data.len() + 4);
        idat_data.extend_from_slice(b"IDAT");
        idat_data.extend(self.idat_data.clone());
        output.reserve(idat_data.len() + 8);
        output.write_u32::<BigEndian>(idat_data.len() as u32 - 4).ok();
        let crc = crc32::checksum_ieee(&idat_data);
        output.append(&mut idat_data);
        output.write_u32::<BigEndian>(crc).ok();
        // Stream end
        let iend_data = b"IEND";
        output.reserve(iend_data.len() + 8);
        output.write_u32::<BigEndian>(0).ok();
        let crc = crc32::checksum_ieee(iend_data);
        output.extend_from_slice(iend_data);
        output.write_u32::<BigEndian>(crc).ok();

        output
    }
    fn scan_lines(&self) -> ScanLines {
        ScanLines {
            png: &self,
            start: 0,
            end: 0,
        }
    }
    pub fn unfilter_image(&self) -> Vec<u8> {
        let mut unfiltered = Vec::with_capacity(self.raw_data.len());
        // This avoids casting to and from floats, which is expensive
        let tmp = self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel();
        let bpp = (tmp + tmp % 8) >> 3;
        let mut last_line: Vec<u8> = vec![];
        for line in self.scan_lines() {
            unfiltered.push(0);
            match line.filter {
                0 => {
                    let mut data = line.data.clone();
                    last_line = data.clone();
                    unfiltered.append(&mut data);
                }
                1 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        match i.checked_sub(bpp as usize) {
                            Some(x) => data.push(byte.wrapping_add(line.data[x])),
                            None => data.push(*byte),
                        }
                    }
                    last_line = data.clone();
                    unfiltered.append(&mut data);
                }
                2 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            data.push(byte.wrapping_add(last_line[i]));
                        } else {
                            data.push(*byte);
                        };
                    }
                    last_line = data.clone();
                    unfiltered.append(&mut data);
                }
                3 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add(
                                    ((line.data[x] as u16 + last_line[i] as u16) >> 1) as u8
                                ),
                                None => byte.wrapping_add(last_line[i] >> 1),
                            });
                        } else {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add(line.data[x] >> 1),
                                None => *byte,
                            });
                        };
                    }
                    last_line = data.clone();
                    unfiltered.append(&mut data);
                }
                4 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    byte.wrapping_add(paeth_predictor(line.data[x],
                                                                      last_line[i],
                                                                      last_line[x]))
                                }
                                None => byte.wrapping_add(last_line[i]),
                            });
                        } else {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add(line.data[x]),
                                None => *byte,
                            });
                        };
                    }
                    last_line = data.clone();
                    unfiltered.append(&mut data);
                }
                _ => panic!("Unreachable"),
            }
        }
        unfiltered
    }
    pub fn filter_image(&self, filter: u8) -> Vec<u8> {
        let mut filtered = Vec::with_capacity(self.raw_data.len());
        // This avoids casting to and from floats, which is expensive
        let tmp = self.ihdr_data.bit_depth.as_u8() * self.channels_per_pixel();
        let bpp = (tmp + tmp % 8) >> 3;
        let mut last_line: Vec<u8> = vec![];
        // We could try a different filter method for each line
        // But that would be prohibitively slow and probably not provide much benefit
        // So we just use one filter method for the whole image
        for line in self.scan_lines() {
            if filter != 5 {
                filtered.push(filter);
            }
            match filter {
                0 => {
                    filtered.extend_from_slice(&line.data);
                }
                1 => {
                    for (i, byte) in line.data.iter().enumerate() {
                        filtered.push(match i.checked_sub(bpp as usize) {
                            Some(x) => byte.wrapping_sub(line.data[x]),
                            None => *byte,
                        });
                    }
                }
                2 => {
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            filtered.push(byte.wrapping_sub(last_line[i]));
                        } else {
                            filtered.push(*byte);
                        };
                    }
                }
                3 => {
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_sub(
                                    ((line.data[x] as u16 + last_line[i] as u16) >> 1) as u8
                                ),
                                None => byte.wrapping_sub(last_line[i] >> 1),
                            });
                        } else {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_sub(line.data[x] >> 1),
                                None => *byte,
                            });
                        };
                    }
                }
                4 => {
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    byte.wrapping_sub(paeth_predictor(line.data[x],
                                                                      last_line[i],
                                                                      last_line[x]))
                                }
                                None => byte.wrapping_sub(last_line[i]),
                            });
                        } else {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_sub(line.data[x]),
                                None => *byte,
                            });
                        };
                    }
                }
                5 => {
                    // Heuristically guess best filter per line
                    // Really simple algorithm, maybe we could replace this with something better
                    // libpng's heuristic no longer exists so I can't reference it
                    // Yes I know this code is ugly, but I didn't want to mess with mutable
                    // references from a HashMap that return options
                    // FIXME: Regardless of that, this is not very memory efficient
                    // Someone who's better at Rust can clean this up if they want
                    let line_0 = line.data.clone();
                    let mut line_1 = Vec::with_capacity(line.data.len());
                    let mut line_2 = Vec::with_capacity(line.data.len());
                    let mut line_3 = Vec::with_capacity(line.data.len());
                    let mut line_4 = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if !last_line.is_empty() {
                            match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    line_1.push(byte.wrapping_sub(line.data[x]));
                                    line_2.push(byte.wrapping_sub(last_line[i]));
                                    line_3.push(byte.wrapping_sub(
                                        ((line.data[x] as u16 + last_line[i] as u16) >> 1) as u8)
                                    );
                                    line_4.push(byte.wrapping_sub(paeth_predictor(line.data[x],
                                                                                  last_line[i],
                                                                                  last_line[x])));
                                }
                                None => {
                                    line_1.push(*byte);
                                    line_2.push(byte.wrapping_sub(last_line[i]));
                                    line_3.push(byte.wrapping_sub(last_line[i] >> 1));
                                    line_4.push(byte.wrapping_sub(last_line[i]));
                                }
                            }
                        } else {
                            match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    line_1.push(byte.wrapping_sub(line.data[x]));
                                    line_2.push(*byte);
                                    line_3.push(byte.wrapping_sub(line.data[x] >> 1));
                                    line_4.push(byte.wrapping_sub(line.data[x]));
                                }
                                None => {
                                    line_1.push(*byte);
                                    line_2.push(*byte);
                                    line_3.push(*byte);
                                    line_4.push(*byte);
                                }
                            }
                        };
                    }

                    // Count the number of unique bytes and take the lowest
                    let mut uniq_0 = line_0.clone();
                    uniq_0.sort();
                    uniq_0.dedup();
                    let mut uniq_1 = line_1.clone();
                    uniq_1.sort();
                    uniq_1.dedup();
                    let mut uniq_2 = line_2.clone();
                    uniq_2.sort();
                    uniq_2.dedup();
                    let mut uniq_3 = line_3.clone();
                    uniq_3.sort();
                    uniq_3.dedup();
                    let mut uniq_4 = line_4.clone();
                    uniq_4.sort();
                    uniq_4.dedup();
                    let mut best: (u8, &[u8], usize) = (0, &line_0, uniq_0.len());
                    if uniq_1.len() < best.2 {
                        best = (1, &line_1, uniq_1.len());
                    }
                    if uniq_2.len() < best.2 {
                        best = (2, &line_2, uniq_2.len());
                    }
                    if uniq_3.len() < best.2 {
                        best = (3, &line_3, uniq_3.len());
                    }
                    if uniq_4.len() < best.2 {
                        best = (4, &line_4, uniq_4.len());
                    }

                    filtered.push(best.0);
                    filtered.extend_from_slice(best.1);
                }
                _ => panic!("Unreachable"),
            }
            last_line = line.data.clone();
        }
        filtered
    }
    pub fn reduce_bit_depth(&mut self) -> bool {
        if self.ihdr_data.bit_depth != BitDepth::Sixteen {
            if self.ihdr_data.color_type == ColorType::Indexed ||
               self.ihdr_data.color_type == ColorType::Grayscale {
                return match reduce_bit_depth_8_or_less(self) {
                    Some(_) => true,
                    None => false,
                };
            }
            return false;
        }

        // It's difficult to estimate without knowing the number of ScanLines
        // So we overallocate to prioritize speed over memory efficiency
        let mut reduced = Vec::with_capacity(self.raw_data.len());

        for line in self.scan_lines() {
            reduced.push(line.filter);
            for (i, byte) in line.data.iter().enumerate() {
                if i % 2 == 0 {
                    // High byte
                    if *byte != 0 {
                        // Can't reduce, exit early
                        return false;
                    }
                } else {
                    // Low byte
                    reduced.push(*byte);
                }
            }
        }

        self.raw_data = reduced;
        true
    }
    pub fn reduce_palette(&mut self) -> bool {
        // TODO: Implement
        false
    }
    pub fn reduce_color_type(&mut self) -> bool {
        let mut changed = false;
        let mut should_reduce_bit_depth = false;

        // Go down one step at a time
        // Maybe not the most efficient, but it's safe
        if self.ihdr_data.color_type == ColorType::RGBA {
            // Do this first, it's more likely to exit early
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
                self.transparency_palette = Some(trans);
                self.ihdr_data.color_type = ColorType::Indexed;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::RGB {
            if let Some((data, palette)) = reduce_rgb_to_palette(self) {
                self.raw_data = data;
                self.palette = Some(palette);
                self.ihdr_data.color_type = ColorType::Indexed;
                changed = true;
                should_reduce_bit_depth = true;
            }
        }

        if self.ihdr_data.color_type == ColorType::Indexed && self.transparency_palette.is_none() {
            if let Some(data) = reduce_palette_to_grayscale(self) {
                self.raw_data = data;
                self.palette = None;
                self.ihdr_data.color_type = ColorType::Grayscale;
                changed = true;
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
    pub fn change_interlacing(&mut self, interlace: u8) -> bool {
        // TODO: Implement
        if interlace != self.ihdr_data.interlaced {
            return false;
        }

        false
    }
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
        // I hate having to iterate twice...
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            let bit_index = bit_depth - (i % bit_depth);
            if bit_index <= allowed_bits {
                reduced.push(bit);
            }
        }
    }

    Some((reduced.to_bytes(), allowed_bits as u8))
}

fn reduce_rgba_to_rgb(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 4 * byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp >= (bpp - byte_depth as usize) {
                if *byte != 255 {
                    return None;
                }
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
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut low_bytes = Vec::with_capacity(4);
        let mut high_bytes = Vec::with_capacity(4);
        let mut trans_bytes = Vec::with_capacity(byte_depth as usize);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp < (bpp - byte_depth as usize) {
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
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = HashMap::with_capacity(255);
    let byte_depth: usize = (png.ihdr_data.bit_depth.as_u8() >> 3) as usize;
    let bpp: usize = 4 * byte_depth;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if !palette.contains_key(&cur_pixel) {
                    let len = palette.len();
                    if len >= 255 {
                        return None;
                    }
                    palette.insert(cur_pixel.clone(), len as u8);
                }
                reduced.push(*palette.get(&cur_pixel).unwrap());
                cur_pixel.clear();
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * byte_depth * 3);
    let mut trans_palette = Vec::with_capacity(palette.len() * byte_depth);
    for color in palette.keys() {
        for (i, byte) in color.iter().enumerate() {
            if i < byte_depth * 3 {
                color_palette.push(*byte);
            } else {
                trans_palette.push(*byte);
            }
        }
    }

    Some((reduced, color_palette, trans_palette))
}

fn reduce_rgb_to_palette(png: &PngData) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let mut palette = HashMap::with_capacity(255);
    let byte_depth: usize = (png.ihdr_data.bit_depth.as_u8() >> 3) as usize;
    let bpp: usize = 3 * byte_depth;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        let mut cur_pixel = Vec::with_capacity(bpp);
        for (i, byte) in line.data.iter().enumerate() {
            cur_pixel.push(*byte);
            if i % bpp == bpp - 1 {
                if !palette.contains_key(&cur_pixel) {
                    let len = palette.len();
                    if len >= 255 {
                        return None;
                    }
                    palette.insert(cur_pixel.clone(), len as u8);
                }
                reduced.push(*palette.get(&cur_pixel).unwrap());
                cur_pixel.clear();
            }
        }
    }

    let mut color_palette = Vec::with_capacity(palette.len() * byte_depth * 3);
    for color in palette.keys() {
        color_palette.extend_from_slice(&color);
    }

    Some((reduced, color_palette))
}

fn reduce_palette_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = BitVec::with_capacity(png.raw_data.len() * 8);
    let mut cur_pixel = Vec::with_capacity(3);
    let palette = png.palette.clone().unwrap();
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

    let bit_depth: usize = png.ihdr_data.bit_depth.as_u8() as usize;
    for line in png.scan_lines() {
        reduced.extend(BitVec::from_bytes(&[line.filter]));
        let bit_vec = BitVec::from_bytes(&line.data);
        let mut cur_pixel = BitVec::with_capacity(bit_depth);
        for bit in bit_vec {
            cur_pixel.push(bit);
            if cur_pixel.len() == bit_depth {
                let palette_idx: usize = ((cur_pixel.to_bytes()[0] - 1) * 3) as usize;
                reduced.extend(BitVec::from_bytes(&[palette[palette_idx]]));
            }
        }
    }

    None
}

fn reduce_grayscale_alpha_to_grayscale(png: &PngData) -> Option<Vec<u8>> {
    let mut reduced = Vec::with_capacity(png.raw_data.len());
    let byte_depth = png.ihdr_data.bit_depth.as_u8() >> 3;
    let bpp: usize = 2 * byte_depth as usize;
    for line in png.scan_lines() {
        reduced.push(line.filter);
        for (i, byte) in line.data.iter().enumerate() {
            if i % bpp >= (bpp - byte_depth as usize) {
                if *byte != 255 {
                    return None;
                }
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
                     byte_offset: &mut usize)
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
    if crc32::checksum_ieee(header_bytes.as_ref()) != crc {
        return Err(format!("Corrupt data chunk found--CRC Mismatch in {}", header));
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
