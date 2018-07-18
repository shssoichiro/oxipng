use std::collections::HashSet;
use byteorder::{BigEndian, ReadBytesExt};
use colors::{BitDepth, ColorType};
use crc::crc32;
use error::PngError;
use std::io::Cursor;

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

#[derive(Debug, PartialEq, Clone)]
/// Options to use for performing operations on headers (such as stripping)
pub enum Headers {
    /// None
    None,
    /// Remove specific chunks
    Strip(Vec<String>),
    /// Headers that won't affect rendering (all but cHRM, gAMA, iCCP, sBIT, sRGB, bKGD, hIST, pHYs, sPLT)
    Safe,
    /// Remove all non-critical chunks except these
    Keep(HashSet<String>),
    /// All non-critical headers
    All,
}

#[inline]
pub fn file_header_is_valid(bytes: &[u8]) -> bool {
    let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    *bytes == expected_header
}

pub fn parse_next_header<'a>(
    byte_data: &'a [u8],
    byte_offset: &mut usize,
    fix_errors: bool,
    ) -> Result<Option<(&'a [u8], &'a [u8])>, PngError> {
    let mut rdr = Cursor::new(byte_data.get(*byte_offset..*byte_offset + 4).ok_or(PngError::TruncatedData)?);
    let length = rdr.read_u32::<BigEndian>().unwrap();
    *byte_offset += 4;

    let header_start = *byte_offset;
    let chunk_name = byte_data.get(header_start..header_start + 4).ok_or(PngError::TruncatedData)?;
    if chunk_name == b"IEND" {
        // End of data
        return Ok(None);
    }
    *byte_offset += 4;

    let data = byte_data.get(*byte_offset..*byte_offset + length as usize).ok_or(PngError::TruncatedData)?;
    *byte_offset += length as usize;
    let mut rdr = Cursor::new(byte_data.get(*byte_offset..*byte_offset + 4).ok_or(PngError::TruncatedData)?);
    let crc = rdr.read_u32::<BigEndian>().unwrap();
    *byte_offset += 4;

    let header_bytes = byte_data.get(header_start..header_start + 4 + length as usize).ok_or(PngError::TruncatedData)?;
    if !fix_errors && crc32::checksum_ieee(header_bytes) != crc {
        return Err(PngError::new(&format!(
            "CRC Mismatch in {} header; May be recoverable by using --fix",
            String::from_utf8_lossy(chunk_name)
        )));
    }

    Ok(Some((chunk_name, data)))
}

pub fn parse_ihdr_header(byte_data: &[u8]) -> Result<IhdrData, PngError> {
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
        width: rdr.read_u32::<BigEndian>().unwrap(),
        height: rdr.read_u32::<BigEndian>().unwrap(),
        compression: byte_data[10],
        filter: byte_data[11],
        interlaced: byte_data[12],
    })
}
