/* Encoding and compression functions 
    Encoders:
    - init encoders
    - ~~encode dna (to 4bit)~~ //handled by utils
    - add length (to compressed)
    - add mask
    - extract mask
    - copy file to out (copies from $start to $start+N from input file to output stream)
    Compression Specific:
    - create zstd stream (handled differently by lib?)
    - compressor init
    - compressor create file
    - compressor end stream
    - compressor done
    - compress 
    - write compressed data (to file)
*/ 