//! Stream recompression and cleanup using lopdf; flate2 helpers for tooling/tests.

use flate2::write::ZlibEncoder;
use flate2::Compression;
use lopdf::Document;
use std::io::Write;

/// Prune orphans, drop empty streams, then recompress streams via lopdf (zlib /FlateDecode).
pub fn smart_compress(document: &mut Document) {
    document.delete_zero_length_streams();
    document.prune_objects();
    document.compress();
}

/// Decompress then compress to normalize stream encodings where supported.
pub fn recompress_streams_roundtrip(document: &mut Document) {
    document.decompress();
    document.compress();
}

/// # Errors
///
/// Returns an error if the zlib encoder fails to write or finish.
/// Standalone zlib compression (flate2) for pipelines and tests.
pub fn zlib_compress_best(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(data)?;
    enc.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zlib_compress_non_empty() -> std::io::Result<()> {
        let data = b"aegispdf compression smoke test data";
        let out = zlib_compress_best(data)?;
        assert!(!out.is_empty());
        Ok(())
    }
}
