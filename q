[1mdiff --git a/src/nbt_tag/mod.rs b/src/nbt_tag/mod.rs[m
[1mindex fc35d6b..f4fa654 100644[m
[1m--- a/src/nbt_tag/mod.rs[m
[1m+++ b/src/nbt_tag/mod.rs[m
[36m@@ -1,37 +1,49 @@[m
[31m-// ## Author[m
[31m-// - caelunshun, mrmarkolinus[m
[31m-//[m
[31m-// ## Date[m
[31m-// - 2023-12-17[m
[31m-//[m
[31m-// ## File Version[m
[31m-// - 1.0.2[m
[31m-//[m
[31m-// ## Changelog[m
[31m-// - 1.0.0: Initial version [caelunshun:2019-07-09][m
[31m-// - 1.0.1: Splitted the file_parser logic from the nbt_tag logic [mrmarkolinus:2023-12-17][m
[31m-// - 1.0.2: Added support for json-nbt bidirectional conversion [mrmarkolinus:2023-12-17][m
[32m+[m[32m// nbt_tag/mod.rs[m
[32m+[m
[32m+[m[32m//! # NBT Tag Module[m
[32m+[m[32m//![m
[32m+[m[32m//! This module defines the structures and enums for handling NBT (Named Binary Tag) data,[m
[32m+[m[32m//! which is the binary format used by Minecraft to store structured data.[m
 [m
 use byteorder::{BigEndian, WriteBytesExt};[m
[32m+[m[32muse serde::{Deserialize, Serialize};[m
 use std::collections::HashMap;[m
[31m-use std::io::Write;[m
[31m-use serde::{Serialize, Deserialize};[m
 use std::fs;[m
[31m-use std::io::{self, BufWriter, BufReader};[m
[32m+[m[32muse std::io::{self, BufReader, BufWriter, Write};[m
[32m+[m[32muse thiserror::Error;[m
 use derive_new::new;[m
 [m
[31m-#[cfg(test)][m
[31m-mod tests;[m
[32m+[m[32m/// Custom error type for NBT Tag operations.[m
[32m+[m[32m#[derive(Error, Debug)][m
[32m+[m[32mpub enum NbtTagError {[m
[32m+[m[32m    #[error("I/O error: {0}")][m
[32m+[m[32m    Io(#[from] io::Error),[m
[32m+[m
[32m+[m[32m    #[error("Serialization/Deserialization error: {0}")][m
[32m+[m[32m    Serde(#[from] serde_json::Error),[m
[32m+[m
[32m+[m[32m    #[error("Invalid NBT tag type: {0}")][m
[32m+[m[32m    InvalidTagType(u8),[m
[32m+[m
[32m+[m[32m    #[error("Unknown compression format: {0}")][m
[32m+[m[32m    UnknownCompression(u8),[m
 [m
[32m+[m[32m    #[error("Invalid chunk header length.")][m
[32m+[m[32m    InvalidChunkHeaderLength,[m
 [m
[32m+[m[32m    #[error("Chunk index out of bounds.")][m
[32m+[m[32m    ChunkIndexOutOfBounds,[m
[32m+[m[32m}[m
[32m+[m
[32m+[m[32m/// Represents an NBT (Named Binary Tag) compound.[m
 #[derive(Clone, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagCompound {[m
     pub name: String,[m
     pub values: HashMap<String, NbtTag>,[m
 }[m
 [m
[31m-[m
 impl NbtTagCompound {[m
[32m+[m[32m    /// Creates a new `NbtTagCompound` with the given name.[m
     pub fn new(name: &str) -> Self {[m
         Self {[m
             name: name.to_string(),[m
[36m@@ -39,82 +51,39 @@[m [mimpl NbtTagCompound {[m
         }[m
     }[m
 [m
[31m-/*     pub fn get(&self, name: &str) -> Option<NbtTag> {[m
[31m-        self.values.get(name).cloned()[m
[31m-    }[m
[31m-[m
[31m-    pub fn set(&mut self, name: &str, value: NbtTag) {[m
[31m-        self.values.insert(name.to_string(), value);[m
[31m-    } */[m
[31m-[m
[31m-    pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {[m
[31m-        // Open a file for writing.[m
[31m-        let file = fs::File::create(path)?;[m
[31m-        let writer = BufWriter::new(file); // Using a BufWriter for more efficient writes.[m
[31m-[m
[31m-        // Write the pretty-printed JSON to the file.[m
[31m-        serde_json::to_writer_pretty(writer, &self)?;[m
[31m-        [m
[31m-        Ok(())[m
[31m-    }[m
[31m-[m
[31m-    /* pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {[m
[31m-        // Open a file for writing.[m
[32m+[m[32m    /// Serializes the compound to a pretty-printed JSON file at the specified path.[m
[32m+[m[32m    pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), NbtTagError> {[m
         let file = fs::File::create(path)?;[m
[31m-        let writer = BufWriter::new(file); // Using a BufWriter for more efficient writes.[m
[31m-[m
[31m-        // Write the pretty-printed JSON to the file.[m
[32m+[m[32m        let writer = BufWriter::new(file);[m
         serde_json::to_writer_pretty(writer, &self)?;[m
[31m-        [m
         Ok(())[m
     }[m
[31m- */[m
[31m-[m
[31m-    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, io::Error> {[m
 [m
[32m+[m[32m    /// Deserializes an `NbtTagCompound` from a JSON file at the specified path.[m
[32m+[m[32m    pub fn from_json<P: AsRef<std::path::Path>>(path: P) -> Result<Self, NbtTagError> {[m
         let file = fs::File::open(path)?;[m
[31m-        let reader = BufReader::new(file); // Wrap the file in a BufReader, since very large file are expected.[m
[31m-[m
[31m-        // Deserialize the JSON data directly from the stream.[m
[32m+[m[32m        let reader = BufReader::new(file);[m
         let deserialized_nbt = serde_json::from_reader(reader)?;[m
[31m-        [m
         Ok(deserialized_nbt)[m
[31m-[m
     }[m
[31m-[m
[31m-    /* pub fn from_json(&self, path: String) -> PyResult<Self> {[m
[31m-        let path = PathBuf::from(path);[m
[31m-        let file = fs::File::open(&path)[m
[31m-            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;[m
[31m-        let reader = BufReader::new(file); // Wrap the file in a BufReader[m
[31m-[m
[31m-        // Deserialize the JSON data directly from the stream.[m
[31m-        serde_json::from_reader(reader)[m
[31m-            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))[m
[31m-    } */[m
 }[m
 [m
[31m-/// Represents the type of an NBT (Named Binary Tag) tag.[m
[31m-///[m
[31m-/// NBT is a tag-based binary format used to store structured data.[m
[31m-/// Each `NbtTagType` variant corresponds to a different data type[m
[31m-/// in the NBT specification.[m
[31m-[m
[31m-#[derive(Clone, Copy, new,  Debug, PartialEq, Eq, Hash, Serialize, Deserialize)][m
[32m+[m[32m/// Represents the type of an NBT tag.[m
[32m+[m[32m#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)][m
 pub enum NbtTagType {[m
[31m-    End,[m
[31m-    Byte,[m
[31m-    Short,[m
[31m-    Int,[m
[31m-    Long,[m
[31m-    Float,[m
[31m-    Double,[m
[31m-    ByteArray,[m
[31m-    String,[m
[31m-    List,[m
[31m-    Compound,[m
[31m-    IntArray,[m
[31m-    LongArray,[m
[32m+[m[32m    End = 0,[m
[32m+[m[32m    Byte = 1,[m
[32m+[m[32m    Short = 2,[m
[32m+[m[32m    Int = 3,[m
[32m+[m[32m    Long = 4,[m
[32m+[m[32m    Float = 5,[m
[32m+[m[32m    Double = 6,[m
[32m+[m[32m    ByteArray = 7,[m
[32m+[m[32m    String = 8,[m
[32m+[m[32m    List = 9,[m
[32m+[m[32m    Compound = 10,[m
[32m+[m[32m    IntArray = 11,[m
[32m+[m[32m    LongArray = 12,[m
 }[m
 [m
 impl Default for NbtTagType {[m
[36m@@ -124,49 +93,34 @@[m [mimpl Default for NbtTagType {[m
 }[m
 [m
 impl NbtTagType {[m
[31m-    fn id(&self) -> u8 {[m
[31m-        match self {[m
[31m-            NbtTagType::End => 0,[m
[31m-            NbtTagType::Byte => 1,[m
[31m-            NbtTagType::Short => 2,[m
[31m-            NbtTagType::Int => 3,[m
[31m-            NbtTagType::Long => 4,[m
[31m-            NbtTagType::Float => 5,[m
[31m-            NbtTagType::Double => 6,[m
[31m-            NbtTagType::ByteArray => 7,[m
[31m-            NbtTagType::String => 8,[m
[31m-            NbtTagType::List => 9,[m
[31m-            NbtTagType::Compound => 10,[m
[31m-            NbtTagType::IntArray => 11,[m
[31m-            NbtTagType::LongArray => 12,[m
[31m-        }[m
[32m+[m[32m    /// Retrieves the byte identifier for the tag type.[m
[32m+[m[32m    pub fn id(&self) -> u8 {[m
[32m+[m[32m        *self as u8[m
     }[m
 [m
[31m-    pub fn from_id(id: u8) -> Option<Self> {[m
[32m+[m[32m    /// Constructs an `NbtTagType` from its byte identifier.[m
[32m+[m[32m    pub fn from_id(id: u8) -> Result<Self, NbtTagError> {[m
         match id {[m
[31m-            0 => Some(NbtTagType::End),[m
[31m-            1 => Some(NbtTagType::Byte),[m
[31m-            2 => Some(NbtTagType::Short),[m
[31m-            3 => Some(NbtTagType::Int),[m
[31m-            4 => Some(NbtTagType::Long),[m
[31m-            5 => Some(NbtTagType::Float),[m
[31m-            6 => Some(NbtTagType::Double),[m
[31m-            7 => Some(NbtTagType::ByteArray),[m
[31m-            8 => Some(NbtTagType::String),[m
[31m-            9 => Some(NbtTagType::List),[m
[31m-            10 => Some(NbtTagType::Compound),[m
[31m-            11 => Some(NbtTagType::IntArray),[m
[31m-            12 => Some(NbtTagType::LongArray),[m
[31m-            _ => None,[m
[32m+[m[32m            0 => Ok(NbtTagType::End),[m
[32m+[m[32m            1 => Ok(NbtTagType::Byte),[m
[32m+[m[32m            2 => Ok(NbtTagType::Short),[m
[32m+[m[32m            3 => Ok(NbtTagType::Int),[m
[32m+[m[32m            4 => Ok(NbtTagType::Long),[m
[32m+[m[32m            5 => Ok(NbtTagType::Float),[m
[32m+[m[32m            6 => Ok(NbtTagType::Double),[m
[32m+[m[32m            7 => Ok(NbtTagType::ByteArray),[m
[32m+[m[32m            8 => Ok(NbtTagType::String),[m
[32m+[m[32m            9 => Ok(NbtTagType::List),[m
[32m+[m[32m            10 => Ok(NbtTagType::Compound),[m
[32m+[m[32m            11 => Ok(NbtTagType::IntArray),[m
[32m+[m[32m            12 => Ok(NbtTagType::LongArray),[m
[32m+[m[32m            _ => Err(NbtTagError::InvalidTagType(id)),[m
         }[m
     }[m
 }[m
 [m
 /// Represents an NBT (Named Binary Tag) tag.[m
[31m-///[m
[31m-/// This enum encapsulates all possible NBT tags, each variant holding[m
[31m-/// data corresponding to its type.[m
[31m-#[derive(Clone, new, Debug, Serialize, Deserialize)][m
[32m+[m[32m#[derive(Clone, Debug, Serialize, Deserialize)][m
 pub enum NbtTag {[m
     End,[m
     Byte(NbtTagByte),[m
[36m@@ -190,7 +144,7 @@[m [mimpl Default for NbtTag {[m
 }[m
 [m
 impl NbtTag {[m
[31m-[m
[32m+[m[32m    /// Returns the type of the NBT tag.[m
     pub fn ty(&self) -> NbtTagType {[m
         match &self {[m
             NbtTag::End => NbtTagType::End,[m
[36m@@ -205,10 +159,11 @@[m [mimpl NbtTag {[m
             NbtTag::List(_) => NbtTagType::List,[m
             NbtTag::Compound(_) => NbtTagType::Compound,[m
             NbtTag::IntArray(_) => NbtTagType::IntArray,[m
[31m-            NbtTag::LongArray(_) => NbtTagType::End,[m
[32m+[m[32m            NbtTag::LongArray(_) => NbtTagType::LongArray,[m
         }[m
[31m-    } [m
[32m+[m[32m    }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagByte` if the tag is of type `Byte`.[m
     pub fn byte(&self) -> Option<NbtTagByte> {[m
         if let NbtTag::Byte(x) = self {[m
             Some(x.clone())[m
[36m@@ -217,6 +172,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagShort` if the tag is of type `Short`.[m
     pub fn short(&self) -> Option<NbtTagShort> {[m
         if let NbtTag::Short(x) = self {[m
             Some(x.clone())[m
[36m@@ -225,6 +181,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagInt` if the tag is of type `Int`.[m
     pub fn int(&self) -> Option<NbtTagInt> {[m
         if let NbtTag::Int(x) = self {[m
             Some(x.clone())[m
[36m@@ -233,6 +190,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagLong` if the tag is of type `Long`.[m
     pub fn long(&self) -> Option<NbtTagLong> {[m
         if let NbtTag::Long(x) = self {[m
             Some(x.clone())[m
[36m@@ -241,6 +199,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagFloat` if the tag is of type `Float`.[m
     pub fn float(&self) -> Option<NbtTagFloat> {[m
         if let NbtTag::Float(x) = self {[m
             Some(x.clone())[m
[36m@@ -249,6 +208,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagDouble` if the tag is of type `Double`.[m
     pub fn double(&self) -> Option<NbtTagDouble> {[m
         if let NbtTag::Double(x) = self {[m
             Some(x.clone())[m
[36m@@ -257,6 +217,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagByteArray` if the tag is of type `ByteArray`.[m
     pub fn byte_array(&self) -> Option<NbtTagByteArray> {[m
         if let NbtTag::ByteArray(x) = self {[m
             Some(x.clone())[m
[36m@@ -265,6 +226,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagString` if the tag is of type `String`.[m
     pub fn string(&self) -> Option<NbtTagString> {[m
         if let NbtTag::String(x) = self {[m
             Some(x.clone())[m
[36m@@ -273,6 +235,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagList` if the tag is of type `List`.[m
     pub fn list(&self) -> Option<NbtTagList> {[m
         if let NbtTag::List(x) = self {[m
             Some(x.clone())[m
[36m@@ -281,6 +244,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a reference to `NbtTagList` if the tag is of type `List`.[m
     pub fn list_as_ref(&self) -> Option<&NbtTagList> {[m
         if let NbtTag::List(x) = self {[m
             Some(x)[m
[36m@@ -289,6 +253,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagCompound` if the tag is of type `Compound`.[m
     pub fn compound(&self) -> Option<NbtTagCompound> {[m
         if let NbtTag::Compound(x) = self {[m
             Some(x.clone())[m
[36m@@ -297,6 +262,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a reference to `NbtTagCompound` if the tag is of type `Compound`.[m
     pub fn compound_as_ref(&self) -> Option<&NbtTagCompound> {[m
         if let NbtTag::Compound(x) = self {[m
             Some(x)[m
[36m@@ -305,6 +271,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagIntArray` if the tag is of type `IntArray`.[m
     pub fn int_array(&self) -> Option<NbtTagIntArray> {[m
         if let NbtTag::IntArray(x) = self {[m
             Some(x.clone())[m
[36m@@ -313,6 +280,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a cloned `NbtTagLongArray` if the tag is of type `LongArray`.[m
     pub fn long_array(&self) -> Option<NbtTagLongArray> {[m
         if let NbtTag::LongArray(x) = self {[m
             Some(x.clone())[m
[36m@@ -321,6 +289,7 @@[m [mimpl NbtTag {[m
         }[m
     }[m
 [m
[32m+[m[32m    /// Retrieves a reference to `NbtTagLongArray` if the tag is of type `LongArray`.[m
     pub fn long_array_as_ref(&self) -> Option<&NbtTagLongArray> {[m
         if let NbtTag::LongArray(x) = self {[m
             Some(x)[m
[36m@@ -328,67 +297,65 @@[m [mimpl NbtTag {[m
             None[m
         }[m
     }[m
[31m-[m
 }[m
 [m
[31m-[m
[31m-[m
[32m+[m[32m/// Represents an NBT `Byte` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagByte {[m
     pub name: String,[m
     pub value: i8,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `Short` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagShort {[m
     pub name: String,[m
     pub value: i16,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `Int` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagInt {[m
     pub name: String,[m
     pub value: i32,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `Long` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagLong {[m
     pub name: String,[m
     pub value: i64,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `Float` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagFloat {[m
     pub name: String,[m
     pub value: f32,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `Double` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagDouble {[m
     pub name: String,[m
     pub value: f64,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `ByteArray` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagByteArray {[m
     pub name: String,[m
     pub values: Vec<i8>,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `String` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagString {[m
     pub name: String,[m
     pub value: String,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `List` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagList {[m
     pub name: String,[m
[36m@@ -396,149 +363,139 @@[m [mpub struct NbtTagList {[m
     pub values: Vec<NbtTag>,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `IntArray` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagIntArray {[m
     pub name: String,[m
     pub values: Vec<i32>,[m
 }[m
 [m
[31m-[m
[32m+[m[32m/// Represents an NBT `LongArray` tag.[m
 #[derive(Clone, new, Debug, Default, Serialize, Deserialize)][m
 pub struct NbtTagLongArray {[m
     pub name: String,[m
     pub values: Vec<i64>,[m
 }[m
 [m
[31m-[m
[31m-pub fn write(buf: &mut Vec<u8>, compound: &NbtTagCompound) {[m
[31m-    write_tag_type(buf, NbtTagType::Compound);[m
[31m-    write_tag_name(buf, &compound.name);[m
[31m-    write_compound(buf, compound);[m
[32m+[m[32m/// Writes an `NbtTagCompound` to a buffer in NBT format.[m
[32m+[m[32mpub fn write(buf: &mut Vec<u8>, compound: &NbtTagCompound) -> Result<(), NbtTagError> {[m
[32m+[m[32m    write_tag_type(buf, NbtTagType::Compound)?;[m
[32m+[m[32m    write_tag_name(buf, &compound.name)?;[m
[32m+[m[32m    write_compound(buf, compound)?;[m
[32m+[m[32m    Ok(())[m
 }[m
 [m
[31m-fn write_compound(buf: &mut Vec<u8>, compound: &NbtTagCompound) {[m
[31m-    for val in compound.values.values() {[m
[31m-        write_value(buf, val, true);[m
[32m+[m[32mfn write_compound(buf: &mut Vec<u8>, compound: &NbtTagCompound) -> Result<(), NbtTagError> {[m
[32m+[m[32m    for value in compound.values.values() {[m
[32m+[m[32m        write_value(buf, value, true)?;[m
     }[m
[32m+[m[32m    // Write the End tag to signify the end of the compound.[m
[32m+[m[32m    write_tag_type(buf, NbtTagType::End)?;[m
[32m+[m[32m    Ok(())[m
 }[m
 [m
[31m-fn write_value(buf: &mut Vec<u8>, value: &NbtTag, write_name: bool) {[m
[32m+[m[32mfn write_value(buf: &mut Vec<u8>, value: &NbtTag, write_name: bool) -> Result<(), NbtTagError> {[m
     let ty = value.ty();[m
[31m-    write_tag_type(buf, ty);[m
[32m+[m[32m    write_tag_type(buf, ty)?;[m
 [m
     match value {[m
         NbtTag::End => (),[m
         NbtTag::Byte(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_i8(val.value).unwrap();[m
[32m+[m[32m            buf.write_i8(val.value)?;[m
         }[m
         NbtTag::Short(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_i16::<BigEndian>(val.value).unwrap();[m
[32m+[m[32m            buf.write_i16::<BigEndian>(val.value)?;[m
         }[m
         NbtTag::Int(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_i32::<BigEndian>(val.value).unwrap();[m
[32m+[m[32m            buf.write_i32::<BigEndian>(val.value)?;[m
         }[m
         NbtTag::Long(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_i64::<BigEndian>(val.value).unwrap();[m
[32m+[m[32m            buf.write_i64::<BigEndian>(val.value)?;[m
         }[m
         NbtTag::Float(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_f32::<BigEndian>(val.value).unwrap();[m
[32m+[m[32m            buf.write_f32::<BigEndian>(val.value)?;[m
         }[m
         NbtTag::Double(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-            buf.write_f64::<BigEndian>(val.value).unwrap();[m
[32m+[m[32m            buf.write_f64::<BigEndian>(val.value)?;[m
         }[m
         NbtTag::ByteArray(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[31m-            }[m
[31m-[m
[31m-            buf.write_i16::<BigEndian>(val.values.len() as i16).unwrap();[m
[31m-            buf.reserve(val.values.len());[m
[31m-[m
[31m-            for x in &val.values {[m
[31m-                buf.write_i8(*x).unwrap();[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[32m+[m[32m            buf.write_i32::<BigEndian>(val.values.len() as i32)?;[m
[32m+[m[32m            buf.extend_from_slice(&val.values);[m
         }[m
         NbtTag::String(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-[m
[31m-            buf.write_u16::<BigEndian>(val.value.len() as u16).unwrap();[m
[31m-            buf.write(val.value.as_bytes()).unwrap();[m
[32m+[m[32m            buf.write_u16::<BigEndian>(val.value.len() as u16)?;[m
[32m+[m[32m            buf.write_all(val.value.as_bytes())?;[m
         }[m
         NbtTag::List(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-[m
[31m-            write_tag_type(buf, val.ty);[m
[31m-            buf.write_i32::<BigEndian>(val.values.len() as i32).unwrap();[m
[31m-[m
[31m-            for val in &val.values {[m
[31m-                // Finally, an actual application of recursion[m
[31m-                write_value(buf, val, false);[m
[32m+[m[32m            write_tag_type(buf, val.ty)?;[m
[32m+[m[32m            buf.write_i32::<BigEndian>(val.values.len() as i32)?;[m
[32m+[m[32m            for item in &val.values {[m
[32m+[m[32m                write_value(buf, item, false)?;[m
             }[m
         }[m
         NbtTag::Compound(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-[m
[31m-            write_compound(buf, val);[m
[32m+[m[32m            write_compound(buf, val)?;[m
         }[m
         NbtTag::IntArray(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-[m
[31m-            buf.write_i32::<BigEndian>(val.values.len() as i32).unwrap();[m
[31m-[m
[31m-            buf.reserve(val.values.len());[m
[31m-[m
[32m+[m[32m            buf.write_i32::<BigEndian>(val.values.len() as i32)?;[m
             for x in &val.values {[m
[31m-                buf.write_i32::<BigEndian>(*x).unwrap();[m
[32m+[m[32m                buf.write_i32::<BigEndian>(*x)?;[m
             }[m
         }[m
         NbtTag::LongArray(val) => {[m
             if write_name {[m
[31m-                write_tag_name(buf, &val.name);[m
[32m+[m[32m                write_tag_name(buf, &val.name)?;[m
             }[m
[31m-[m
[31m-            buf.write_i32::<BigEndian>(val.values.len() as i32).unwrap();[m
[31m-[m
[31m-            buf.reserve(val.values.len());[m
[31m-[m
[32m+[m[32m            buf.write_i32::<BigEndian>(val.values.len() as i32)?;[m
             for x in &val.values {[m
[31m-                buf.write_i64::<BigEndian>(*x).unwrap();[m
[32m+[m[32m                buf.write_i64::<BigEndian>(*x)?;[m
             }[m
         }[m
     }[m
[32m+[m
[32m+[m[32m    Ok(())[m
 }[m
 [m
[31m-fn write_tag_name(buf: &mut Vec<u8>, s: &str) {[m
[31m-    buf.write_i16::<BigEndian>(s.len() as i16).unwrap();[m
[31m-    buf.write(s.as_bytes()).unwrap();[m
[32m+[m[32mfn write_tag_name(buf: &mut Vec<u8>, s: &str) -> Result<(), NbtTagError> {[m
[32m+[m[32m    buf.write_i16::<BigEndian>(s.len() as i16)?;[m
[32m+[m[32m    buf.write_all(s.as_bytes())?;[m
[32m+[m[32m    Ok(())[m
 }[m
 [m
[31m-fn write_tag_type(buf: &mut Vec<u8>, ty: NbtTagType) {[m
[31m-    buf.write_u8(ty.id()).unwrap();[m
[32m+[m[32mfn write_tag_type(buf: &mut Vec<u8>, ty: NbtTagType) -> Result<(), NbtTagError> {[m
[32m+[m[32m    buf.write_u8(ty.id())?;[m
[32m+[m[32m    Ok(())[m
 }[m
