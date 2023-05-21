use binary_layout::prelude::*;

define_layout!(snss_navigation_entry, LittleEndian, {
    // header
    session_id: i32,
    index: i32,
    // variable length fields, will have to handle...
    //url: [u8],
    //title: [u16],

    // page state
    page_state_length: i32,
    //page_state_raw: [u8]

    //
    transition_type: u32,
    type_mask: u32,
    //referrer_url: [u8]
    original_request_url: u32,
    is_overriding_user_agent: u8,
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

pub struct NavigationEntryLengths {
    url: u32,
    title: u32,
    page_state: u32,
    referrer_url: u32,
    search_terms: u32,
    extended_map: u32,
}

impl NavigationEntryLengths {
    pub fn new() -> NavigationEntryLengths {
        NavigationEntryLengths {
            url: 0,
            title: 0,
            page_state: 0,
            referrer_url: 0,
            search_terms: 0,
            extended_map: 0,
        }
    }
}

pub struct NavigationEntry<'a> {
    data: &'a [u8],
    lengths: NavigationEntryLengths,
}

impl NavigationEntry<'_> {
    pub fn new(data: &[u8]) -> NavigationEntry {
        NavigationEntry {
            data,
            lengths: NavigationEntryLengths::new(),
        }
    }
}
