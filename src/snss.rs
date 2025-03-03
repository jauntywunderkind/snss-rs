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

//define_layout!(snss_layout, LittleEndian, {
//    header: snss_header,
//    packets: [snss_packet],
//});

pub struct Snss<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Snss<'a> {
    fn new(data: &'a [u8]) -> Snss<'a> {
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
            println!("hi-none");
            return None;
        }
        println!("hi");

        let mut prev = self.position;
        self.position += 2;
        let data_end = snss_packet::length::read(&self.data[prev..self.position]); // how to offset?

        println!("hi1 {} {} {}", prev, self.position, data_end);
        //self.position += 2; // snss_packet#length's length
        //let data_start = self.position + 2; // snss_packet.length
        //let end_position = data_start + data_len as usize;
        //let end_position = pos + ;
        //let data_end = self.position + data_len as usize;

        //let pos_data_end = data_end as usize;
        //
        prev = self.position;
        self.position += data_end as usize;
        //self.position += data_end as usize; // or just equals?
        //println!("hi2 {} + {} = {}", self.position, data_len, data_end);
        println!("hi2 {} {}", prev, self.position);

        //let data = &self.data[prev..self.position];
        //let slice = unsafe { self.data.get_unchecked(self.position..pos_data_end) };
        //let slice = &self.data[data_start..end_position];
        //self.position = pos_data_end; 
        println!("hi3 {:?}", self.data);
        let entry = NavigationEntry::new(self.data, Some(self.position));
        println!("hi5 {:?}", entry);
        Some(entry)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("symlink")?;
    let mmap = unsafe { Mmap::map(&file)? };

    let snss = Snss::new(&mmap);

    for i in snss {
        println!("> {:?}", i)
    }

    Ok(())
}
