use std::{fmt::Display, mem::transmute};

use crate::error::PngError;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum RowFilter {
    // Standard filter types
    None,
    Sub,
    Up,
    Average,
    Paeth,
    // Heuristic strategies
    MinSum,
}

impl TryFrom<u8> for RowFilter {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::LAST {
            return Err(());
        }
        unsafe { transmute(value as i8) }
    }
}

impl Display for RowFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:8}",
            match *self {
                Self::None => "None",
                Self::Sub => "Sub",
                Self::Up => "Up",
                Self::Average => "Average",
                Self::Paeth => "Paeth",
                Self::MinSum => "MinSum",
            }
        )
    }
}

impl RowFilter {
    pub const LAST: u8 = Self::MinSum as u8;
    pub const STANDARD: [Self; 5] = [Self::None, Self::Sub, Self::Up, Self::Average, Self::Paeth];

    pub fn filter_line(self, bpp: usize, data: &[u8], last_line: &[u8], buf: &mut Vec<u8>) {
        assert!(data.len() >= bpp);
        assert!(last_line.is_empty() || data.len() == last_line.len());
        buf.reserve(data.len());
        match self {
            Self::None => {
                buf.extend_from_slice(data);
            }
            Self::Sub => {
                buf.extend_from_slice(&data[0..bpp]);
                buf.extend(
                    data.iter()
                        .skip(bpp)
                        .zip(data.iter())
                        .map(|(cur, last)| cur.wrapping_sub(*last)),
                );
            }
            Self::Up => {
                if last_line.is_empty() {
                    buf.extend_from_slice(data);
                } else {
                    assert_eq!(data.len(), last_line.len());
                    buf.extend(
                        data.iter()
                            .zip(last_line.iter())
                            .map(|(cur, last)| cur.wrapping_sub(*last)),
                    );
                };
            }
            Self::Average => {
                for (i, byte) in data.iter().enumerate() {
                    if last_line.is_empty() {
                        buf.push(match i.checked_sub(bpp) {
                            Some(x) => byte.wrapping_sub(data[x] >> 1),
                            None => *byte,
                        });
                    } else {
                        buf.push(match i.checked_sub(bpp) {
                            Some(x) => byte.wrapping_sub(
                                ((u16::from(data[x]) + u16::from(last_line[i])) >> 1) as u8,
                            ),
                            None => byte.wrapping_sub(last_line[i] >> 1),
                        });
                    };
                }
            }
            Self::Paeth => {
                for (i, byte) in data.iter().enumerate() {
                    if last_line.is_empty() {
                        buf.push(match i.checked_sub(bpp) {
                            Some(x) => byte.wrapping_sub(data[x]),
                            None => *byte,
                        });
                    } else {
                        buf.push(match i.checked_sub(bpp) {
                            Some(x) => byte.wrapping_sub(paeth_predictor(
                                data[x],
                                last_line[i],
                                last_line[x],
                            )),
                            None => byte.wrapping_sub(last_line[i]),
                        });
                    };
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn unfilter_line(
        self,
        bpp: usize,
        data: &[u8],
        last_line: &[u8],
        buf: &mut Vec<u8>,
    ) -> Result<(), PngError> {
        buf.clear();
        buf.reserve(data.len());
        assert!(data.len() >= bpp);
        assert_eq!(data.len(), last_line.len());
        match self {
            Self::None => {
                buf.extend_from_slice(data);
            }
            Self::Sub => {
                for (i, &cur) in data.iter().enumerate() {
                    let prev_byte = i.checked_sub(bpp).and_then(|x| buf.get(x).copied());
                    buf.push(match prev_byte {
                        Some(b) => cur.wrapping_add(b),
                        None => cur,
                    });
                }
            }
            Self::Up => {
                buf.extend(
                    data.iter()
                        .zip(last_line)
                        .map(|(&cur, &last)| cur.wrapping_add(last)),
                );
            }
            Self::Average => {
                for (i, (&cur, &last)) in data.iter().zip(last_line).enumerate() {
                    let prev_byte = i.checked_sub(bpp).and_then(|x| buf.get(x).copied());
                    buf.push(match prev_byte {
                        Some(b) => cur.wrapping_add(((u16::from(b) + u16::from(last)) >> 1) as u8),
                        None => cur.wrapping_add(last >> 1),
                    });
                }
            }
            Self::Paeth => {
                for (i, (&cur, &up)) in data.iter().zip(last_line).enumerate() {
                    buf.push(
                        match i
                            .checked_sub(bpp)
                            .map(|x| (buf.get(x).copied(), last_line.get(x).copied()))
                        {
                            Some((Some(left), Some(left_up))) => {
                                cur.wrapping_add(paeth_predictor(left, up, left_up))
                            }
                            _ => cur.wrapping_add(up),
                        },
                    );
                }
            }
            _ => return Err(PngError::InvalidData),
        }
        Ok(())
    }
}

fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = i32::from(a) + i32::from(b) - i32::from(c);
    let pa = (p - i32::from(a)).abs();
    let pb = (p - i32::from(b)).abs();
    let pc = (p - i32::from(c)).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
