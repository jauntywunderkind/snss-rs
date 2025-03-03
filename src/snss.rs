extern crate bitflags;
extern crate byteorder;
extern crate thiserror;

mod iterator;

use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;
use std::time::SystemTime;
use thiserror::Error;

use iterator::{PickleError, PickleIterator};

#[derive(Error, Debug)]
pub enum SnssError {
    #[error("Invalid magic number")]
    InvalidMagic,
    #[error("Unsupported version")]
    UnsupportedVersion,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Pickle error: {0}")]
    PickleError(#[from] PickleError),
    #[error("Invalid command type")]
    InvalidCommandType,
    #[error("Unprocessed entry: {0} {1}")]
    UnprocessedEntry(SnssFileType, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnssFileType {
    Session,
    Tab,
}

impl fmt::Display for SnssFileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnssFileType::Session => write!(f, "Session"),
            SnssFileType::Tab => write!(f, "Tab"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionRestoreIdType {
    CommandSetTabWindow = 0,
    CommandSetWindowBounds = 1,
    CommandSetTabIndexInWindow = 2,
    CommandTabNavigationPathPrunedFromBack = 5,
    CommandUpdateTabNavigation = 6,
    CommandSetSelectedNavigationIndex = 7,
    CommandSetSelectedTabInIndex = 8,
    CommandSetWindowType = 9,
    CommandSetWindowBounds2 = 10,
    CommandTabNavigationPathPrunedFromFront = 11,
    CommandSetPinnedState = 12,
    CommandSetExtensionAppID = 13,
    CommandSetWindowBounds3 = 14,
    CommandSetWindowAppName = 15,
    CommandTabClosed = 16,
    CommandWindowClosed = 17,
    CommandSetTabUserAgentOverride = 18,
    CommandSessionStorageAssociated = 19,
    CommandSetActiveWindow = 20,
    CommandLastActiveTime = 21,
    CommandSetWindowWorkspace = 22,
    CommandSetWindowWorkspace2 = 23,
    CommandTabNavigationPathPruned = 24,
    CommandSetTabGroup = 25,
    CommandSetTabGroupMetadata = 26,
    CommandSetTabGroupMetadata2 = 27,
    CommandSetTabGuid = 28,
    CommandSetTabUserAgentOverride2 = 29,
    CommandSetTabData = 30,
    CommandSetWindowUserTitle = 31,
    CommandSetWindowVisibleOnAllWorkspaces = 32,
    CommandAddTabExtraData = 33,
    CommandAddWindowExtraData = 34,
    EdgeCommandUnknown131 = 131,
    EdgeCommandUnknown132 = 132,
    UnusedCommand = 255,
    Unknown(u8) = 254,
}

impl SessionRestoreIdType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => SessionRestoreIdType::CommandSetTabWindow,
            1 => SessionRestoreIdType::CommandSetWindowBounds,
            2 => SessionRestoreIdType::CommandSetTabIndexInWindow,
            5 => SessionRestoreIdType::CommandTabNavigationPathPrunedFromBack,
            6 => SessionRestoreIdType::CommandUpdateTabNavigation,
            7 => SessionRestoreIdType::CommandSetSelectedNavigationIndex,
            8 => SessionRestoreIdType::CommandSetSelectedTabInIndex,
            9 => SessionRestoreIdType::CommandSetWindowType,
            10 => SessionRestoreIdType::CommandSetWindowBounds2,
            11 => SessionRestoreIdType::CommandTabNavigationPathPrunedFromFront,
            12 => SessionRestoreIdType::CommandSetPinnedState,
            13 => SessionRestoreIdType::CommandSetExtensionAppID,
            14 => SessionRestoreIdType::CommandSetWindowBounds3,
            15 => SessionRestoreIdType::CommandSetWindowAppName,
            16 => SessionRestoreIdType::CommandTabClosed,
            17 => SessionRestoreIdType::CommandWindowClosed,
            18 => SessionRestoreIdType::CommandSetTabUserAgentOverride,
            19 => SessionRestoreIdType::CommandSessionStorageAssociated,
            20 => SessionRestoreIdType::CommandSetActiveWindow,
            21 => SessionRestoreIdType::CommandLastActiveTime,
            22 => SessionRestoreIdType::CommandSetWindowWorkspace,
            23 => SessionRestoreIdType::CommandSetWindowWorkspace2,
            24 => SessionRestoreIdType::CommandTabNavigationPathPruned,
            25 => SessionRestoreIdType::CommandSetTabGroup,
            26 => SessionRestoreIdType::CommandSetTabGroupMetadata,
            27 => SessionRestoreIdType::CommandSetTabGroupMetadata2,
            28 => SessionRestoreIdType::CommandSetTabGuid,
            29 => SessionRestoreIdType::CommandSetTabUserAgentOverride2,
            30 => SessionRestoreIdType::CommandSetTabData,
            31 => SessionRestoreIdType::CommandSetWindowUserTitle,
            32 => SessionRestoreIdType::CommandSetWindowVisibleOnAllWorkspaces,
            33 => SessionRestoreIdType::CommandAddTabExtraData,
            34 => SessionRestoreIdType::CommandAddWindowExtraData,
            131 => SessionRestoreIdType::EdgeCommandUnknown131,
            132 => SessionRestoreIdType::EdgeCommandUnknown132,
            255 => SessionRestoreIdType::UnusedCommand,
            unknown => SessionRestoreIdType::Unknown(unknown),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabRestoreIdType {
    CommandUpdateTabNavigation = 1,
    CommandRestoredEntry = 2,
    CommandWindowDeprecated = 3,
    CommandSelectedNavigationInTab = 4,
    CommandPinnedState = 5,
    CommandSetExtensionAppID = 6,
    CommandSetWindowAppName = 7,
    CommandSetTabUserAgentOverride = 8,
    CommandWindow = 9,
    CommandSetTabGroupData = 10,
    CommandSetTabUserAgentOverride2 = 11,
    CommandSetWindowUserTitle = 12,
    CommandCreateGroup = 13,
    CommandAddTabExtraData = 14,
    UnusedCommand = 255,
    Unknown(u8) = 254,
}

impl TabRestoreIdType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => TabRestoreIdType::CommandUpdateTabNavigation,
            2 => TabRestoreIdType::CommandRestoredEntry,
            3 => TabRestoreIdType::CommandWindowDeprecated,
            4 => TabRestoreIdType::CommandSelectedNavigationInTab,
            5 => TabRestoreIdType::CommandPinnedState,
            6 => TabRestoreIdType::CommandSetExtensionAppID,
            7 => TabRestoreIdType::CommandSetWindowAppName,
            8 => TabRestoreIdType::CommandSetTabUserAgentOverride,
            9 => TabRestoreIdType::CommandWindow,
            10 => TabRestoreIdType::CommandSetTabGroupData,
            11 => TabRestoreIdType::CommandSetTabUserAgentOverride2,
            12 => TabRestoreIdType::CommandSetWindowUserTitle,
            13 => TabRestoreIdType::CommandCreateGroup,
            14 => TabRestoreIdType::CommandAddTabExtraData,
            255 => TabRestoreIdType::UnusedCommand,
            unknown => TabRestoreIdType::Unknown(unknown),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandIdType {
    Session(SessionRestoreIdType),
    Tab(TabRestoreIdType),
    Invalid,
}

#[derive(Debug)]
pub struct UnprocessedEntry {
    command_type: CommandIdType,
    offset: u64,
    length: usize,
}
#[derive(Debug)]
pub enum SessionCommand {
    NavigationEntry(NavigationEntry),
    UnprocessedEntry(UnprocessedEntry),
    EOF,
}

#[derive(Debug, Clone, Copy)]
pub enum CoreTransition {
    Link,
    Typed,
    AutoBookmark,
    AutoSubframe,
    ManualSubframe,
    Generated,
    AutoToplevel,
    FormSubmit,
    Reload,
    Keyword,
    KeywordGenerated,
    Unknown,
}

impl CoreTransition {
    pub fn from_u32(value: u32) -> Self {
        match value & 0xff {
            0 => CoreTransition::Link,
            1 => CoreTransition::Typed,
            2 => CoreTransition::AutoBookmark,
            3 => CoreTransition::AutoSubframe,
            4 => CoreTransition::ManualSubframe,
            5 => CoreTransition::Generated,
            6 => CoreTransition::AutoToplevel,
            7 => CoreTransition::FormSubmit,
            8 => CoreTransition::Reload,
            9 => CoreTransition::Keyword,
            10 => CoreTransition::KeywordGenerated,
            _ => CoreTransition::Unknown,
        }
    }
}

impl fmt::Display for CoreTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match *self {
            CoreTransition::Link => "Link",
            CoreTransition::Typed => "Typed",
            CoreTransition::AutoBookmark => "AutoBookmark",
            CoreTransition::AutoSubframe => "AutoSubframe",
            CoreTransition::ManualSubframe => "ManualSubframe",
            CoreTransition::Generated => "Generated",
            CoreTransition::AutoToplevel => "AutoToplevel",
            CoreTransition::FormSubmit => "FormSubmit",
            CoreTransition::Reload => "Reload",
            CoreTransition::Keyword => "Keyword",
            CoreTransition::KeywordGenerated => "KeywordGenerated",
            CoreTransition::Unknown => "Unknown",
            _ => "Unknown",
        };
        write!(f, "{}", name)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Qualifier: u32 {
        const Blocked = 0x00800000;
        const ForwardBack = 0x01000000;
        const FromAddressBar = 0x02000000;
        const HomePage = 0x04000000;
        const FromApi = 0x08000000;
        const ChainStart = 0x10000000;
        const ChainEnd = 0x20000000;
        const ClientRedirect = 0x40000000;
        const ServerRedirect = 0x80000000;
    }
}

impl Qualifier {
    pub fn from_u32(value: u32) -> Self {
        Qualifier::from_bits_truncate(value & 0xFFFFFF00)
    }
}

#[derive(Debug)]
pub struct PageTransition {
    core_transition: CoreTransition,
    qualifiers: Qualifier,
    value: u32,
}

impl PageTransition {
    pub fn new(value: u32) -> Self {
        let core_transition = CoreTransition::from_u32(value);
        let qualifiers = Qualifier::from_u32(value);

        Self {
            core_transition,
            qualifiers,
            value,
        }
    }
}

#[derive(Debug)]
pub struct NavigationEntry {
    //offset: u64,
    //id_type: SessionRestoreIdType,
    session_id: i32,
    index: i32,
    url: String,
    title: String,
    page_state_raw: Vec<u8>,
    transition_type: PageTransition,
    type_mask: u32,
    unknown: i32,
    referrer_url: Option<String>,
    original_request_url: Option<String>,
    is_overriding_user_agent: Option<bool>,
    search_terms: Option<String>,
    timestamp: SystemTime,
    http_status: Option<i32>,
    referrer_policy: Option<i32>,
    extended_map: HashMap<String, String>,
    task_id: Option<i64>,
    parent_task_id: Option<i64>,
    root_task_id: Option<i64>,
    child_task_id_count: Option<i32>,
}

impl NavigationEntry {
    pub fn from_pickle(pickle: &mut PickleIterator) -> Result<Self, PickleError> {
        let session_id = pickle.read_int32()?;
        let index = pickle.read_int32()?;
        let url = pickle.read_string()?;
        let title = pickle.read_string16()?;
        let page_state_length = pickle.read_int32()?;
        let page_state_raw = pickle.read_aligned(page_state_length as usize)?;
        let transition_type_value = pickle.read_uint32()?;
        let transition_type = PageTransition::new(transition_type_value);
        let type_mask = pickle.read_uint32()?;
        let referrer_url = pickle.read_string().ok();
        let unknown = pickle.read_int32()?;
        let original_request_url = pickle.read_string().ok();
        let is_overriding_user_agent = pickle.read_bool().ok();
        let timestamp = pickle.read_datetime()?;
        let search_terms = pickle.read_string16().ok();
        let http_status = pickle.read_int32().ok();
        let referrer_policy = pickle.read_int32().ok();
        let extended_map_length = pickle.read_int32()?;
        let mut extended_map = HashMap::new();
        for _ in 0..extended_map_length {
            let key = pickle.read_string()?;
            let value = pickle.read_string()?;
            extended_map.insert(key, value);
        }
        let task_id = pickle.read_int64().ok();
        let parent_task_id = pickle.read_int64().ok();
        let root_task_id = pickle.read_int64().ok();
        let child_task_id_count = pickle.read_int32().ok();

        // Construct the NavigationEntry
        Ok(NavigationEntry {
            session_id,
            index,
            url,
            title,
            page_state_raw,
            transition_type,
            type_mask,
            referrer_url,
            unknown,
            original_request_url,
            is_overriding_user_agent,
            timestamp,
            search_terms,
            http_status,
            referrer_policy,
            extended_map,
            task_id,
            parent_task_id,
            root_task_id,
            child_task_id_count,
        })
    }

    pub fn has_post_data(self) -> bool {
        (self.type_mask & 0x01) > 0
    }
}

#[derive(Debug)]
pub struct SnssFile {
    file_type: SnssFileType,
    version: u32,
    cursor: Cursor<Vec<u8>>,
}

impl SnssFile {
    pub fn new(file_type: SnssFileType, mut file: File) -> Result<Self, SnssError> {
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;

        if &header[0..4] != b"SNSS" {
            return Err(SnssError::InvalidMagic);
        }

        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        if version != 1 && version != 3 {
            return Err(SnssError::UnsupportedVersion);
        }

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let cursor = Cursor::new(data);

        Ok(Self {
            file_type,
            version,
            cursor,
        })
    }

    pub fn reset(&mut self) {
        //self.cursor.set_position(8);
        self.cursor.set_position(0);
    }

    pub fn iter_session_commands(
        &mut self,
    ) -> impl Iterator<Item = Result<SessionCommand, SnssError>> + '_ {
        //self.reset();
        std::iter::from_fn(move || Some(self.get_next_session_command()))
    }

    fn get_next_session_command(&mut self) -> Result<SessionCommand, SnssError> {
        let start_pos = self.cursor.position();
        let length = match self.cursor.read_u16::<LittleEndian>() {
            Ok(len) => len,
            Err(_) => return Ok(SessionCommand::EOF),
        };

        let mut data = vec![0u8; length as usize];
        self.cursor.read_exact(&mut data)?;
        let command_id = data[0];

        //let command_id = self.cursor.read_u8()?;
        //println!("Foo {} {} {:?}", self.cursor.position(), length, command_id);

        let command = match self.file_type {
            SnssFileType::Session => {
                CommandIdType::Session(SessionRestoreIdType::from_u8(command_id))
            }
            SnssFileType::Tab => CommandIdType::Tab(TabRestoreIdType::from_u8(command_id)),
            _ => CommandIdType::Invalid,
        };

        let nav_command = match command {
            CommandIdType::Session(session) => {
                session == SessionRestoreIdType::CommandUpdateTabNavigation
            }
            CommandIdType::Tab(tab) => tab == TabRestoreIdType::CommandUpdateTabNavigation,
            _ => false,
        };
        if !nav_command {
            let unprocessed = UnprocessedEntry {
                command_type: command,
                length: length as usize,
                offset: self.cursor.position(),
            };
            return Ok(SessionCommand::UnprocessedEntry(unprocessed));
        }

        let mut pickle = PickleIterator::new(data[1..].to_vec(), 4)?;
        //let pickle = PickleIterator::new(&mut self.cursor);
        let nav = NavigationEntry::from_pickle(&mut pickle)?;
        println!(
            "End {} {}",
            self.cursor.position(),
            start_pos + length as u64 + 2
        );
        Ok(SessionCommand::NavigationEntry(nav))
    }
}

fn main() -> Result<(), SnssError> {
    let in_path = Path::new("Tabs_12345");
    let file_type = if in_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("Session_")
    {
        SnssFileType::Session
    } else if in_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .starts_with("Tabs_")
    {
        SnssFileType::Tab
    } else {
        return Err(SnssError::InvalidCommandType);
    };

    let file = File::open(in_path)?;
    let mut snss_file = SnssFile::new(file_type, file)?;

    for command in snss_file.iter_session_commands() {
        println!("{:?}", command?);
    }

    Ok(())
}
