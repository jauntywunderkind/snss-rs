extern crate serde;
extern crate thiserror;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PickleError {
    #[error("Invalid pickle length")]
    InvalidPickleLength,
    #[error("Invalid boolean value")]
    InvalidBool,
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("UTF-16 conversion error")]
    Utf16ConversionError,
    #[error("UTF-16 decoding error: {0}")]
    Utf16Error(#[from] std::string::FromUtf16Error),
}

#[derive(Debug)]
pub struct PickleIterator {
    cursor: Cursor<Vec<u8>>,
    alignment: usize,
}

impl PickleIterator {
    pub fn new(data: Vec<u8>, alignment: usize) -> Result<Self, PickleError> {
        let mut cursor = Cursor::new(data);
        let pickle_length = cursor.read_u32::<LittleEndian>()? as usize;

        if cursor.get_ref().len() != pickle_length + 4 {
            return Err(PickleError::InvalidPickleLength);
        }

        Ok(Self { cursor, alignment })
    }

    // TODO: just impl directly on Cursor<Vec<u8>> !
    /*
    pub fn new(mut cursor: &'a mut Cursor<Vec<u8>>) -> Result<Self, PickleError> {
        println!("cursor");
        /*let pickle_length = */
        //cursor.read_u32::<LittleEndian>()?;

        /*
        if cursor.get_ref().len() != pickle_length + 4 {
            return Err(PickleError::InvalidPickleLength);
        }
        */

        Ok(Self {
            cursor,
            alignment: 4,
        })
    }
     */

    pub fn read_aligned(&mut self, length: usize) -> Result<Vec<u8>, PickleError> {
        let mut buffer = vec![0u8; length];
        self.cursor.read_exact(&mut buffer)?;

        let align_count = self.alignment - (length % self.alignment);
        if align_count != self.alignment {
            println!("seek {} {}", align_count, self.alignment);
            self.cursor.seek(SeekFrom::Current(align_count as i64))?;
        }

        Ok(buffer)
    }

    pub fn read_uint16(&mut self) -> Result<u16, PickleError> {
        let raw = self.read_aligned(2)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    pub fn read_uint32(&mut self) -> Result<u32, PickleError> {
        let raw = self.read_aligned(4)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    pub fn read_uint64(&mut self) -> Result<u64, PickleError> {
        let raw = self.read_aligned(8)?;
        Ok(u64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    pub fn read_int16(&mut self) -> Result<i16, PickleError> {
        let raw = self.read_aligned(2)?;
        Ok(i16::from_le_bytes([raw[0], raw[1]]))
    }

    pub fn read_int32(&mut self) -> Result<i32, PickleError> {
        let raw = self.read_aligned(4)?;
        Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    pub fn read_int64(&mut self) -> Result<i64, PickleError> {
        let raw = self.read_aligned(8)?;
        Ok(i64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    pub fn read_bool(&mut self) -> Result<bool, PickleError> {
        let raw = self.read_int32()?;
        match raw {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PickleError::InvalidBool),
        }
    }

    pub fn read_single(&mut self) -> Result<f32, PickleError> {
        let raw = self.read_aligned(4)?;
        Ok(f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    pub fn read_double(&mut self) -> Result<f64, PickleError> {
        let raw = self.read_aligned(8)?;
        Ok(f64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    pub fn read_string(&mut self) -> Result<String, PickleError> {
        let length = self.read_uint32()? as usize;
        let raw = self.read_aligned(length)?;
        Ok(String::from_utf8(raw)?)
    }

    pub fn read_string16(&mut self) -> Result<String, PickleError> {
        let length = self.read_uint32()? as usize * 2;
        let raw = self.read_aligned(length)?;

        let iter = (0..length).map(|i| u16::from_be_bytes([raw[2 * i], raw[2 * i + 1]]));

        let decode = char::decode_utf16(iter)
            .collect::<Result<String, _>>()
            .unwrap();

        Ok(decode)
    }

    pub fn read_datetime(&mut self) -> Result<SystemTime, PickleError> {
        let microseconds = self.read_uint64()?;
        let duration = std::time::Duration::from_micros(microseconds);
        Ok(UNIX_EPOCH + duration)
    }
}

/*
fn main() {
    // Example usage
    let data = vec![
        // Example pickle data
    ];
    let mut iterator = PickleIterator::new(data, 4).unwrap();
    let value = iterator.read_uint32().unwrap();
    println!("Read value: {}", value);
}
*/
