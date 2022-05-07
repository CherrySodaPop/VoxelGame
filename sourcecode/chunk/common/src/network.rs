use serde::{de::DeserializeOwned, Serialize};

pub fn encode_and_compress<T: Serialize>(data: T) -> Vec<u8> {
    let encoded = bincode::serialize(&data).unwrap();
    let compression_prefs = lzzzz::lz4f::Preferences::default();
    let mut compressed_buffer =
        vec![0; lzzzz::lz4f::max_compressed_size(encoded.len(), &compression_prefs)];
    let compressed_size =
        lzzzz::lz4f::compress(&encoded, &mut compressed_buffer, &compression_prefs).unwrap();
    compressed_buffer[..compressed_size].into()
}

pub fn decode_compressed<T: DeserializeOwned>(compressed: &[u8]) -> T {
    let mut decompressed = Vec::new();
    lzzzz::lz4f::decompress_to_vec(compressed, &mut decompressed).unwrap();
    bincode::deserialize(&decompressed).unwrap()
}
