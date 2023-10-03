/* Encoding and compression functions
    Encoders:
    - init encoders // handled by nafobj
    - ~~encode dna (to 4bit)~~ //handled by utils
    - add length (to compressed)
    - add mask
    - extract mask
    - copy file to out (copies from $start to $start+N from input file to output stream)
    Compression Specific:
    - create zstd stream (handled differently by lib?)
    - compressor init // part of create stream
    - compressor create file // handled externally
    - compressor end stream // handled automatically
*/

use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use zstd::{stream::AutoFinishEncoder, Encoder};

pub fn create_zstd_stream<'a>(
    file: BufWriter<File>,
    level: Option<u8>,
) -> Result<AutoFinishEncoder<'a, BufWriter<File>>, Error> {
    let compression_level: i32 = match level {
        Some(u) => u as i32,
        None => 0, // uses zstd default (3)
    };
    let res = Encoder::new(file, compression_level).unwrap();
    return Ok(res.auto_finish()); // automatically finish the stream on drop
}
