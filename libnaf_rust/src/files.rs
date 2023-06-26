use std::fs::File;
/* 
    - open file (input/output)
    - create temp file prefix
    - close file (closed when falls out of scope)
    - flush file (saaame)
*/

pub fn open_file(filename: &str) -> File {
    
    match File::open(filename) {
        Err(reason) => panic!("cannot open {}: {}", filename, reason),
        Ok(file) => file,
    }
}

pub fn create_file(filename: &str) -> File {
    
    match File::create(filename) {
        Err(reason) => panic!("cannot open {}: {}", filename, reason),
        Ok(file) => file,
    }
}

pub fn creat_temp_file_prefix() {
    todo!("Create temp file prefix")
}