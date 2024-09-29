// nbt_tag/mod.rs

//! # NBT Tag Module
//!
//! This module defines the structures and enums for handling NBT (Named Binary Tag) data,
//! which is the binary format used by Minecraft to store structured data.

use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufReader, BufWriter, Write};
use thiserror::Error;
use derive_new::new;

/// Custom error type for NBT Tag operations.
#[derive(Error, Debug)]
pub enum NbtTagError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid NBT tag type: {0}")]
    InvalidTagType(u8),

    #[error("Unknown compression format: {0}")]
    UnknownCompression(u8),

    #[error("Invalid chunk header length.")]
    InvalidChunkHeaderLength,

    #[error("Chunk index out of bounds.")]
    ChunkIndexOutOfBounds,
}

/// Represents an NBT (Named Binary Tag) compound.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagCompound {
    pub name: String,
    pub values: HashMap<String, NbtTag>,
}

impl NbtTagCompound {
    /// Creates a new `NbtTagCompound` with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::new(),
        }
    }

    /// Serializes the compound to a pretty-printed JSON file at the specified path.
    pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), NbtTagError> {
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    /// Deserializes an `NbtTagCompound` from a JSON file at the specified path.
    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, NbtTagError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let deserialized_nbt = serde_json::from_reader(reader)?;
        Ok(deserialized_nbt)
    }
}

/// Represents the type of an NBT tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NbtTagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl Default for NbtTagType {
    fn default() -> Self {
        NbtTagType::End
    }
}

impl NbtTagType {
    /// Retrieves the byte identifier for the tag type.
    pub fn id(&self) -> u8 {
        *self as u8
    }

    /// Constructs an `NbtTagType` from its byte identifier.
    pub fn from_id(id: u8) -> Result<Self, NbtTagError> {
        match id {
            0 => Ok(NbtTagType::End),
            1 => Ok(NbtTagType::Byte),
            2 => Ok(NbtTagType::Short),
            3 => Ok(NbtTagType::Int),
            4 => Ok(NbtTagType::Long),
            5 => Ok(NbtTagType::Float),
            6 => Ok(NbtTagType::Double),
            7 => Ok(NbtTagType::ByteArray),
            8 => Ok(NbtTagType::String),
            9 => Ok(NbtTagType::List),
            10 => Ok(NbtTagType::Compound),
            11 => Ok(NbtTagType::IntArray),
            12 => Ok(NbtTagType::LongArray),
            _ => Err(NbtTagError::InvalidTagType(id)),
        }
    }
}

/// Represents an NBT (Named Binary Tag) tag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NbtTag {
    End,
    Byte(NbtTagByte),
    Short(NbtTagShort),
    Int(NbtTagInt),
    Long(NbtTagLong),
    Float(NbtTagFloat),
    Double(NbtTagDouble),
    ByteArray(NbtTagByteArray),
    String(NbtTagString),
    List(NbtTagList),
    Compound(NbtTagCompound),
    IntArray(NbtTagIntArray),
    LongArray(NbtTagLongArray),
}

impl Default for NbtTag {
    fn default() -> Self {
        NbtTag::End
    }
}

impl NbtTag {
    /// Returns the type of the NBT tag.
    pub fn ty(&self) -> NbtTagType {
        match &self {
            NbtTag::End => NbtTagType::End,
            NbtTag::Byte(_) => NbtTagType::Byte,
            NbtTag::Short(_) => NbtTagType::Short,
            NbtTag::Int(_) => NbtTagType::Int,
            NbtTag::Long(_) => NbtTagType::Long,
            NbtTag::Float(_) => NbtTagType::Float,
            NbtTag::Double(_) => NbtTagType::Double,
            NbtTag::ByteArray(_) => NbtTagType::ByteArray,
            NbtTag::String(_) => NbtTagType::String,
            NbtTag::List(_) => NbtTagType::List,
            NbtTag::Compound(_) => NbtTagType::Compound,
            NbtTag::IntArray(_) => NbtTagType::IntArray,
            NbtTag::LongArray(_) => NbtTagType::LongArray,
        }
    }

