use miniz_oxide::deflate::core::*;

pub fn compress_to_vec_oxipng(input: &[u8], level: u8, window_bits: i32, strategy: i32) -> Vec<u8> {
    // The comp flags function sets the zlib flag if the window_bits parameter is > 0.
    let flags = create_comp_flags_from_zip_params(level.into(), window_bits, strategy);
    let mut compressor = CompressorOxide::new(flags);
    let mut output = Vec::with_capacity(input.len() / 2);
    // # Unsafe
    // We trust compress to not read the uninitialized bytes.
    unsafe {
        let cap = output.capacity();
        output.set_len(cap);
    }
    let mut in_pos = 0;
    let mut out_pos = 0;
    loop {
        let (status, bytes_in, bytes_out) = compress(
            &mut compressor,
            &input[in_pos..],
            &mut output[out_pos..],
            TDEFLFlush::Finish,
        );

        out_pos += bytes_out;
        in_pos += bytes_in;

        match status {
            TDEFLStatus::Done => {
                output.truncate(out_pos);
                break;
            }
            TDEFLStatus::Okay => {
                // We need more space, so extend the vector.
                if output.len().saturating_sub(out_pos) < 30 {
                    let current_len = output.len();
                    output.reserve(current_len);

                    // # Unsafe
                    // We trust compress to not read the uninitialized bytes.
                    unsafe {
                        let cap = output.capacity();
                        output.set_len(cap);
                    }
                }
            }
            // Not supposed to happen unless there is a bug.
            _ => panic!("Bug! Unexpectedly failed to compress!"),
        }
    }

    output
}
