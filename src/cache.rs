use std::hash::Hash;
use std::hash::Hasher;
use std::io::Write;

extern crate dirs;

fn hash_data_to_hex_string(data: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    let in_data_hash: u64 = hasher.finish();
    let in_data_has_str: String = format!("{:x}", in_data_hash);
    return in_data_has_str;
}

fn get_oxipng_cache_path() -> std::path::PathBuf {
    let mut cache_path: std::path::PathBuf = dirs::cache_dir().unwrap();
    cache_path.push("oxipng");
    return cache_path;
}

fn read_cache_file_and_check_for_hash(hash_string: &str) -> bool {
    let mut cache_path = get_oxipng_cache_path();
    std::fs::create_dir_all(&cache_path).unwrap();
    cache_path.push("optimized_file_hashes");
    let file = match std::fs::File::open(cache_path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    use std::io::BufRead;
    let buf_reader = std::io::BufReader::new(file);
    for line in buf_reader.lines() {
        let line = line.unwrap();
        if line == hash_string {
            return true;
        }
    }
    return false;
}

pub fn check_cache_for_data_hash(data: &[u8]) -> bool {
    let in_data_hash_str = hash_data_to_hex_string(data);
    return read_cache_file_and_check_for_hash(&in_data_hash_str);
}

pub fn write_data_hash_to_cache(data: &[u8]) {
    let data_hash_str: String = hash_data_to_hex_string(data);
    if read_cache_file_and_check_for_hash(&data_hash_str) {
        return;
    }
    let mut cache_path = get_oxipng_cache_path();
    cache_path.push("optimized_file_hashes");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cache_path)
        .unwrap();
    let mut buf_writer = std::io::BufWriter::new(file);
    writeln!(buf_writer, "{}", data_hash_str).unwrap();
}
