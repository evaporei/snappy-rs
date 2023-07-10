use libc::{c_char, c_int, size_t};

type snappy_status = c_int;

#[link(name = "snappy")]
extern "C" {
    fn snappy_compress(input: *const c_char,
                       input_length: size_t,
                       compressed: *mut c_char,
                       compressed_length: *mut size_t) -> snappy_status;

    fn snappy_uncompress(compressed: *const c_char,
                         compressed_length: size_t,
                         uncompressed: *mut c_char,
                         uncompressed_length: *mut size_t) -> snappy_status;

    fn snappy_max_compressed_length(source_length: size_t) -> size_t;

    fn snappy_uncompressed_length(compressed: *const c_char,
                                  compressed_length: size_t,
                                  result: *mut size_t) -> snappy_status;

    fn snappy_validate_compressed_buffer(compressed: *const c_char,
                                         compressed_length: size_t) -> snappy_status;
}

#[test]
fn test_len() {
    let len = unsafe { snappy_max_compressed_length(123) };
    println!("{len}");
}
