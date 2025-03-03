use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use serde::Deserialize;
use byteorder::{LittleEndian, ReadBytesExt};

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnssFileType {
    Session,
    Tab,
}

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
}

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
}

#[derive(Debug)]
pub enum SessionCommand {
    NavigationEntry(NavigationEntry),
    UnprocessedEntry(UnprocessedEntry),
}

#[derive(Debug)]
pub struct NavigationEntry {
    offset: u64,
    id_type: SessionRestoreIdType,
    index: i32,
    url: String,
    title: String,
    page_state_raw: Vec<u8>,
    transition_type: PageTransition,
    has_post_data: Option<bool>,
    referrer_url: Option<String>,
    original_request_url: Option<String>,
    is_overriding_user_agent: Option<bool>,
    timestamp: Option<SystemTime>,
    http_status: Option<i32>,
    referrer_policy: Option<i32>,
    extended_map: HashMap<String, String>,
    task_id: Option<i64>,
    parent_task_id: Option<i64>,
    root_task_id: Option<i64>,
    session_id: Option<i32>,
}

#[derive(Debug)]
pub struct UnprocessedEntry {
    offset: u64,
    id_type: SessionRestoreIdType,
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
        self.cursor.set_position(8);
    }

    pub fn iter_session_commands(&mut self) -> impl Iterator<Item = Result<SessionCommand, SnssError>> + '_ {
        self.reset();
        std::iter::from_fn(move || self.get_next_session_command().transpose())
    }

    fn get_next_session_command(&mut self) -> Result<Option<SessionCommand>, SnssError> {
        let start_offset = self.cursor.position();
        let length = match self.cursor.read_u16::<LittleEndian>() {
            Ok(len) => len,
            Err(_) => return Ok(None), // EOF
        };

        let mut data = vec![0u8; length as usize];
        self.cursor.read_exact(&mut data)?;

        let record_id_type = match self.file_type {
            SnssFileType::Session => SessionRestoreIdType::from_u8(data[0])?,
            SnssFileType::Tab => TabRestoreIdType::from_u8(data[0])?,
        };

        match record_id_type {
            SessionRestoreIdType::CommandUpdateTabNavigation | TabRestoreIdType::CommandUpdateTabNavigation => {
                let mut pickle = PickleIterator::new(data[1..].to_vec(), 4)?;
                let session_id = pickle.read_int32()?;
                let nav = NavigationEntry::from_pickle(&mut pickle, record_id_type, start_offset, Some(session_id))?;
                Ok(Some(SessionCommand::NavigationEntry(nav)))
            }
            _ => Ok(Some(SessionCommand::UnprocessedEntry(UnprocessedEntry {
                offset: start_offset,
                id_type: record_id_type,
            }))),
        }
    }
}

#[derive(Debug)]
pub struct PageTransition {
    core_transition: String,
    qualifiers: Vec<String>,
    value: u32,
}

impl PageTransition {
    pub fn new(value: u32) -> Self {
        let core_transition = match value & 0xff {
            0 => "Link",
            1 => "Typed",
            2 => "AutoBookmark",
            3 => "AutoSubframe",
            4 => "ManualSubframe",
            5 => "Generated",
            6 => "AutoToplevel",
            7 => "FormSubmit",
            8 => "Reload",
            9 => "Keyword",
            10 => "KeywordGenerated",
            _ => "Unknown",
        }.to_string();

        let qualifiers = vec![
            (0x00800000, "Blocked"),
            (0x01000000, "ForwardBack"),
            (0x02000000, "FromAddressBar"),
            (0x04000000, "HomePage"),
            (0x08000000, "FromApi"),
            (0x10000000, "ChainStart"),
            (0x20000000, "ChainEnd"),
            (0x40000000, "ClientRedirect"),
            (0x80000000, "ServerRedirect"),
        ]
        .into_iter()
        .filter(|(flag, _)| (value & 0xffffff00) & flag > 0)
        .map(|(_, name)| name.to_string())
        .collect();

        Self {
            core_transition,
            qualifiers,
            value,
        }
    }
}

fn main() -> Result<(), SnssError> {
    let in_path = Path::new("Session_12345");
    let file_type = if in_path.file_name().unwrap().to_str().unwrap().starts_with("Session_") {
        SnssFileType::Session
    } else if in_path.file_name().unwrap().to_str().unwrap().starts_with("Tabs_") {
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
