// Raw un-exported bindings to libz for encoding/decoding
// Copyright (c) 2014 Alex Crichton, MIT & Apache licenses
// Originally from flate2 crate for miniz
// Modified for use in Optipng

use std::marker;
use std::mem;
use libc::{c_int, c_uint};
use libz_sys;

pub struct Stream<D: Direction> {
    raw: libz_sys::z_stream,
    _marker: marker::PhantomData<D>,
}

pub enum Compress {}
pub enum Decompress {}

#[doc(hidden)]
pub trait Direction {
    unsafe fn destroy(stream: *mut libz_sys::z_stream) -> c_int;
}

impl Stream<Compress> {
    pub fn new_compress(lvl: c_int,
                        window_bits: c_int,
                        mem_size: c_int,
                        strategy: c_int)
                        -> Stream<Compress> {
        unsafe {
            let mut state: libz_sys::z_stream = mem::zeroed();
            let ret = libz_sys::deflateInit2_(&mut state,
                                              lvl,
                                              libz_sys::Z_DEFLATED,
                                              window_bits,
                                              mem_size,
                                              strategy,
                                              libz_sys::zlibVersion(),
                                              mem::size_of::<libz_sys::z_stream>() as i32);
            debug_assert_eq!(ret, 0);
            Stream {
                raw: state,
                _marker: marker::PhantomData,
            }
        }
    }

    pub fn new_decompress() -> Stream<Decompress> {
        unsafe {
            let mut state: libz_sys::z_stream = mem::zeroed();
            let ret = libz_sys::inflateInit2_(&mut state,
                                              15,
                                              libz_sys::zlibVersion(),
                                              mem::size_of::<libz_sys::z_stream>() as i32);
            debug_assert_eq!(ret, 0);
            Stream {
                raw: state,
                _marker: marker::PhantomData,
            }
        }
    }
}

impl<T: Direction> Stream<T> {
    pub fn total_in(&self) -> u64 {
        self.raw.total_in as u64
    }

    pub fn total_out(&self) -> u64 {
        self.raw.total_out as u64
    }
}

impl Stream<Decompress> {
    pub fn decompress_vec(&mut self, input: &mut [u8], output: &mut Vec<u8>) -> c_int {
        self.raw.avail_in = (input.len() - self.total_in() as usize) as c_uint;
        self.raw.avail_out = (output.capacity() - self.total_out() as usize) as c_uint;

        unsafe {
            self.raw.next_in = input.as_mut_ptr().offset(self.total_in() as isize);
            self.raw.next_out = output.as_mut_ptr().offset(self.total_out() as isize);
            let rc = libz_sys::inflate(&mut self.raw, libz_sys::Z_NO_FLUSH);
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
            let rc = libz_sys::deflate(&mut self.raw,
                                       if self.raw.avail_in > 0 {
                                           libz_sys::Z_NO_FLUSH
                                       } else {
                                           libz_sys::Z_FINISH
                                       });
            output.set_len(self.total_out() as usize);
            rc
        }
    }

    pub fn reset(&mut self) -> c_int {
        unsafe { libz_sys::deflateReset(&mut self.raw) }
    }
}

impl Direction for Compress {
    unsafe fn destroy(stream: *mut libz_sys::z_stream) -> c_int {
        libz_sys::deflateEnd(stream)
    }
}
impl Direction for Decompress {
    unsafe fn destroy(stream: *mut libz_sys::z_stream) -> c_int {
        libz_sys::inflateEnd(stream)
    }
}

impl<D: Direction> Drop for Stream<D> {
    fn drop(&mut self) {
        unsafe {
            let _ = <D as Direction>::destroy(&mut self.raw);
        }
    }
}
