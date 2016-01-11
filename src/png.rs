use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator;
use std::path::Path;

#[derive(Debug)]
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

#[derive(Debug)]
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
}

struct ScanLines<'a> {
    png: &'a PngData,
    start: usize,
    end: usize,
    len: usize,
}

impl<'a> Iterator for ScanLines<'a> {
    type Item = ScanLine;
    fn next(&mut self) -> Option<Self::Item> {
        if self.end == self.len {
            None
        } else {
            let bits_per_line = self.png.ihdr_data.width as usize *
                                self.png.ihdr_data.bit_depth.as_u8() as usize *
                                self.png.bits_per_pixel_raw() as usize;
            // This avoids casting to and from floats, which is expensive
            let bytes_per_line = ((bits_per_line + bits_per_line % 8) >> 3) + 1;
            self.start = self.end;
            self.end = self.start + bytes_per_line;
            Some(ScanLine {
                filter: self.png.raw_data[self.start].clone(),
                data: self.png.raw_data[self.start + 1..self.end].to_owned(),
            })
        }
    }
}

#[derive(Debug)]
struct ScanLine {
    filter: u8,
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct PngData {
    pub idat_data: Vec<u8>,
    pub ihdr_data: IhdrData,
    pub raw_data: Vec<u8>,
    pub palette: Option<Vec<u8>>,
    pub aux_headers: HashMap<String, Vec<u8>>,
}

#[derive(Debug)]
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
        let ihdr_header = match parse_ihdr_header(aux_headers.get("IHDR").unwrap().as_ref()) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        let raw_data = match super::deflate::deflate::inflate(idat_headers.as_ref()) {
            Ok(x) => x,
            Err(x) => return Err(x),
        };
        let mut png_data = PngData {
            idat_data: idat_headers.clone(),
            ihdr_data: ihdr_header,
            raw_data: raw_data,
            palette: aux_headers.remove("PLTE"),
            aux_headers: aux_headers,
        };
        png_data.raw_data = png_data.unfilter_image();
        // Return the PngData
        Ok(png_data)
    }
    pub fn bits_per_pixel_raw(&self) -> u8 {
        match self.ihdr_data.color_type {
            ColorType::Grayscale => 1,
            ColorType::RGB => 3,
            ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGBA => 4,
        }
    }
    pub fn output(&self) -> Vec<u8> {
        // PNG header
        let mut output = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR
        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.width).ok();
        ihdr_data.write_u32::<BigEndian>(self.ihdr_data.height).ok();
        ihdr_data.write_u8(self.ihdr_data.bit_depth.as_u8()).ok();
        ihdr_data.write_u8(self.ihdr_data.color_type.png_header_code()).ok();
        ihdr_data.write_u8(0).ok();
        ihdr_data.write_u8(self.ihdr_data.filter).ok();
        ihdr_data.write_u8(self.ihdr_data.interlaced).ok();
        output.reserve(ihdr_data.len() + 12);
        output.write_u32::<BigEndian>(ihdr_data.len() as u32).ok();
        let mut type_head = "IHDR".as_bytes().to_owned();
        let crc = crc32::checksum_ieee(&ihdr_data);
        output.append(&mut type_head);
        output.append(&mut ihdr_data);
        output.write_u32::<BigEndian>(crc).ok();
        // Ancillary headers
        for (key, header) in &self.aux_headers {
            let mut header_data = header.clone();
            output.reserve(header_data.len() + 12);
            output.write_u32::<BigEndian>(header_data.len() as u32).ok();
            let mut type_head = key.as_bytes().to_owned();
            let crc = crc32::checksum_ieee(&header_data);
            output.append(&mut type_head);
            output.append(&mut header_data);
            output.write_u32::<BigEndian>(crc).ok();
        }
        // Palette
        if let Some(palette) = self.palette.clone() {
            let mut palette_data = palette.clone();
            output.reserve(palette_data.len() + 12);
            output.write_u32::<BigEndian>(palette_data.len() as u32).ok();
            let mut type_head = "PLTE".as_bytes().to_owned();
            let crc = crc32::checksum_ieee(&palette_data);
            output.append(&mut type_head);
            output.append(&mut palette_data);
            output.write_u32::<BigEndian>(crc).ok();
        }
        // IDAT data
        let mut idat_data = self.idat_data.clone();
        output.reserve(idat_data.len() + 12);
        output.write_u32::<BigEndian>(idat_data.len() as u32).ok();
        let mut type_head = "IDAT".as_bytes().to_owned();
        let crc = crc32::checksum_ieee(&idat_data);
        output.append(&mut type_head);
        output.append(&mut idat_data);
        output.write_u32::<BigEndian>(crc).ok();
        // Stream end
        let mut iend_data = vec![];
        output.reserve(iend_data.len() + 12);
        output.write_u32::<BigEndian>(iend_data.len() as u32).ok();
        let mut type_head = "IEND".as_bytes().to_owned();
        let crc = crc32::checksum_ieee(&iend_data);
        output.append(&mut type_head);
        output.append(&mut iend_data);
        output.write_u32::<BigEndian>(crc).ok();

        output
    }
    fn scan_lines(&self) -> ScanLines {
        ScanLines {
            png: &self,
            start: 0,
            end: 0,
            len: self.raw_data.len(),
        }
    }
    pub fn unfilter_image(&self) -> Vec<u8> {
        let mut unfiltered = Vec::with_capacity(self.raw_data.len());
        // This avoids casting to and from floats, which is expensive
        let tmp = self.ihdr_data.bit_depth.as_u8() * self.bits_per_pixel_raw();
        let bpp = (tmp + tmp % 8) >> 3;
        let mut last_line: Option<Vec<u8>> = None;
        for line in self.scan_lines() {
            unfiltered.push(line.filter);
            match line.filter {
                0 => {
                    let mut data = line.data.clone();
                    last_line = Some(data.clone());
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
                    last_line = Some(data.clone());
                    unfiltered.append(&mut data);
                }
                2 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if let Some(last) = last_line.clone() {
                            data.push(byte.wrapping_add(last[i]));
                        } else {
                            data.push(*byte);
                        };
                    }
                    last_line = Some(data.clone());
                    unfiltered.append(&mut data);
                }
                3 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if let Some(last) = last_line.clone() {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add((line.data[x] + last[i]) >> 1),
                                None => byte.wrapping_add(last[i] >> 1),
                            });
                        } else {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add(line.data[x] >> 1),
                                None => *byte,
                            });
                        };
                    }
                    last_line = Some(data.clone());
                    unfiltered.append(&mut data);
                }
                4 => {
                    let mut data = Vec::with_capacity(line.data.len());
                    for (i, byte) in line.data.iter().enumerate() {
                        if let Some(last) = last_line.clone() {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    byte.wrapping_add(paeth_predictor(line.data[x],
                                                                      last[i],
                                                                      last[x]))
                                }
                                None => byte.wrapping_add(last[i]),
                            });
                        } else {
                            data.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_add(line.data[x]),
                                None => *byte,
                            });
                        };
                    }
                    last_line = Some(data.clone());
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
        let tmp = self.ihdr_data.bit_depth.as_u8() * self.bits_per_pixel_raw();
        let bpp = (tmp + tmp % 8) >> 3;
        let mut last_line: Option<Vec<u8>> = None;
        // We could try a different filter method for each line
        // But that would be prohibitively slow and probably not provide much benefit
        // So we just use one filter method for the whole image
        for line in self.scan_lines() {
            let internal_filter = match filter {
                5 => {
                    // TODO: Heuristically guess best filter per line
                    0
                }
                x => x,
            };
            filtered.push(internal_filter);
            match internal_filter {
                0 => {
                    let mut data = line.data.clone();
                    filtered.append(&mut data);
                },
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
                        if let Some(last) = last_line.clone() {
                            filtered.push(byte.wrapping_sub(last[i]));
                        } else {
                            filtered.push(*byte);
                        };
                    }
                }
                3 => {
                    for (i, byte) in line.data.iter().enumerate() {
                        if let Some(last) = last_line.clone() {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_sub((line.data[x] + last[i]) >> 1),
                                None => byte.wrapping_sub(last[i] >> 1),
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
                        if let Some(last) = last_line.clone() {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => {
                                    byte.wrapping_sub(paeth_predictor(line.data[x],
                                                                      last[i],
                                                                      last[x]))
                                }
                                None => byte.wrapping_sub(last[i]),
                            });
                        } else {
                            filtered.push(match i.checked_sub(bpp as usize) {
                                Some(x) => byte.wrapping_sub(line.data[x]),
                                None => *byte,
                            });
                        };
                    }
                }
                _ => panic!("Unreachable"),
            }
            last_line = Some(line.data.clone());
        }
        filtered
    }
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
