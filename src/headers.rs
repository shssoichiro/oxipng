use indexmap::IndexSet;
use log::warn;
use rgb::{RGB16, RGBA8};

use crate::{
    colors::{BitDepth, ColorType},
    deflate::{crc32, inflate},
    error::PngError,
    interlace::Interlacing,
    AtomicMin, Deflaters, PngResult,
};

#[derive(Debug, Clone)]
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
    /// The interlacing mode of the image
    pub interlaced: Interlacing,
}

impl IhdrData {
    /// Bits per pixel
    #[must_use]
    #[inline]
    pub fn bpp(&self) -> usize {
        self.bit_depth as usize * self.color_type.channels_per_pixel() as usize
    }

    /// Byte length of IDAT that is correct for this IHDR
    #[must_use]
    pub fn raw_data_size(&self) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        let bpp = self.bpp();

        fn bitmap_size(bpp: usize, w: usize, h: usize) -> usize {
            ((w * bpp + 7) / 8) * h
        }

        if self.interlaced == Interlacing::None {
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

#[derive(Debug, Clone)]
pub struct Chunk {
    pub name: [u8; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// Options to use when stripping chunks
pub enum StripChunks {
    /// None
    None,
    /// Remove specific chunks
    Strip(IndexSet<[u8; 4]>),
    /// Remove all chunks that won't affect image display
    Safe,
    /// Remove all non-critical chunks except these
    Keep(IndexSet<[u8; 4]>),
    /// All non-critical chunks
    All,
}

impl StripChunks {
    /// List of chunks that affect image display and will be kept when using the `Safe` option
    pub const DISPLAY: [[u8; 4]; 7] = [
        *b"cICP", *b"iCCP", *b"sRGB", *b"pHYs", *b"acTL", *b"fcTL", *b"fdAT",
    ];

    pub(crate) fn keep(&self, name: &[u8; 4]) -> bool {
        match &self {
            StripChunks::None => true,
            StripChunks::Keep(names) => names.contains(name),
            StripChunks::Strip(names) => !names.contains(name),
            StripChunks::Safe => Self::DISPLAY.contains(name),
            StripChunks::All => false,
        }
    }
}

#[inline]
pub fn file_header_is_valid(bytes: &[u8]) -> bool {
    let expected_header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    *bytes == expected_header
}

#[derive(Debug, Clone, Copy)]
pub struct RawChunk<'a> {
    pub name: [u8; 4],
    pub data: &'a [u8],
}

pub fn parse_next_chunk<'a>(
    byte_data: &'a [u8],
    byte_offset: &mut usize,
    fix_errors: bool,
) -> PngResult<Option<RawChunk<'a>>> {
    let length = read_be_u32(
        byte_data
            .get(*byte_offset..*byte_offset + 4)
            .ok_or(PngError::TruncatedData)?,
    );
    *byte_offset += 4;

    let chunk_start = *byte_offset;
    let chunk_name = byte_data
        .get(chunk_start..chunk_start + 4)
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
    let crc = read_be_u32(
        byte_data
            .get(*byte_offset..*byte_offset + 4)
            .ok_or(PngError::TruncatedData)?,
    );
    *byte_offset += 4;

    let chunk_bytes = byte_data
        .get(chunk_start..chunk_start + 4 + length as usize)
        .ok_or(PngError::TruncatedData)?;
    if !fix_errors && crc32(chunk_bytes) != crc {
        return Err(PngError::new(&format!(
            "CRC Mismatch in {} chunk; May be recoverable by using --fix",
            String::from_utf8_lossy(chunk_name)
        )));
    }

    let name: [u8; 4] = chunk_name.try_into().unwrap();
    Ok(Some(RawChunk { name, data }))
}

pub fn parse_ihdr_chunk(
    byte_data: &[u8],
    palette_data: Option<Vec<u8>>,
    trns_data: Option<Vec<u8>>,
) -> PngResult<IhdrData> {
    // This eliminates bounds checks for the rest of the function
    let interlaced = byte_data.get(12).copied().ok_or(PngError::TruncatedData)?;
    Ok(IhdrData {
        color_type: match byte_data[9] {
            0 => ColorType::Grayscale {
                transparent_shade: trns_data
                    .filter(|t| t.len() >= 2)
                    .map(|t| u16::from_be_bytes([t[0], t[1]])),
            },
            2 => ColorType::RGB {
                transparent_color: trns_data.filter(|t| t.len() >= 6).map(|t| RGB16 {
                    r: u16::from_be_bytes([t[0], t[1]]),
                    g: u16::from_be_bytes([t[2], t[3]]),
                    b: u16::from_be_bytes([t[4], t[5]]),
                }),
            },
            3 => ColorType::Indexed {
                palette: palette_to_rgba(palette_data, trns_data).unwrap_or_default(),
            },
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::RGBA,
            _ => return Err(PngError::new("Unexpected color type in header")),
        },
        bit_depth: byte_data[8].try_into()?,
        width: read_be_u32(&byte_data[0..4]),
        height: read_be_u32(&byte_data[4..8]),
        interlaced: interlaced.try_into()?,
    })
}

/// Construct an RGBA palette from the raw palette and transparency data
fn palette_to_rgba(
    palette_data: Option<Vec<u8>>,
    trns_data: Option<Vec<u8>>,
) -> Result<Vec<RGBA8>, PngError> {
    let palette_data = palette_data.ok_or_else(|| PngError::new("no palette in indexed image"))?;
    let mut palette: Vec<_> = palette_data
        .chunks(3)
        .map(|color| RGBA8::new(color[0], color[1], color[2], 255))
        .collect();

    if let Some(trns_data) = trns_data {
        for (color, trns) in palette.iter_mut().zip(trns_data) {
            color.a = trns;
        }
    }
    Ok(palette)
}

#[inline]
fn read_be_u32(bytes: &[u8]) -> u32 {
    u32::from_be_bytes(bytes.try_into().unwrap())
}

/// Extract and decompress the ICC profile from an iCCP chunk
pub fn extract_icc(iccp: &Chunk) -> Option<Vec<u8>> {
    // Skip (useless) profile name
    let mut data = iccp.data.as_slice();
    loop {
        let (&n, rest) = data.split_first()?;
        data = rest;
        if n == 0 {
            break;
        }
    }

    let (&compression_method, compressed_data) = data.split_first()?;
    if compression_method != 0 {
        return None; // The profile is supposed to be compressed (method 0)
    }
    // The decompressed size is unknown so we have to guess the required buffer size
    let max_size = compressed_data.len() * 2 + 1000;
    match inflate(compressed_data, max_size) {
        Ok(icc) => Some(icc),
        Err(e) => {
            // Log the error so we can know if the buffer size needs to be adjusted
            warn!("Failed to decompress icc: {}", e);
            None
        }
    }
}

/// Construct an iCCP chunk by compressing the ICC profile
pub fn construct_iccp(icc: &[u8], deflater: Deflaters) -> PngResult<Chunk> {
    let mut compressed = deflater.deflate(icc, &AtomicMin::new(None))?;
    let mut data = Vec::with_capacity(compressed.len() + 5);
    data.extend(b"icc"); // Profile name - generally unused, can be anything
    data.extend([0, 0]); // Null separator, zlib compression method
    data.append(&mut compressed);
    Ok(Chunk {
        name: *b"iCCP",
        data,
    })
}

/// If the profile is sRGB, extracts the rendering intent value from it
pub fn srgb_rendering_intent(icc_data: &[u8]) -> Option<u8> {
    let rendering_intent = *icc_data.get(67)?;

    // The known profiles are the same as in libpng's `png_sRGB_checks`.
    // The Profile ID header of ICC has a fixed layout,
    // and is supposed to contain MD5 of profile data at this offset
    match icc_data.get(84..100)? {
        b"\x29\xf8\x3d\xde\xaf\xf2\x55\xae\x78\x42\xfa\xe4\xca\x83\x39\x0d"
        | b"\xc9\x5b\xd6\x37\xe9\x5d\x8a\x3b\x0d\xf3\x8f\x99\xc1\x32\x03\x89"
        | b"\xfc\x66\x33\x78\x37\xe2\x88\x6b\xfd\x72\xe9\x83\x82\x28\xf1\xb8"
        | b"\x34\x56\x2a\xbf\x99\x4c\xcd\x06\x6d\x2c\x57\x21\xd0\xd6\x8c\x5d" => {
            Some(rendering_intent)
        }
        b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" => {
            // Known-bad profiles are identified by their CRC
            match (crc32(icc_data), icc_data.len()) {
                (0x5d51_29ce, 3024) | (0x182e_a552, 3144) | (0xf29e_526d, 3144) => {
                    Some(rendering_intent)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
