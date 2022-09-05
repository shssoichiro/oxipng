use crate::colors::{BitDepth, ColorType};
use crate::error::PngError;
use crate::PngResult;
use crc::{Crc, CRC_32_ISO_HDLC};
use indexmap::IndexSet;
use std::io;
use std::io::{Cursor, Read};

#[derive(Debug, Clone, Copy)]
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

impl IhdrData {
    /// Bits per pixel
    #[must_use]
    #[inline]
    pub fn bpp(&self) -> u8 {
        self.bit_depth.as_u8() * self.color_type.channels_per_pixel()
    }

    /// Byte length of IDAT that is correct for this IHDR
    #[must_use]
    pub fn raw_data_size(&self) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        let bpp = self.bpp();

        fn bitmap_size(bpp: u8, w: usize, h: usize) -> usize {
            (((w / 8) * bpp as usize) + ((w & 7) * bpp as usize + 7) / 8) * h
        }

        if self.interlaced == 0 {
            bitmap_size(bpp, w, h) + h
        } else {
            let mut size = bitmap_size(bpp, (w + 7) >> 3, (h + 7) >> 3) + ((h + 7) >> 3);
            if w > 4 {
                size += bitmap_size(bpp, (w + 3) >> 3, (h + 7) >> 3) + ((h + 7) >> 3);
            }
            size += bitmap_size(bpp, (w + 3) >> 2, (h + 3) >> 3) + ((h + 3) >> 3);
            if w > 2 {
                size += bitmap_size(bpp, (w + 1) >> 2, (h + 3) >> 2) + ((h + 3) >> 2);
            }
            size += bitmap_size(bpp, (w + 1) >> 1, (h + 1) >> 2) + ((h + 1) >> 2);
            if w > 1 {
                size += bitmap_size(bpp, w >> 1, (h + 1) >> 1) + ((h + 1) >> 1);
            }
            size + bitmap_size(bpp, w, h >> 1) + (h >> 1)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Options to use for performing operations on headers (such as stripping)
pub enum Headers {
    /// None
    None,
    /// Remove specific chunks
    Strip(Vec<String>),
    /// Headers that won't affect rendering (all but cHRM, gAMA, iCCP, sBIT, sRGB, bKGD, hIST, pHYs, sPLT)
    Safe,
    /// Remove all non-critical chunks except these
    Keep(IndexSet<String>),
    /// All non-critical headers
    All,
}

#[inline]
pub fn file_header_is_valid(bytes: &[u8]) -> bool {
    let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    *bytes == expected_header
}

#[derive(Debug, Clone, Copy)]
pub struct RawHeader<'a> {
    pub name: [u8; 4],
    pub data: &'a [u8],
}

pub fn parse_next_header<'a>(
    byte_data: &'a [u8],
    byte_offset: &mut usize,
    fix_errors: bool,
) -> PngResult<Option<RawHeader<'a>>> {
    let mut rdr = Cursor::new(
        byte_data
            .get(*byte_offset..*byte_offset + 4)
            .ok_or(PngError::TruncatedData)?,
    );
    let length = read_be_u32(&mut rdr).unwrap();
    *byte_offset += 4;

    let header_start = *byte_offset;
    let chunk_name = byte_data
        .get(header_start..header_start + 4)
        .ok_or(PngError::TruncatedData)?;
    if chunk_name == b"IEND" {
        // End of data
        return Ok(None);
    }
    *byte_offset += 4;

    let data = byte_data
        .get(*byte_offset..*byte_offset + length as usize)
        .ok_or(PngError::TruncatedData)?;
    *byte_offset += length as usize;
    let mut rdr = Cursor::new(
        byte_data
            .get(*byte_offset..*byte_offset + 4)
            .ok_or(PngError::TruncatedData)?,
    );
    let crc = read_be_u32(&mut rdr).unwrap();
    *byte_offset += 4;

    let header_bytes = byte_data
        .get(header_start..header_start + 4 + length as usize)
        .ok_or(PngError::TruncatedData)?;
    if !fix_errors && Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(header_bytes) != crc {
        return Err(PngError::new(&format!(
            "CRC Mismatch in {} header; May be recoverable by using --fix",
            String::from_utf8_lossy(chunk_name)
        )));
    }

    let mut name = [0_u8; 4];
    name.copy_from_slice(chunk_name);
    Ok(Some(RawHeader { name, data }))
}

pub fn parse_ihdr_header(byte_data: &[u8]) -> PngResult<IhdrData> {
    // This eliminates bounds checks for the rest of the function
    let interlaced = byte_data.get(12).copied().ok_or(PngError::TruncatedData)?;
    let mut rdr = Cursor::new(&byte_data[0..8]);
    Ok(IhdrData {
        color_type: match byte_data[9] {
            0 => ColorType::Grayscale,
            2 => ColorType::RGB,
            3 => ColorType::Indexed,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
            _ => return Err(PngError::new("Unexpected color type in header")),
        },
        bit_depth: match byte_data[8] {
            1 => BitDepth::One,
            2 => BitDepth::Two,
            4 => BitDepth::Four,
            8 => BitDepth::Eight,
            16 => BitDepth::Sixteen,
            _ => return Err(PngError::new("Unexpected bit depth in header")),
        },
        width: read_be_u32(&mut rdr).map_err(|_| PngError::TruncatedData)?,
        height: read_be_u32(&mut rdr).map_err(|_| PngError::TruncatedData)?,
        compression: byte_data[10],
        filter: byte_data[11],
        interlaced,
    })
}

#[inline]
fn read_be_u32<T: AsRef<[u8]>>(rdr: &mut Cursor<T>) -> Result<u32, io::Error> {
    let mut int_buf = [0; 4];
    rdr.read_exact(&mut int_buf)?;
    Ok(u32::from_be_bytes(int_buf))
}
