use crate::colors::ColorType;
use crate::deflate;
use crate::error::PngError;
use crate::filters::*;
use crate::headers::*;
use crate::interlace::{deinterlace_image, interlace_image};
use indexmap::IndexMap;
use rgb::ComponentSlice;
use rgb::RGBA8;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::iter::Iterator;
use std::path::Path;
use std::sync::Arc;

/// Must use normal (lazy) compression, as faster ones (greedy) are not representative
pub(crate) const STD_COMPRESSION: u8 = 5;
pub(crate) const STD_FILTERS: [u8; 2] = [0, 5];

pub(crate) mod scan_lines;

use self::scan_lines::{ScanLines, ScanLinesMut};

#[derive(Debug, Clone)]
pub struct PngImage {
    /// The headers stored in the IHDR chunk
    pub ihdr: IhdrData,
    /// The uncompressed, optionally filtered data from the IDAT chunk
    pub data: Vec<u8>,
    /// The palette containing colors used in an Indexed image
    /// Contains 3 bytes per color (R+G+B), up to 768
    pub palette: Option<Vec<RGBA8>>,
    /// The pixel value that should be rendered as transparent
    pub transparency_pixel: Option<Vec<u8>>,
    /// All non-critical headers from the PNG are stored here
    pub aux_headers: IndexMap<[u8; 4], Vec<u8>>,
}

/// Contains all data relevant to a PNG image
#[derive(Debug, Clone)]
pub struct PngData {
    /// Uncompressed image data
    pub raw: Arc<PngImage>,
    /// The filtered and compressed data of the IDAT chunk
    pub idat_data: Vec<u8>,
}

type PaletteWithTrns = (Option<Vec<RGBA8>>, Option<Vec<u8>>);

impl PngData {
    /// Create a new `PngData` struct by opening a file
    #[inline]
    pub fn new(filepath: &Path, fix_errors: bool) -> Result<Self, PngError> {
        let byte_data = Self::read_file(filepath)?;

        Self::from_slice(&byte_data, fix_errors)
    }

