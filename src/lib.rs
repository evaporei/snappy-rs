use libc::size_t;

#[link(name = "snappy")]
extern "C" {
    fn snappy_max_compressed_length(source_length: size_t) -> size_t;
}

#[test]
fn test_len() {
    let len = unsafe { snappy_max_compressed_length(123) };
    println!("{len}");
}
