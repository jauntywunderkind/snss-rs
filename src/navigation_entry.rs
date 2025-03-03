use binary_layout::prelude::*;

// numbers of fixed bytes before this variable length section
const OFFSET_URL: usize = 8;
const OFFSET_TITLE: usize = 0;
const OFFSET_PAGE_STATE: usize = 4;
const OFFSET_REFERRER_URL: usize = 8;
const OFFSET_SEARCH_TERMS: usize = 16;
const OFFSET_EXTENDED_MAP: usize = 8;

define_layout!(snss_navigation_entry, LittleEndian, {
    session: i32,
    index: i32,
    //url: [u8],

    //title: [u16],

    page_state_length: i32,
    //page_state_raw: [u8]

    transition_type: u32,
    type_mask: u32,
    //referrer_url: [u8]
    //referrer_policy0: u32, // ccl-chromium-snss2 lists referrer_polciy twice???

    original_request_url: u32,
    is_overriding_user_agent: u32, // bool
    timestamp: u64,
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

// absolute position in data for the end of these sections
#[derive(Debug)]
pub struct NavigationEntryPos {
    start: usize,
    url: usize,
    title: usize,
    page_state: usize,
    referrer_url: usize,
    search_terms: usize,
    extended_map: usize,
}

fn get_end_offset(data: &[u8], pos: usize) -> usize {
    let end = pos + 4;
    let len_bytes = &data[pos..end];
    println!("bytes {:?}", len_bytes);
    let len = u32::from_le_bytes(*array_ref![len_bytes, 0, 4]);
    println!("len {} {}:{}", pos, len, len as usize);
    len as usize
}

impl NavigationEntryPos {
    pub fn new(entry: &[u8], pos: Option<usize>) -> [NavigationEntryPos; 2] {
        let start = match pos {
            Some(x) => x,
            None => 0,
        };

        println!("");
        println!("nel-ne {} {:?}", start, entry);

        let url_start = start + OFFSET_URL;
        let url_end = url_start + get_end_offset(entry, url_start);
        println!("nel-url {} {}", url_start, url_end);

        let title_start = url_end + OFFSET_TITLE + 4;
        let title_end = url_start + get_end_offset(entry, title_start);
        println!("nel-title {} {}", title_start, title_end);

        let page_state_start = title_end + OFFSET_PAGE_STATE + 4;
        let page_state_end = page_state_start + get_end_offset(entry, title_end);
        println!("nel-ps {} {}", page_state_start, page_state_end);

        let referrer_url_start = page_state_end + OFFSET_REFERRER_URL + 4;
        let referrer_url_end = referrer_url_start + get_end_offset(entry, referrer_url_start);
        println!("nel-ru {} {}", referrer_url_start, referrer_url_end);

        let search_terms_start = referrer_url_end + OFFSET_SEARCH_TERMS + 4;
        let search_terms_end = search_terms_start + get_end_offset(entry, search_terms_start);
        println!("nel-st {} {}", search_terms_start, search_terms_end);

        let extended_map_start = search_terms_end + OFFSET_SEARCH_TERMS + 4;
        let extended_map_end = extended_map_start + get_end_offset(entry, extended_map_start);
        println!("nel-em {} {}", extended_map_start, extended_map_end);

        [NavigationEntryPos {
            start,
            url: url_start,
            title: title_start,
            page_state: page_state_start,
            referrer_url: referrer_url_start,
            search_terms: search_terms_start,
            extended_map: extended_map_start,
        },
        NavigationEntryPos {
            start,
            url: url_end,
            title: title_end,
            page_state: page_state_end,
            referrer_url: referrer_url_end,
            search_terms: search_terms_end,
            extended_map: extended_map_end,
        }]
    }
}

#[derive(Debug)]
pub struct NavigationEntry<'a> {
    data: &'a [u8],
    pos_start: NavigationEntryPos,
    pos_end: NavigationEntryPos,
}

impl NavigationEntry<'_> {
    pub fn new(data: &[u8], start: Option<usize>) -> NavigationEntry {
        let [pos_start, pos_end] = NavigationEntryPos::new(data, start);
        NavigationEntry { data, pos_start, pos_end }
    }
}