    /// Retrieves a cloned `NbtTagByte` if the tag is of type `Byte`.
    pub fn byte(&self) -> Option<NbtTagByte> {
        if let NbtTag::Byte(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagShort` if the tag is of type `Short`.
    pub fn short(&self) -> Option<NbtTagShort> {
        if let NbtTag::Short(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagInt` if the tag is of type `Int`.
    pub fn int(&self) -> Option<NbtTagInt> {
        if let NbtTag::Int(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagLong` if the tag is of type `Long`.
    pub fn long(&self) -> Option<NbtTagLong> {
        if let NbtTag::Long(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagFloat` if the tag is of type `Float`.
    pub fn float(&self) -> Option<NbtTagFloat> {
        if let NbtTag::Float(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagDouble` if the tag is of type `Double`.
    pub fn double(&self) -> Option<NbtTagDouble> {
        if let NbtTag::Double(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagByteArray` if the tag is of type `ByteArray`.
    pub fn byte_array(&self) -> Option<NbtTagByteArray> {
        if let NbtTag::ByteArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagString` if the tag is of type `String`.
    pub fn string(&self) -> Option<NbtTagString> {
        if let NbtTag::String(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagList` if the tag is of type `List`.
    pub fn list(&self) -> Option<NbtTagList> {
        if let NbtTag::List(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a reference to `NbtTagList` if the tag is of type `List`.
    pub fn list_as_ref(&self) -> Option<&NbtTagList> {
        if let NbtTag::List(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagCompound` if the tag is of type `Compound`.
    pub fn compound(&self) -> Option<NbtTagCompound> {
        if let NbtTag::Compound(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a reference to `NbtTagCompound` if the tag is of type `Compound`.
    pub fn compound_as_ref(&self) -> Option<&NbtTagCompound> {
        if let NbtTag::Compound(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagIntArray` if the tag is of type `IntArray`.
    pub fn int_array(&self) -> Option<NbtTagIntArray> {
        if let NbtTag::IntArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a cloned `NbtTagLongArray` if the tag is of type `LongArray`.
    pub fn long_array(&self) -> Option<NbtTagLongArray> {
        if let NbtTag::LongArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    /// Retrieves a reference to `NbtTagLongArray` if the tag is of type `LongArray`.
    pub fn long_array_as_ref(&self) -> Option<&NbtTagLongArray> {
        if let NbtTag::LongArray(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

/// Represents an NBT `Byte` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagByte {
    pub name: String,
    pub value: i8,
}

/// Represents an NBT `Short` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagShort {
    pub name: String,
    pub value: i16,
}

/// Represents an NBT `Int` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagInt {
    pub name: String,
    pub value: i32,
}

/// Represents an NBT `Long` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagLong {
    pub name: String,
    pub value: i64,
}

/// Represents an NBT `Float` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagFloat {
    pub name: String,
    pub value: f32,
}

/// Represents an NBT `Double` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagDouble {
    pub name: String,
    pub value: f64,
}

/// Represents an NBT `ByteArray` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagByteArray {
    pub name: String,
    pub values: Vec<i8>,
}

/// Represents an NBT `String` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagString {
    pub name: String,
    pub value: String,
}

/// Represents an NBT `List` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagList {
    pub name: String,
    pub ty: NbtTagType,
    pub values: Vec<NbtTag>,
}

/// Represents an NBT `IntArray` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagIntArray {
    pub name: String,
    pub values: Vec<i32>,
}

/// Represents an NBT `LongArray` tag.
#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagLongArray {
    pub name: String,
    pub values: Vec<i64>,
}

/// Writes an `NbtTagCompound` to a buffer in NBT format.
pub fn write(buf: &mut Vec<u8>, compound: &NbtTagCompound) -> Result<(), NbtTagError> {
    write_tag_type(buf, NbtTagType::Compound)?;
    write_tag_name(buf, &compound.name)?;
    write_compound(buf, compound)?;
    Ok(())
}

fn write_compound(buf: &mut Vec<u8>, compound: &NbtTagCompound) -> Result<(), NbtTagError> {
    for value in compound.values.values() {
        write_value(buf, value, true)?;
    }
    // Write the End tag to signify the end of the compound.
    write_tag_type(buf, NbtTagType::End)?;
    Ok(())
}

fn write_value(buf: &mut Vec<u8>, value: &NbtTag, write_name: bool) -> Result<(), NbtTagError> {
    let ty = value.ty();
    write_tag_type(buf, ty)?;

    match value {
        NbtTag::End => (),
        NbtTag::Byte(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i8(val.value)?;
        }
        NbtTag::Short(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i16::<BigEndian>(val.value)?;
        }
        NbtTag::Int(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i32::<BigEndian>(val.value)?;
        }
        NbtTag::Long(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i64::<BigEndian>(val.value)?;
        }
        NbtTag::Float(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_f32::<BigEndian>(val.value)?;
        }
        NbtTag::Double(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_f64::<BigEndian>(val.value)?;
        }
        NbtTag::ByteArray(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i32::<BigEndian>(val.values.len() as i32)?;
            buf.extend_from_slice(&val.values);
        }
        NbtTag::String(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_u16::<BigEndian>(val.value.len() as u16)?;
            buf.write_all(val.value.as_bytes())?;
        }
        NbtTag::List(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            write_tag_type(buf, val.ty)?;
            buf.write_i32::<BigEndian>(val.values.len() as i32)?;
            for item in &val.values {
                write_value(buf, item, false)?;
            }
        }
        NbtTag::Compound(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            write_compound(buf, val)?;
        }
        NbtTag::IntArray(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i32::<BigEndian>(val.values.len() as i32)?;
            for x in &val.values {
                buf.write_i32::<BigEndian>(*x)?;
            }
        }
        NbtTag::LongArray(val) => {
            if write_name {
                write_tag_name(buf, &val.name)?;
            }
            buf.write_i32::<BigEndian>(val.values.len() as i32)?;
            for x in &val.values {
                buf.write_i64::<BigEndian>(*x)?;
            }
        }
    }

    Ok(())
}

fn write_tag_name(buf: &mut Vec<u8>, s: &str) -> Result<(), NbtTagError> {
    buf.write_i16::<BigEndian>(s.len() as i16)?;
    buf.write_all(s.as_bytes())?;
    Ok(())
}

fn write_tag_type(buf: &mut Vec<u8>, ty: NbtTagType) -> Result<(), NbtTagError> {
    buf.write_u8(ty.id())?;
    Ok(())
}
