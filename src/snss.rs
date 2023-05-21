#![allow(unused)]
pub mod navigation_entry;

extern crate binary_layout;
extern crate memmap2;

use binary_layout::prelude::*;
use memmap2::Mmap;
use navigation_entry::NavigationEntry;
use std::fs::File;

define_layout!(snss_header, LittleEndian, {
    header: [u8; 4],
    version: [u8; 4],
});

define_layout!(snss_packet, LittleEndian, {
    length: u16,
    data: [u8],
});

pub struct Snss<'a> {
    data: &'a [u8],
    position: u32,
}

impl<'a> Snss<'a> {
    fn new(data: &'a [u8]) -> Snss {
        Snss { data, position: 0 }
    }
}

impl<'a> Iterator for Snss<'a> {
    type Item = NavigationEntry;
    fn next(&mut self) -> Option<NavigationEntry> {
        Some(NavigationEntry::new())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("README.md")?;
    let mmap = unsafe { Mmap::map(&file)? };

    let snss = Snss::new(&mmap);

    Ok(())
}
