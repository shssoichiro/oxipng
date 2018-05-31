use error::PngError;
use miniz_oxide;
use std::cmp::max;
use zopfli;

#[doc(hidden)]
pub mod miniz_stream;

/// Decompress a data stream using the DEFLATE algorithm
pub fn inflate(data: &[u8]) -> Result<Vec<u8>, PngError> {
    miniz_oxide::inflate::decompress_to_vec_zlib(data)
        .map_err(|e| PngError::new(&format!("Error on decompress: {:?}", e)))
}

/// Compress a data stream using the DEFLATE algorithm
#[cfg(not(feature = "cfzlib"))]
pub fn deflate(data: &[u8], zc: u8, zs: u8, zw: u8) -> Vec<u8> {
    miniz_stream::compress_to_vec_oxipng(data, zc, zw.into(), zs.into())
}

#[cfg(feature = "cfzlib")]
pub fn deflate(data: &[u8], level: u8, strategy: u8, window_bits: u8) -> Vec<u8> {
    use std::mem;
    use cloudflare_zlib_sys::*;

    assert!(data.len() < u32::max_value() as usize);
    unsafe {
        let mut stream = mem::zeroed();
        assert_eq!(Z_OK, deflateInit2(
                    &mut stream,
                    level.into(),
                    Z_DEFLATED,
                    window_bits.into(),
                    MAX_MEM_LEVEL,
                    strategy.into()));

        let max_size = deflateBound(&mut stream, data.len() as uLong) as usize;
        // it's important to have the capacity pre-allocated,
        // as unsafe set_len is called later
        let mut out = Vec::with_capacity(max_size);

        stream.next_in = data.as_ptr() as *mut _;
        stream.total_in = data.len() as uLong;
        stream.avail_in = data.len() as uInt;
        stream.next_out = out.as_mut_ptr();
        stream.avail_out = out.capacity() as uInt;
        assert_eq!(Z_STREAM_END, deflate(&mut stream, Z_FINISH));
        assert_eq!(Z_OK, deflateEnd(&mut stream));
        debug_assert!(stream.total_out as usize <= out.capacity());
        out.set_len(stream.total_out as usize);
        return out;
    }
}

pub fn zopfli_deflate(data: &[u8]) -> Result<Vec<u8>, PngError> {
    let mut output = Vec::with_capacity(max(1024, data.len() / 20));
    let options = zopfli::Options::default();
    match zopfli::compress(&options, &zopfli::Format::Zlib, data, &mut output) {
        Ok(_) => (),
        Err(_) => return Err(PngError::new("Failed to compress in zopfli")),
    };
    output.shrink_to_fit();
    Ok(output)
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// DEFLATE algorithms supported by oxipng
pub enum Deflaters {
    /// Use the Zlib/Miniz DEFLATE implementation
    Zlib,
    /// Use the better but slower Zopfli implementation
    Zopfli,
}
