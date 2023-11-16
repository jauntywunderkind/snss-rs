#![allow(unused)]
pub mod navigation_entry;

#[macro_use]
extern crate arrayref;
extern crate binary_layout;
extern crate memmap2;

use binary_layout::prelude::*;
use memmap2::Mmap;
use navigation_entry::NavigationEntry;
use std::fs::File;

define_layout!(snss_header, LittleEndian, {
    header: [u8; 4],
    version: u32,
});

define_layout!(snss_packet, LittleEndian, {
    length: u16,
    data: [u8],
});

pub struct Snss<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Snss<'a> {
    fn new(data: &'a [u8]) -> Snss {
        let mut snss = Snss { data, position: 8 };
        snss.assert_header();
        snss
    }
    fn assert_header(&mut self) {
        assert_eq!(snss_header::header::data(self.data), "SNSS".as_bytes());
        assert_eq!(3, snss_header::version::read(self.data));
    }
}

impl<'a> Iterator for Snss<'a> {
    type Item = NavigationEntry<'a>;
    fn next(&mut self) -> Option<NavigationEntry<'a>> {
        if self.data.len() < self.position {
            return None;
        }

        let data_len = snss_packet::length::read(self.data);
        let data_start = self.position + 2;
        let end_position = data_start + data_len as usize;
        //let slice = unsafe { self.data.get_unchecked(self.position..end_position) };
        let slice = &self.data[data_start..end_position];
        let entry = NavigationEntry::new(slice);
        self.position = end_position;
        Some(entry)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("README.md")?;
    let mmap = unsafe { Mmap::map(&file)? };

    let snss = Snss::new(&mmap);

    Ok(())
}
