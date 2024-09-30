// ## Author
// - caelunshun, mrmarkolinus
//
// ## Date
// - 2023-12-17
//
// ## File Version
// - 1.0.2
//
// ## Changelog
// - 1.0.0: Initial version [caelunshun:2019-07-09]
// - 1.0.1: Splitted the file_parser logic from the nbt_tag logic [mrmarkolinus:2023-12-17]
// - 1.0.2: Added support for json-nbt bidirectional conversion [mrmarkolinus:2023-12-17]

use byteorder::{BigEndian, WriteBytesExt};
use std::collections::HashMap;
use std::io::Write;
use serde::{Serialize, Deserialize};
use std::fs;
use std::io::{self, BufWriter, BufReader};
use thiserror::Error;
use derive_new::new;

#[cfg(test)]
mod tests;
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

    #[error("Max NBT List length exceeded.")]
    MaxNbtListLengthExceeded,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagCompound {
    pub name: String,
    pub values: HashMap<String, NbtTag>,
}


impl NbtTagCompound {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            values: HashMap::new(),
        }
    }

/*     pub fn get(&self, name: &str) -> Option<NbtTag> {
        self.values.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: NbtTag) {
        self.values.insert(name.to_string(), value);
    } */

    pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        // Open a file for writing.
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file); // Using a BufWriter for more efficient writes.

        // Write the pretty-printed JSON to the file.
        serde_json::to_writer_pretty(writer, &self)?;
        
        Ok(())
    }

    /* pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        // Open a file for writing.
        let file = fs::File::create(path)?;
        let writer = BufWriter::new(file); // Using a BufWriter for more efficient writes.

        // Write the pretty-printed JSON to the file.
        serde_json::to_writer_pretty(writer, &self)?;
        
        Ok(())
    }
 */

    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, io::Error> {

        let file = fs::File::open(path)?;
        let reader = BufReader::new(file); // Wrap the file in a BufReader, since very large file are expected.

        // Deserialize the JSON data directly from the stream.
        let deserialized_nbt = serde_json::from_reader(reader)?;
        
        Ok(deserialized_nbt)

    }

    /* pub fn from_json(&self, path: String) -> PyResult<Self> {
        let path = PathBuf::from(path);
        let file = fs::File::open(&path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;
        let reader = BufReader::new(file); // Wrap the file in a BufReader

        // Deserialize the JSON data directly from the stream.
        serde_json::from_reader(reader)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))
    } */
}

/// Represents the type of an NBT (Named Binary Tag) tag.
///
/// NBT is a tag-based binary format used to store structured data.
/// Each `NbtTagType` variant corresponds to a different data type
/// in the NBT specification.

#[derive(Clone, Copy, new,  Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NbtTagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
}

impl Default for NbtTagType {
    fn default() -> Self {
        NbtTagType::End
    }
}

impl NbtTagType {
    fn id(&self) -> u8 {
        match self {
            NbtTagType::End => 0,
            NbtTagType::Byte => 1,
            NbtTagType::Short => 2,
            NbtTagType::Int => 3,
            NbtTagType::Long => 4,
            NbtTagType::Float => 5,
            NbtTagType::Double => 6,
            NbtTagType::ByteArray => 7,
            NbtTagType::String => 8,
            NbtTagType::List => 9,
            NbtTagType::Compound => 10,
            NbtTagType::IntArray => 11,
            NbtTagType::LongArray => 12,
        }
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
///
/// This enum encapsulates all possible NBT tags, each variant holding
/// data corresponding to its type.
#[derive(Clone, new, Debug, Serialize, Deserialize)]
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

    pub fn byte(&self) -> Option<NbtTagByte> {
        if let NbtTag::Byte(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn short(&self) -> Option<NbtTagShort> {
        if let NbtTag::Short(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn int(&self) -> Option<NbtTagInt> {
        if let NbtTag::Int(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn long(&self) -> Option<NbtTagLong> {
        if let NbtTag::Long(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn float(&self) -> Option<NbtTagFloat> {
        if let NbtTag::Float(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn double(&self) -> Option<NbtTagDouble> {
        if let NbtTag::Double(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn byte_array(&self) -> Option<NbtTagByteArray> {
        if let NbtTag::ByteArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn string(&self) -> Option<NbtTagString> {
        if let NbtTag::String(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn list(&self) -> Option<NbtTagList> {
        if let NbtTag::List(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn list_as_ref(&self) -> Option<&NbtTagList> {
        if let NbtTag::List(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn compound(&self) -> Option<NbtTagCompound> {
        if let NbtTag::Compound(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn compound_as_ref(&self) -> Option<&NbtTagCompound> {
        if let NbtTag::Compound(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn int_array(&self) -> Option<NbtTagIntArray> {
        if let NbtTag::IntArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn long_array(&self) -> Option<NbtTagLongArray> {
        if let NbtTag::LongArray(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn long_array_as_ref(&self) -> Option<&NbtTagLongArray> {
        if let NbtTag::LongArray(x) = self {
            Some(x)
        } else {
            None
        }
    }

}



#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagByte {
    pub name: String,
    pub value: i8,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagShort {
    pub name: String,
    pub value: i16,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagInt {
    pub name: String,
    pub value: i32,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagLong {
    pub name: String,
    pub value: i64,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagFloat {
    pub name: String,
    pub value: f32,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagDouble {
    pub name: String,
    pub value: f64,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagByteArray {
    pub name: String,
    pub values: Vec<i8>,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagString {
    pub name: String,
    pub value: String,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagList {
    pub name: String,
    pub ty: NbtTagType,
    pub values: Vec<NbtTag>,
}


#[derive(Clone, new, Debug, Default, Serialize, Deserialize)]
pub struct NbtTagIntArray {
    pub name: String,
    pub values: Vec<i32>,
}


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
                write_tag_name(buf, &val.name);
            }

            buf.write_i16::<BigEndian>(val.values.len() as i16)?;
            buf.reserve(val.values.len());

            for x in &val.values {
                buf.write_i8(*x)?;
            }
        }
        NbtTag::String(val) => {
            if write_name {
                write_tag_name(buf, &val.name);
            }

            buf.write_u16::<BigEndian>(val.value.len() as u16)?;
            buf.write(val.value.as_bytes())?;
        }
        NbtTag::List(val) => {
            if write_name {
                write_tag_name(buf, &val.name);
            }

            write_tag_type(buf, val.ty);
            buf.write_i32::<BigEndian>(val.values.len() as i32)?;

            for val in &val.values {
                // Finally, an actual application of recursion
                write_value(buf, val, false);
            }
        }
        NbtTag::Compound(val) => {
            if write_name {
                write_tag_name(buf, &val.name);
            }

            write_compound(buf, val);
        }
        NbtTag::IntArray(val) => {
            if write_name {
                write_tag_name(buf, &val.name);
            }

            buf.write_i32::<BigEndian>(val.values.len() as i32)?;

            buf.reserve(val.values.len());

            for x in &val.values {
                buf.write_i32::<BigEndian>(*x)?;
            }
        }
        NbtTag::LongArray(val) => {
            if write_name {
                write_tag_name(buf, &val.name);
            }

            buf.write_i32::<BigEndian>(val.values.len() as i32)?;

            buf.reserve(val.values.len());

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
