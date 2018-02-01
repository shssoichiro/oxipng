use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DisposalType {
    None = 0,
    Background = 1,
    Previous = 2,
}

impl From<u8> for DisposalType {
    fn from(val: u8) -> Self {
        match val {
            0 => DisposalType::None,
            1 => DisposalType::Background,
            2 => DisposalType::Previous,
            _ => panic!("Unrecognized disposal type"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BlendType {
    Source = 0,
    Over = 1,
}

impl From<u8> for BlendType {
    fn from(val: u8) -> Self {
        match val {
            0 => BlendType::Source,
            1 => BlendType::Over,
            _ => panic!("Unrecognized blend type"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApngFrame {
    pub sequence_number: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub delay_num: u16,
    pub delay_den: u16,
    pub dispose_op: DisposalType,
    pub blend_op: BlendType,
    /// The compressed, filtered data from the fdAT chunks
    pub frame_data: Vec<u8>,
    /// The uncompressed, optionally filtered data from the fdAT chunks
    pub raw_data: Vec<u8>,
}

impl<'a> From<&'a [u8]> for ApngFrame {
    /// Converts a fcTL header to an `ApngFrame`. Will panic if `data` is less than 26 bytes.
    fn from(data: &[u8]) -> Self {
        let mut cursor = Cursor::new(data);
        ApngFrame {
            sequence_number: cursor.read_u32::<BigEndian>().unwrap(),
            width: cursor.read_u32::<BigEndian>().unwrap(),
            height: cursor.read_u32::<BigEndian>().unwrap(),
            x_offset: cursor.read_u32::<BigEndian>().unwrap(),
            y_offset: cursor.read_u32::<BigEndian>().unwrap(),
            delay_num: cursor.read_u16::<BigEndian>().unwrap(),
            delay_den: cursor.read_u16::<BigEndian>().unwrap(),
            dispose_op: cursor.read_u8().unwrap().into(),
            blend_op: cursor.read_u8().unwrap().into(),
            frame_data: Vec::new(),
            raw_data: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ApngHeaders {
    pub frames: u32,
    pub plays: u32,
}
