// Raw un-exported bindings to miniz for encoding/decoding
// Copyright (c) 2014 Alex Crichton, MIT & Apache licenses
// Originally from flate2 crate
// Modified for use in oxipng

use std::marker;
use std::mem;
use libc::{c_int, c_uint};
use miniz_sys;

pub struct Stream<D: Direction> {
    raw: miniz_sys::mz_stream,
    _marker: marker::PhantomData<D>,
}

pub enum Compress {}
pub enum Decompress {}

#[doc(hidden)]
pub trait Direction {
    unsafe fn destroy(stream: *mut miniz_sys::mz_stream) -> c_int;
}

impl Stream<Compress> {
    pub fn new_compress(
        lvl: c_int,
        window_bits: c_int,
        mem_size: c_int,
        strategy: c_int,
    ) -> Stream<Compress> {
        unsafe {
            let mut state: miniz_sys::mz_stream = mem::zeroed();
            let ret = miniz_sys::mz_deflateInit2(
                &mut state,
                lvl,
                miniz_sys::MZ_DEFLATED,
                window_bits,
                mem_size,
                strategy,
            );
            debug_assert_eq!(ret, 0);
            Stream {
                raw: state,
                _marker: marker::PhantomData,
            }
        }
    }

    pub fn new_decompress() -> Stream<Decompress> {
        unsafe {
            let mut state: miniz_sys::mz_stream = mem::zeroed();
            let ret = miniz_sys::mz_inflateInit2(&mut state, 15);
            debug_assert_eq!(ret, 0);
            Stream {
                raw: state,
                _marker: marker::PhantomData,
            }
        }
    }
}

impl<T: Direction> Stream<T> {
    #[inline]
    pub fn total_in(&self) -> usize {
        self.raw.total_in as usize
    }

    #[inline]
    pub fn total_out(&self) -> usize {
        self.raw.total_out as usize
    }
}

impl Stream<Decompress> {
    pub fn decompress_vec(&mut self, input: &mut [u8], output: &mut Vec<u8>) -> c_int {
        self.raw.avail_in = (input.len() - self.total_in()) as c_uint;
        self.raw.avail_out = (output.capacity() - self.total_out()) as c_uint;

        unsafe {
            self.raw.next_in = input.as_mut_ptr().offset(self.total_in() as isize);
            self.raw.next_out = output.as_mut_ptr().offset(self.total_out() as isize);
            let rc = miniz_sys::mz_inflate(&mut self.raw, miniz_sys::MZ_NO_FLUSH);
            output.set_len(self.total_out() as usize);
            rc
        }
    }
}

impl Stream<Compress> {
    pub fn compress_vec(&mut self, input: &mut [u8], output: &mut Vec<u8>) -> c_int {
        self.raw.avail_in = (input.len() - self.total_in() as usize) as c_uint;
        self.raw.avail_out = (output.capacity() - self.total_out() as usize) as c_uint;

        unsafe {
            self.raw.next_in = input.as_mut_ptr().offset(self.total_in() as isize);
            self.raw.next_out = output.as_mut_ptr().offset(self.total_out() as isize);
            let rc = miniz_sys::mz_deflate(
                &mut self.raw,
                if self.raw.avail_in > 0 {
                    miniz_sys::MZ_NO_FLUSH
                } else {
                    miniz_sys::MZ_FINISH
                },
            );
            output.set_len(self.total_out() as usize);
            rc
        }
    }
}

impl Direction for Compress {
    #[inline]
    unsafe fn destroy(stream: *mut miniz_sys::mz_stream) -> c_int {
        miniz_sys::mz_deflateEnd(stream)
    }
}
impl Direction for Decompress {
    #[inline]
    unsafe fn destroy(stream: *mut miniz_sys::mz_stream) -> c_int {
        miniz_sys::mz_inflateEnd(stream)
    }
}

impl<D: Direction> Drop for Stream<D> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let _ = <D as Direction>::destroy(&mut self.raw);
        }
    }
}