    pub fn read_file(filepath: &Path) -> Result<Vec<u8>, PngError> {
        let file = match File::open(filepath) {
            Ok(f) => f,
            Err(_) => return Err(PngError::new("Failed to open file for reading")),
        };
        let file_len = file.metadata().map(|m| m.len() as usize).unwrap_or(0);
        let mut reader = BufReader::new(file);
        // Check file for PNG header
        let mut header = [0; 8];
        if reader.read_exact(&mut header).is_err() {
            return Err(PngError::new("Not a PNG file: too small"));
        }
        if !file_header_is_valid(&header) {
            return Err(PngError::new("Invalid PNG header detected"));
        }
        // Read raw png data into memory
        let mut byte_data: Vec<u8> = Vec::with_capacity(file_len);
        byte_data.extend_from_slice(&header);
        match reader.read_to_end(&mut byte_data) {
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
        let mut aux_headers: IndexMap<[u8; 4], Vec<u8>> = IndexMap::new();
        let mut idat_headers: Vec<u8> = Vec::new();
        while let Some(header) = parse_next_header(byte_data, &mut byte_offset, fix_errors)? {
            match &header.name {
                b"IDAT" => idat_headers.extend_from_slice(header.data),
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
        let raw_data = deflate::inflate(idat_headers.as_ref(), ihdr_header.raw_data_size())?;

        // Reject files with incorrect width/height or truncated data
        if raw_data.len() != ihdr_header.raw_data_size() {
            return Err(PngError::TruncatedData);
        }

        let (palette, transparency_pixel) = Self::palette_to_rgba(
            ihdr_header.color_type,
            aux_headers.remove(b"PLTE"),
            aux_headers.remove(b"tRNS"),
        )?;

        let mut raw = PngImage {
            ihdr: ihdr_header,
            data: raw_data,
            palette,
            transparency_pixel,
            aux_headers,
        };
        raw.data = raw.unfilter_image()?;
        // Return the PngData
        Ok(Self {
            idat_data: idat_headers,
            raw: Arc::new(raw),
        })
    }

    /// Handle transparency header
    fn palette_to_rgba(
        color_type: ColorType,
        palette_data: Option<Vec<u8>>,
        trns_data: Option<Vec<u8>>,
    ) -> Result<PaletteWithTrns, PngError> {
        if color_type == ColorType::Indexed {
            let palette_data =
                palette_data.ok_or_else(|| PngError::new("no palette in indexed image"))?;
            let mut palette: Vec<_> = palette_data
                .chunks(3)
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

    /// Format the `PngData` struct into a valid PNG bytestream
    pub fn output(&self) -> Vec<u8> {
        // PNG header
        let mut output = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        // IHDR
        let mut ihdr_data = Vec::with_capacity(13);
        ihdr_data.write_all(&self.raw.ihdr.width.to_be_bytes()).ok();
        ihdr_data
            .write_all(&self.raw.ihdr.height.to_be_bytes())
            .ok();
        ihdr_data.write_all(&[self.raw.ihdr.bit_depth.as_u8()]).ok();
        ihdr_data
            .write_all(&[self.raw.ihdr.color_type.png_header_code()])
            .ok();
        ihdr_data.write_all(&[0]).ok(); // Compression -- deflate
        ihdr_data.write_all(&[0]).ok(); // Filter method -- 5-way adaptive filtering
        ihdr_data.write_all(&[self.raw.ihdr.interlaced]).ok();
        write_png_block(b"IHDR", &ihdr_data, &mut output);
        // Ancillary headers
        for (key, header) in self
            .raw
            .aux_headers
            .iter()
            .filter(|&(key, _)| !(key == b"bKGD" || key == b"hIST" || key == b"tRNS"))
        {
            write_png_block(key, header, &mut output);
        }
        // Palette
        if let Some(ref palette) = self.raw.palette {
            let mut palette_data = Vec::with_capacity(palette.len() * 3);
            let max_palette_size = 1 << (self.raw.ihdr.bit_depth.as_u8() as usize);
            for px in palette.iter().take(max_palette_size) {
                palette_data.extend_from_slice(px.rgb().as_slice());
            }
            write_png_block(b"PLTE", &palette_data, &mut output);
            let num_transparent =
                palette
                    .iter()
                    .take(max_palette_size)
                    .enumerate()
                    .fold(
                        0,
                        |prev, (index, px)| {
                            if px.a == 255 {
                                prev
                            } else {
                                index + 1
                            }
                        },
                    );
            if num_transparent > 0 {
                let trns_data: Vec<_> = palette[0..num_transparent].iter().map(|px| px.a).collect();
                write_png_block(b"tRNS", &trns_data, &mut output);
            }
        } else if let Some(ref transparency_pixel) = self.raw.transparency_pixel {
            // Transparency pixel
            write_png_block(b"tRNS", transparency_pixel, &mut output);
        }
        // Special ancillary headers that need to come after PLTE but before IDAT
        for (key, header) in self
            .raw
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
}

impl PngImage {
    /// Convert the image to the specified interlacing type
    /// Returns true if the interlacing was changed, false otherwise
    /// The `interlace` parameter specifies the *new* interlacing mode
    /// Assumes that the data has already been de-filtered
    #[inline]
    #[must_use]
    pub fn change_interlacing(&self, interlace: u8) -> Option<PngImage> {
        if interlace == self.ihdr.interlaced {
            return None;
        }

        Some(if interlace == 1 {
            // Convert progressive to interlaced data
            interlace_image(self)
        } else {
            // Convert interlaced to progressive data
            deinterlace_image(self)
        })
    }

    /// Return the number of channels in the image, based on color type
    #[inline]
    pub fn channels_per_pixel(&self) -> u8 {
        self.ihdr.color_type.channels_per_pixel()
    }

    /// Return an iterator over the scanlines of the image
    #[inline]
    pub fn scan_lines(&self) -> ScanLines<'_> {
        ScanLines::new(self)
    }

    /// Return an iterator over the scanlines of the image
    #[inline]
    pub fn scan_lines_mut(&mut self) -> ScanLinesMut<'_> {
        ScanLinesMut::new(self)
    }

    /// Reverse all filters applied on the image, returning an unfiltered IDAT bytestream
    fn unfilter_image(&self) -> Result<Vec<u8>, PngError> {
        let mut unfiltered = Vec::with_capacity(self.data.len());
        let bpp = ((self.ihdr.bit_depth.as_u8() * self.channels_per_pixel() + 7) / 8) as usize;
        let mut last_line: Vec<u8> = Vec::new();
        let mut last_pass = None;
        let mut unfiltered_buf = Vec::new();
        for line in self.scan_lines() {
            if last_pass != line.pass {
                last_line.clear();
                last_pass = line.pass;
            }
            last_line.resize(line.data.len(), 0);
            unfilter_line(line.filter, bpp, line.data, &last_line, &mut unfiltered_buf)?;
            unfiltered.push(0);
            unfiltered.extend_from_slice(&unfiltered_buf);
            std::mem::swap(&mut last_line, &mut unfiltered_buf);
            unfiltered_buf.clear();
        }
        Ok(unfiltered)
    }

    /// Apply the specified filter type to all rows in the image
    /// 0: None
    /// 1: Sub
    /// 2: Up
    /// 3: Average
    /// 4: Paeth
    /// 5: All (heuristically pick the best filter for each line)
    pub fn filter_image(&self, filter: u8) -> Vec<u8> {
        let mut filtered = Vec::with_capacity(self.data.len());
        let bpp = ((self.ihdr.bit_depth.as_u8() * self.channels_per_pixel() + 7) / 8) as usize;
        let mut last_line: &[u8] = &[];
        let mut last_pass: Option<u8> = None;
        let mut f_buf = Vec::new();
        for line in self.scan_lines() {
            f_buf.clear();
            if last_pass != line.pass {
                last_line = &[];
            }
            match filter {
                0 | 1 | 2 | 3 | 4 => {
                    let filter = if last_pass == line.pass || filter <= 1 {
                        filter
                    } else {
                        0
                    };
                    filtered.push(filter);
                    filter_line(filter, bpp, line.data, last_line, &mut f_buf);
                    filtered.extend_from_slice(&f_buf);
                }
                5 => {
                    // Heuristically guess best filter per line
                    // Uses MSAD algorithm mentioned in libpng reference docs
                    // http://www.libpng.org/pub/png/book/chapter09.html
                    let mut best_filter = 0;
                    let mut best_line = Vec::new();
                    let mut best_size = u64::MAX;

                    // Avoid vertical filtering on first line of each interlacing pass
                    for filter in if last_pass == line.pass { 0..5 } else { 0..2 } {
                        filter_line(filter, bpp, line.data, last_line, &mut f_buf);
                        let size = f_buf.iter().fold(0_u64, |acc, &x| {
                            let signed = x as i8;
                            acc + i16::from(signed).unsigned_abs() as u64
                        });
                        if size < best_size {
                            best_size = size;
                            best_filter = filter;
                            std::mem::swap(&mut best_line, &mut f_buf);
                        }
                        f_buf.clear() //discard buffer, and start again
                    }
                    filtered.push(best_filter);
                    filtered.extend_from_slice(&best_line);
                }
                _ => unreachable!(),
            }
            last_line = line.data;
            last_pass = line.pass;
        }
        filtered
    }
}
fn write_png_block(key: &[u8], header: &[u8], output: &mut Vec<u8>) {
    let mut header_data = Vec::with_capacity(header.len() + 4);
    header_data.extend_from_slice(key);
    header_data.extend_from_slice(header);
    output.reserve(header_data.len() + 8);
    output.extend_from_slice(&(header_data.len() as u32 - 4).to_be_bytes());
    let crc = deflate::crc32(&header_data);
    output.append(&mut header_data);
    output.extend_from_slice(&crc.to_be_bytes());
}
