use libc::{c_char, size_t};

#[allow(unused)]
#[derive(Debug, PartialEq)]
#[repr(C)]
enum SnappyStatus {
    Ok = 0,
    InvalidInput = 1,
    BufferTooSmall = 2,
}

impl SnappyStatus {
    fn is_ok(&self) -> bool {
        matches!(self, SnappyStatus::Ok)
    }

    fn is_err(&self) -> bool {
        !self.is_ok()
    }
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
            &mut output_len,
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
            &mut uncompressed_len,
        )
    };
    assert_eq!(uncompressed_len_status, SnappyStatus::Ok);

    let mut uncompressed: Vec<u8> = Vec::with_capacity(uncompressed_len);
    let uncompressed_status = unsafe {
        let s = snappy_uncompress(
            output.as_ptr() as *const c_char,
            output_len,
            uncompressed.as_mut_ptr() as *mut c_char,
            &mut uncompressed_len,
        );
        uncompressed.set_len(uncompressed_len);
        s
    };
    assert_eq!(uncompressed_status, SnappyStatus::Ok);

    assert_eq!(uncompressed, input);
}

#[derive(Debug, PartialEq)]
pub enum SnappyError {
    InvalidInput,
    BufferTooSmall,
}

impl From<SnappyStatus> for SnappyError {
    fn from(status: SnappyStatus) -> Self {
        match status {
            SnappyStatus::Ok => panic!("cannot convert from snappy status Ok into snappy error"),
            SnappyStatus::InvalidInput => Self::InvalidInput,
            SnappyStatus::BufferTooSmall => Self::BufferTooSmall,
        }
    }
}

pub fn validate_compressed_buffer(src: &[u8]) -> Result<(), SnappyError> {
    let status =
        unsafe { snappy_validate_compressed_buffer(src.as_ptr() as *const c_char, src.len()) };

    if status.is_err() {
        return Err(status.into());
    }

    Ok(())
}

pub fn compress(src: &[u8]) -> Result<Vec<u8>, SnappyError> {
    let src_len = src.len();
    let src_ptr = src.as_ptr() as *const c_char;

    let mut dst_len = unsafe { snappy_max_compressed_length(src_len) };
    let mut dst: Vec<u8> = Vec::with_capacity(dst_len);
    let dst_ptr = dst.as_mut_ptr() as *mut c_char;

    unsafe {
        let status = snappy_compress(src_ptr, src_len, dst_ptr, &mut dst_len);
        if status.is_err() {
            return Err(status.into());
        }
        dst.set_len(dst_len);
    };

    Ok(dst)
}

pub fn uncompress(src: &[u8]) -> Result<Vec<u8>, SnappyError> {
    let src_len = src.len();
    let src_ptr = src.as_ptr() as *const c_char;

    let mut dst_len: size_t = 0;
    let status = unsafe { snappy_uncompressed_length(src_ptr, src_len, &mut dst_len) };
    if status.is_err() {
        return Err(status.into());
    }

    let mut dst = Vec::with_capacity(dst_len);
    let dst_ptr = dst.as_mut_ptr() as *mut c_char;

    let status = unsafe { snappy_uncompress(src_ptr, src_len, dst_ptr, &mut dst_len) };
    if status.is_err() {
        return Err(status.into());
    }

    unsafe {
        dst.set_len(dst_len);
    }

    Ok(dst)
}

#[test]
fn valid() {
    let d = vec![0xde, 0xad, 0xd0, 0x0d];
    let c: &[u8] = &compress(&d).unwrap();
    assert!(validate_compressed_buffer(c).is_ok());
    assert!(uncompress(c) == Ok(d));
}

#[test]
fn invalid() {
    let d = vec![0, 0, 0, 0];
    assert_eq!(
        validate_compressed_buffer(&d),
        Err(SnappyError::InvalidInput)
    );
    assert_eq!(uncompress(&d), Err(SnappyError::InvalidInput));
}

#[test]
fn empty() {
    let d = vec![];
    assert_eq!(
        validate_compressed_buffer(&d),
        Err(SnappyError::InvalidInput)
    );
    assert_eq!(uncompress(&d), Err(SnappyError::InvalidInput));
    let c = compress(&d).unwrap();
    assert!(validate_compressed_buffer(&c).is_ok());
    assert!(uncompress(&c) == Ok(d));
}
