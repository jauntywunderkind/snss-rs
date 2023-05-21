use std::fs::File;

extern crate binary_layout;
extern crate memmap2;
use binary_layout::prelude::*;
use memmap2::Mmap;

define_layout!(markdown_h1, BigEndian, {
  marker: u8,
  space: u8,
  title: [u8],
});

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("README.md")?;
    let mmap = unsafe { Mmap::map(&file)? };

    let view = markdown_h1::View::new(mmap);

    let marker = view.marker().read();
    let title = view.title();

    print!("got {} {}", marker, &title[0]);

    Ok(())
}
