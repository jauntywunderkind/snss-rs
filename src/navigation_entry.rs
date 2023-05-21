use binary_layout::prelude::*;

// numbers of fixed bytes before this variable length section
const OFFSET_URL: usize = 8;
const OFFSET_TITLE: usize = 0;
const OFFSET_PAGE_STATE: usize = 0;
const OFFSET_REFERRER_URL: usize = 8;
const OFFSET_SEARCH_TERMS: usize = 16;
const OFFSET_EXTENDED_MAP: usize = 8;

define_layout!(snss_navigation_entry, LittleEndian, {
    session_id: i32,
    index: i32,
    //url: [u8],

    //title: [u16],

    page_state_length: i32,
    //page_state_raw: [u8]

    transition_type: u32,
    type_mask: u32,
    //referrer_url: [u8]

    original_request_url: u32,
    is_overriding_user_agent: u32,
    //timestamp: u64,
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
pub struct NavigationEntryLengths {
    url: usize,
    title: usize,
    page_state: usize,
    referrer_url: usize,
    search_terms: usize,
    extended_map: usize,
}

fn as_usize(data: &[u8]) -> [u8; 8] {
    let mut len_usize: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for i in 0..4 {
        len_usize[i] = data[i]
    }
    len_usize
}

fn get_end_offset(data: &[u8], pos: usize, offset: usize) -> usize {
    let len_bytes = &data[offset..offset + 4];
    let len = u32::from_le_bytes(len_bytes);
    pos + offset + len as usize
}

impl NavigationEntryLengths {
    pub fn new(data: &[u8]) -> NavigationEntryLengths {
        let url = get_end_offset(data, 0, OFFSET_URL);
        let title = get_end_offset(data, url, OFFSET_TITLE);
        let page_state = get_end_offset(data, title, OFFSET_PAGE_STATE);
        let referrer_url = get_end_offset(data, page_state, OFFSET_REFERRER_URL);
        let search_terms = get_end_offset(data, referrer_url, OFFSET_SEARCH_TERMS);
        let extended_map = get_end_offset(data, search_terms, OFFSET_EXTENDED_MAP);

        NavigationEntryLengths {
            url,
            title,
            page_state,
            referrer_url,
            search_terms,
            extended_map,
        }
    }
}

pub struct NavigationEntry<'a> {
    data: &'a [u8],
    lengths: NavigationEntryLengths,
}

impl NavigationEntry<'_> {
    pub fn new(data: &[u8]) -> NavigationEntry {
        let lengths = NavigationEntryLengths::new(data);
        NavigationEntry { data, lengths }
    }
}
