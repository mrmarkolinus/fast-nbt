// generic_bin/mod.rs

//! # Generic Binary Module
//!
//! This module provides functionality to handle generic binary files used in Minecraft,
//! including decompression based on different compression methods.

use crate::file_parser;
use crate::nbt_tag::{NbtTag, NbtTagCompound};
use flate2::read::{GzDecoder, ZlibDecoder};
use serde::de::Error;
use std::io::{self, ErrorKind, Read};
use std::path::PathBuf;
use thiserror::Error;

/// Enum representing different file types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    Nbt,
    Region,
}

/// Enum representing different compression types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompressionType {
    Uncompressed = 0,
    Gzip = 1,
    Zlib = 2,
}

impl CompressionType {
    /// Constructs a `CompressionType` from a byte identifier.
    pub fn from_u8(value: u8) -> Result<Self, CompressionError> {
        match value {
            0 => Ok(CompressionType::Uncompressed),
            1 => Ok(CompressionType::Gzip),
            2 => Ok(CompressionType::Zlib),
            _ => Err(CompressionError::UnknownCompression(value)),
        }
    }

    /// Returns the byte identifier for the compression type.
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Custom error type for compression-related operations.
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Unknown compression format: {0}")]
    UnknownCompression(u8),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Represents a generic binary file with raw data.
pub struct GenericBinFile {
    raw_data: Vec<u8>,
}

impl GenericBinFile {
    /// Creates a new `GenericBinFile` by reading the entire file.
    pub fn new(file_path: PathBuf, file_type: FileType) -> Result<Self, GenericBinError> {
        let file_parser = file_parser::FileParser::new(
            file_path.clone(),
            file_parser::ReadMode::EntireFile,
            file_type,
        );
        let raw_data = file_parser.read().map_err(|_| GenericBinError::Io(io::Error::new(ErrorKind::Other, "Failed to read generic binary file")))?;
        Ok(GenericBinFile { raw_data })
    }

    /// Retrieves a reference to the raw data.
    pub fn get_raw_data(&self) -> &Vec<u8> {
        &self.raw_data
    }

    /// Converts the raw data to an `NbtTag`.
    pub fn to_tag(&self) -> Result<NbtTag, GenericBinError> {
        let uncompressed_data = self.try_decode_data()?;
        let root = file_parser::parse_bytes(&uncompressed_data)
            .map_err(|_| GenericBinError::ParseError("Invalid NBT data".into()))?;
        Ok(root)
    }

    /// Converts the raw data to an `NbtTagCompound`.
    pub fn to_tag_compound(&self) -> Result<NbtTagCompound, GenericBinError> {
        let tag = self.to_tag()?;
        tag.compound()
            .ok_or_else(|| GenericBinError::ParseError("Root tag is not a compound".into()))
    }

    /// Converts the raw data to a list of `NbtTagCompound`.
    pub fn to_compounds_list(&self) -> Result<Vec<NbtTagCompound>, GenericBinError> {
        let compound = self.to_tag_compound()?;
        Ok(vec![compound])
    }

    /// Attempts to decode the raw data based on known compression methods.
    pub fn try_decode_data(&self) -> Result<Vec<u8>, GenericBinError> {
        let methods = [
            CompressionType::Gzip,
            CompressionType::Zlib,
            CompressionType::Uncompressed,
        ];

        for method in methods.iter() {
            match self.decode_binary_data(&self.raw_data, &[method.to_u8()]) {
                Ok(data) => return Ok(data),
                Err(_) => continue,
            }
        }

        Err(GenericBinError::DecompressionFailed)
    }

    /// Decodes binary data using the specified compression method.
    ///
    /// # Arguments
    ///
    /// * `chunk_payload` - The compressed data payload.
    /// * `chunk_compression_method` - A slice containing the compression method identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the compression method is unknown or decompression fails.
    pub fn decode_binary_data(
        &self,
        chunk_payload: &[u8],
        chunk_compression_method: &[u8],
    ) -> Result<Vec<u8>, GenericBinError> {
        let compression_type = CompressionType::from_u8(chunk_compression_method[0])?;

        match compression_type {
            CompressionType::Gzip => {
                let mut decoder = GzDecoder::new(chunk_payload);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionType::Zlib => {
                let mut decoder = ZlibDecoder::new(chunk_payload);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionType::Uncompressed => Ok(chunk_payload.to_vec()),
        }
    }
}

/// Custom error type for generic binary file operations.
#[derive(Error, Debug)]
pub enum GenericBinError {
    #[error("Compression error: {0}")]
    Compression(#[from] CompressionError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Decompression failed: All methods failed.")]
    DecompressionFailed,

    #[error("Parse error: {0}")]
    ParseError(String),
}
