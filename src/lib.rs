use libc::{c_char, c_int, size_t};
use std::ffi::CString;

#[derive(Debug, PartialEq)]
#[repr(C)]
enum SnappyStatus {
    Ok = 0,
    InvalidInput = 1,
    BufferTooSmall = 2,
}

#[link(name = "snappy")]
extern "C" {
    fn snappy_compress(
        input: *const c_char,
        input_length: size_t,
        compressed: *mut c_char,
        compressed_length: *mut size_t,
    ) -> SnappyStatus;

    fn snappy_uncompress(
        compressed: *const c_char,
        compressed_length: size_t,
        uncompressed: *mut c_char,
        uncompressed_length: *mut size_t,
    ) -> SnappyStatus;

    fn snappy_max_compressed_length(source_length: size_t) -> size_t;

    fn snappy_uncompressed_length(
        compressed: *const c_char,
        compressed_length: size_t,
        result: *mut size_t,
    ) -> SnappyStatus;

    fn snappy_validate_compressed_buffer(
        compressed: *const c_char,
        compressed_length: size_t,
    ) -> SnappyStatus;
}

#[test]
fn test_round_trip() {
    let input = b"The quick brown fox jumps over the lazy dog";
    let mut output_len = unsafe { snappy_max_compressed_length(input.len()) };
    let mut output: Vec<u8> = Vec::with_capacity(output_len);

    let compress_status = unsafe {
        let s = snappy_compress(
            input.as_ptr() as *const c_char,
            input.len(),
            output.as_mut_ptr() as *mut c_char,
            &mut output_len as *mut size_t,
        );
        output.set_len(output_len);
        s
    };
    assert_eq!(compress_status, SnappyStatus::Ok);

    let validate_status =
        unsafe { snappy_validate_compressed_buffer(output.as_ptr() as *const c_char, output_len) };
    assert_eq!(validate_status, SnappyStatus::Ok);

    let mut uncompressed_len: size_t = 0;
    let uncompressed_len_status = unsafe {
        snappy_uncompressed_length(
            output.as_ptr() as *const c_char,
            output_len,
            &mut uncompressed_len as *mut size_t,
        )
    };
    assert_eq!(uncompressed_len_status, SnappyStatus::Ok);

    let mut uncompressed: Vec<u8> = Vec::with_capacity(uncompressed_len);
    let uncompressed_status = unsafe {
        let s = snappy_uncompress(
            output.as_ptr() as *const c_char,
            output_len,
            uncompressed.as_mut_ptr() as *mut c_char,
            &mut uncompressed_len as *mut size_t,
        );
        uncompressed.set_len(uncompressed_len);
        s
    };
    assert_eq!(uncompressed_status, SnappyStatus::Ok);

    assert_eq!(uncompressed, input);
}

pub fn validate_compressed_buffer(src: &[u8]) -> bool {
    unsafe {
        snappy_validate_compressed_buffer(src.as_ptr() as *const c_char, src.len())
            == SnappyStatus::Ok
    }
}

pub fn compress(src: &[u8]) -> Vec<u8> {
    let src_len = src.len() as size_t;
    let src_ptr = src.as_ptr() as *const c_char;

    let mut dst_len = unsafe { snappy_max_compressed_length(src_len) };
    let mut dst: Vec<u8> = Vec::with_capacity(dst_len);
    let dst_ptr = dst.as_mut_ptr() as *mut c_char;

    unsafe {
        snappy_compress(src_ptr, src_len, dst_ptr, &mut dst_len);
        dst.set_len(dst_len);
    };

    dst
}

pub fn uncompress(src: &[u8]) -> Option<Vec<u8>> {
    let src_len = src.len() as size_t;
    let src_ptr = src.as_ptr() as *const c_char;

    let mut dst_len: size_t = 0;
    unsafe { snappy_uncompressed_length(src_ptr, src_len, &mut dst_len) };

    let mut dst = Vec::with_capacity(dst_len);
    let dst_ptr = dst.as_mut_ptr() as *mut c_char;

    if unsafe { snappy_uncompress(src_ptr, src_len, dst_ptr, &mut dst_len) } == SnappyStatus::Ok {
        unsafe {
            dst.set_len(dst_len);
        }
        Some(dst)
    } else {
        None
    }
}

#[test]
fn valid() {
    let d = vec![0xde, 0xad, 0xd0, 0x0d];
    let c: &[u8] = &compress(&d);
    assert!(validate_compressed_buffer(c));
    assert!(uncompress(c) == Some(d));
}

#[test]
fn invalid() {
    let d = vec![0, 0, 0, 0];
    assert!(!validate_compressed_buffer(&d));
    assert!(uncompress(&d).is_none());
}

#[test]
fn empty() {
    let d = vec![];
    assert!(!validate_compressed_buffer(&d));
    assert!(uncompress(&d).is_none());
    let c = compress(&d);
    assert!(validate_compressed_buffer(&c));
    assert!(uncompress(&c) == Some(d));
}
