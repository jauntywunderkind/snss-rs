use std::fs::File;
use std::io::Read;

extern crate memmap2;
use memmap2::Mmap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("README.md")?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    let mmap = unsafe { Mmap::map(&file)? };

    assert_eq!(&contents[..], &mmap[..]);
    Ok(())
}
