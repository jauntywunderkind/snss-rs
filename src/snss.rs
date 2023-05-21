use std::fs::File;

extern crate binary_layout;
extern crate memmap2;
use binary_layout::prelude::*;
use memmap2::Mmap;

define_layout!(snss_header, LittleEndian, {
    header: [u8, 4],
    version: [u8, 4],
});

define_layout!(snss_packet, LittleEndian, {
    length: u16,
    data: [u8],
});

define_layout!(snss_navigation_entry, LittleEndian, {
    session_id: i32,
    index: i32,
    // variable length fields, will have to handle...
    //url: [u8],
    //title: [u16],
    page_state_length: i32,
    //page_state_raw: [u8]
    transition_type: u32,
    type_mask: u32,
    //referrer_url: [u8]
    original_request_url: u32,
    is_overriding_user_agent: bool,
    //timestamp: datetime,
    //search_terms: [u16],
    http_status: u32,
    referrer_policy: i32,
    extended_map_size: i32,
    // [string, string] kv
    task_id: i64,
    parent_task_id: i64,
    root_task_id: i64,
    child_task_id_count: i32,
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
