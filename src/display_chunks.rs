/// List of chunks that affect image display and will be kept when using the `Safe` chunk strip option
pub const DISPLAY_CHUNKS: [[u8; 4]; 7] = [
    *b"cICP", *b"iCCP", *b"sRGB", *b"pHYs", *b"acTL", *b"fcTL", *b"fdAT",
];
