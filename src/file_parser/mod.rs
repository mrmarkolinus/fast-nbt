// ## Author
// - caelunshun, mrmarkolinus
//
// ## Date
// - 2023-12-17
//
// ## File Version
// - 1.0.1
//
// ## Changelog
// - 1.0.0: Initial version [caelunshun:2019-07-09]
// - 1.0.1: Splitted the file_parser logic from the nbt_tag logic [mrmarkolinus:2023-12-17]

use crate::nbt_tag::*;
use crate::generic_bin;

use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::path::PathBuf;
use std::fs;
use std::io::BufReader;
use std::io::Read;

#[cfg(test)]
mod tests;

pub enum ReadMode {
    EntireFile,
    Stream,
}

pub struct FileParser {
    file_path: PathBuf,
    read_mode: ReadMode,
    file_type: generic_bin::FileType,
}

impl FileParser {
    pub fn new(file_path: PathBuf, read_mode: ReadMode, file_type: generic_bin::FileType) -> Self {
        FileParser { 
            file_path: file_path.to_path_buf(), 
            read_mode,
            file_type
        }

    }

    pub fn parse(&self) -> std::io::Result<NbtTag> {
        let buf = match self.read_mode {
            ReadMode::EntireFile => self.read_entire_file()?,
            ReadMode::Stream => self.read_stream()?,
        };

        // Handle the result from parse_bytes
        match parse_bytes(&buf) {
            Ok(nbt_tag) => Ok(nbt_tag),  // On success, return the NbtTag
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Parse error")),  // On error, return an std::io::Error
        }
    }

    pub fn read (&self) -> std::io::Result<Vec<u8>> {
        let buf = match self.read_mode {
            ReadMode::EntireFile => self.read_entire_file()?,
            ReadMode::Stream => self.read_stream()?,
        };

        Ok(buf)
    }

    fn read_entire_file(&self) -> std::io::Result<Vec<u8>> {
        
        // Open the file and create a buffered reader for efficient reading
        let file = fs::File::open(&self.file_path)?;
        // let decoder = GzDecoder::new(file);
        let mut reader = BufReader::new(file);

        // Read the entire contents into a buffer
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;


        Ok(buf)
    }

    fn read_stream(&self) -> std::io::Result<Vec<u8>> {
        // Implementation for streaming read
        // ...
        //let mut buf = Vec::new();
        //buf = "not implemented".as_bytes().to_vec();
        todo!("not implemented yet");
        //Ok(buf)
    }

}


//TODO: put these guys in FileParser, workaround for region file
pub fn parse_bytes(bytes: &[u8]) -> Result<NbtTag, NbtTagError> {
    let mut cursor = Cursor::new(bytes);
    
    // Read root compound - read type first
    let ty = {
        let id = cursor.read_u8()?;
        NbtTagType::from_id(id)?
    };
    if ty != NbtTagType::Compound {
        return Err(NbtTagError::InvalidTagType(0));
    }

    let name_len = cursor.read_i16::<BigEndian>()?;
    let mut name = String::with_capacity(name_len as usize);
    for _ in 0..name_len {
        let ch = cursor.read_u8()?;
        name.push(ch as char);
    }

    let root = parse_compound(&mut cursor, name)?;

    Ok(NbtTag::Compound(root))
}

fn parse_compound(cursor: &mut Cursor<&[u8]>, name: String) -> Result<NbtTagCompound, NbtTagError> {
    let mut compound = NbtTagCompound::new(name.as_str());

    // Read values until NBT_End is reached
    loop {
        let type_id = cursor.read_u8()?;

        let ty = NbtTagType::from_id(type_id)?;
        if ty == NbtTagType::End {
            // Finish early - nothing more to read
            break;
        }

        // Read name
        let name = {
            let len = cursor.read_i16::<BigEndian>()?;
            let mut name = String::with_capacity(len as usize);
            for _ in 0..len {
                let ch = cursor.read_u8()?;
                name.push(ch as char);
            }

            name
        };

        // Read value
        let value = parse_value(cursor, ty, name.clone())?;

        compound.values.insert(name, value);
    }

    Ok(compound)
}

fn parse_list(cursor: &mut Cursor<&[u8]>, name: String) -> Result<NbtTagList, NbtTagError> {
    // Type of values contained in the list
    let ty = {
        let id = cursor.read_u8()?;
        NbtTagType::from_id(id)?
    };

    // Length of list, in number of values (not bytes)
    let len = cursor.read_i32::<BigEndian>()?;
    if len > 65536 {
        return Err(NbtTagError::MaxNbtListLengthExceeded);
    }

    let mut values = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let val = parse_value(cursor, ty, "".to_string())?;
        // expose to python
        //let py_val = PyNbtTag::new(&val);
        values.push(val);
    }


    Ok(NbtTagList::new(name, ty, values))
}

/// Parses a single NBT value based on its type.
fn parse_value(cursor: &mut Cursor<&[u8]>, ty: NbtTagType, name: String) -> Result<NbtTag, NbtTagError> {
    match ty {
        NbtTagType::End => Err(NbtTagError::InvalidTagType(0)), // Shouldn't occur here.
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
                return Err(NbtTagError::MaxNbtListLengthExceeded);
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
                return Err(NbtTagError::MaxNbtListLengthExceeded);
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
                return Err(NbtTagError::MaxNbtListLengthExceeded);
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
