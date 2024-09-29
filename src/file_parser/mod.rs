// file_parser/mod.rs

//! Module for parsing various Minecraft binary file formats.

use crate::generic_bin;
use crate::nbt_tag::*;
use crate::FastNbtError;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs;
use std::io::{BufReader, Cursor, Read};
use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// Enum representing different file read modes.
pub enum ReadMode {
    EntireFile,
    Stream,
}

/// Struct responsible for parsing Minecraft binary files.
pub struct FileParser {
    file_path: PathBuf,
    read_mode: ReadMode,
    file_type: generic_bin::FileType,
}

impl FileParser {
    /// Creates a new FileParser instance.
    pub fn new(file_path: PathBuf, read_mode: ReadMode, file_type: generic_bin::FileType) -> Self {
        Self {
            file_path: file_path.clone(),
            read_mode,
            file_type,
        }
    }

    /// Parses the file and returns the root NbtTag.
    pub fn parse(&self) -> Result<NbtTag, FastNbtError> {
        let buffer = match self.read_mode {
            ReadMode::EntireFile => self.read_entire_file()?,
            ReadMode::Stream => self.read_stream()?,
        };

        parse_bytes(&buffer).map_err(|_| FastNbtError::NbtParse("Failed to parse NBT data".into()))
    }

    /// Reads the entire file into a buffer.
    pub fn read(&self) -> Result<Vec<u8>, FastNbtError> {
        match self.read_mode {
            ReadMode::EntireFile => self.read_entire_file(),
            ReadMode::Stream => self.read_stream(),
        }
    }

    /// Reads the entire file into a buffer.
    fn read_entire_file(&self) -> Result<Vec<u8>, FastNbtError> {
        let file = fs::File::open(&self.file_path)
            .map_err(|e| FastNbtError::Io(e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| FastNbtError::Io(e))?;
        Ok(buffer)
    }

    /// Reads the file in stream mode. (Not yet implemented)
    fn read_stream(&self) -> Result<Vec<u8>, FastNbtError> {
        // Future implementation for streaming read.
        Err(FastNbtError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Streaming read not implemented",
        )))
    }
}

/// Parses a byte slice into an NbtTag.
pub fn parse_bytes(bytes: &[u8]) -> Result<NbtTag, NbtTagError> {
    let mut cursor = Cursor::new(bytes);

    // Read root compound type.
    let ty = {
        let id = cursor.read_u8()?;
        NbtTagType::from_id(id)?
    };

    if ty != NbtTagType::Compound {
        return Err(NbtTagError::InvalidTagType(ty.id()));
    }

    // Read the name of the root compound.
    let name_len = cursor.read_i16::<BigEndian>()?;
    let mut name = String::with_capacity(name_len as usize);
    for _ in 0..name_len {
        let ch = cursor.read_u8()?;
        name.push(ch as char);
    }

    let root = parse_compound(&mut cursor, name)?;

    Ok(NbtTag::Compound(root))
}

/// Recursively parses a compound from the cursor.
fn parse_compound(cursor: &mut Cursor<&[u8]>, name: String) -> Result<NbtTagCompound, NbtTagError> {
    let mut compound = NbtTagCompound::new(&name);

    loop {
        let type_id = cursor.read_u8()?;
        let ty = NbtTagType::from_id(type_id)?;

        if ty == NbtTagType::End {
            break;
        }

        let name_len = cursor.read_i16::<BigEndian>()?;
        let mut tag_name = String::with_capacity(name_len as usize);
        for _ in 0..name_len {
            let ch = cursor.read_u8()?;
            tag_name.push(ch as char);
        }

        let value = parse_value(cursor, ty, tag_name)?;
        compound.values.insert(value.get_name().to_string(), value);
    }

    Ok(compound)
}

/// Parses a list from the cursor.
fn parse_list(cursor: &mut Cursor<&[u8]>, name: String) -> Result<NbtTagList, NbtTagError> {
    let ty = {
        let id = cursor.read_u8()?;
        NbtTagType::from_id(id)?
    };

    let len = cursor.read_i32::<BigEndian>()?;
    if len > 65_536 {
        return Err(NbtTagError::InvalidTagType(ty.id()));
    }

    let mut values = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let val = parse_value(cursor, ty, String::new())?;
        values.push(val);
    }

    Ok(NbtTagList::new(name.clone(), ty, values))
}

/// Parses a single NBT value based on its type.
fn parse_value(cursor: &mut Cursor<&[u8]>, ty: NbtTagType, name: String) -> Result<NbtTag, NbtTagError> {
    match ty {
        NbtTagType::End => Err(NbtTagError::InvalidTagType(ty.id())), // Shouldn't occur here.
        NbtTagType::Byte => {
            let x = cursor.read_i8()?;
            Ok(NbtTag::Byte(NbtTagByte::new(name.clone(), x)))
        }
        NbtTagType::Short => {
            let x = cursor.read_i16::<BigEndian>()?;
            Ok(NbtTag::Short(NbtTagShort::new(name.clone(), x)))
        }
        NbtTagType::Int => {
            let x = cursor.read_i32::<BigEndian>()?;
            Ok(NbtTag::Int(NbtTagInt::new(name.clone(), x)))
        }
        NbtTagType::Long => {
            let x = cursor.read_i64::<BigEndian>()?;
            Ok(NbtTag::Long(NbtTagLong::new(name.clone(), x)))
        }
        NbtTagType::Float => {
            let x = cursor.read_f32::<BigEndian>()?;
            Ok(NbtTag::Float(NbtTagFloat::new(name.clone(), x)))
        }
        NbtTagType::Double => {
            let x = cursor.read_f64::<BigEndian>()?;
            Ok(NbtTag::Double(NbtTagDouble::new(name.clone(), x)))
        }
        NbtTagType::ByteArray => {
            let len = cursor.read_i32::<BigEndian>()?;
            if len > 65_536 {
                return Err(NbtTagError::InvalidTagType(ty.id()));
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i8()?;
                buf.push(x);
            }

            Ok(NbtTag::ByteArray(NbtTagByteArray::new(name.clone(), buf)))
        }
        NbtTagType::String => {
            let len = cursor.read_u16::<BigEndian>()?;
            let mut buf = String::with_capacity(len as usize);
            for _ in 0..len {
                let ch = cursor.read_u8()?;
                buf.push(ch as char);
            }
            Ok(NbtTag::String(NbtTagString::new(name.clone(), buf)))
        }
        NbtTagType::List => {
            let list = parse_list(cursor, name)?;
            Ok(NbtTag::List(list))
        }
        NbtTagType::Compound => {
            let compound = parse_compound(cursor, name)?;
            Ok(NbtTag::Compound(compound))
        }
        NbtTagType::IntArray => {
            let len = cursor.read_i32::<BigEndian>()?;
            if len > 65_536 {
                return Err(NbtTagError::InvalidTagType(ty.id()));
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i32::<BigEndian>()?;
                buf.push(x);
            }

            Ok(NbtTag::IntArray(NbtTagIntArray::new(name.clone(), buf)))
        }
        NbtTagType::LongArray => {
            let len = cursor.read_i32::<BigEndian>()?;
            if len > 65_536 {
                return Err(NbtTagError::InvalidTagType(ty.id()));
            }

            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let x = cursor.read_i64::<BigEndian>()?;
                buf.push(x);
            }

            Ok(NbtTag::LongArray(NbtTagLongArray::new(name.clone(), buf)))
        }
    }
}
