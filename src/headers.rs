use byteorder::{BigEndian, ReadBytesExt};
use colors::{BitDepth, ColorType};
use crc::crc32;
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
    /// Some, with a list of 4-character chunk codes
    Some(Vec<String>),
    /// Headers that won't affect rendering (all but cHRM, gAMA, iCCP, sBIT, sRGB, bKGD, hIST, pHYs, sPLT)
    Safe,
    /// All non-critical headers
    All,
}

pub fn file_header_is_valid(bytes: &[u8]) -> bool {
    let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    bytes.iter().zip(expected_header.iter()).all(|x| x.0 == x.1)
}

pub fn parse_next_header(byte_data: &[u8],
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

pub fn parse_ihdr_header(byte_data: &[u8]) -> Result<IhdrData, String> {
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
