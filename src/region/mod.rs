// regions/mod.rs

//! # Regions Module
//!
//! This module handles the parsing and processing of Minecraft region files (.mca and .mcr).
//! It provides functionality to read region file headers, extract chunk data, and convert
//! chunk data into NBT compounds.

use crate::file_parser;
use crate::generic_bin::*;
use crate::nbt_tag::*;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Constants defining the structure of a Minecraft region file.
const HEADER_LENGTH: usize = 4096;
const CHUNK_HEADER_LENGTH: usize = 4;
const CHUNK_HEADER_COMPRESSION: usize = CHUNK_HEADER_LENGTH + 1;

/// Represents a Minecraft region file.
pub struct RegionFile {
    bin_content: GenericBinFile,
    num_chunks: usize,
    chunk_offsets: Vec<(u32, u32)>,
}

impl RegionFile {
    /// Creates a new `RegionFile` by parsing the given file path.
    pub fn new(file_path: PathBuf) -> Result<Self, RegionError> {
        let generic_bin = GenericBinFile::new(file_path.clone(), FileType::Region).map_err(|_| RegionError::ParseError("Failed to parse region file"))?;
        let region_file = RegionFile {
            bin_content: generic_bin,
            num_chunks: 0,
            chunk_offsets: Vec::new(),
        };

        let region_content = region_file.bin_content.get_raw_data();

        let header = Self::read_header(region_content)?;
        let offsets = Self::parse_chunk_offsets(header);
        let num_chunks = offsets.len();

        Ok(RegionFile {
            bin_content: region_file.bin_content,
            num_chunks,
            chunk_offsets: offsets,
        })
    }

    /// Returns the number of chunks in the region file.
    pub fn get_chunks_num(&self) -> usize {
        self.num_chunks
    }

    /// Converts all chunks in the region file to a list of NBT compounds.
    pub fn to_compounds_list(&self) -> Result<Vec<NbtTagCompound>, RegionError> {
        let chunks_as_nbt = self.process_all_chunks()?;
        Ok(chunks_as_nbt)
    }

    /// Reads the header from the region file content.
    fn read_header(region_content: &[u8]) -> Result<&[u8], RegionError> {
        if region_content.len() >= HEADER_LENGTH {
            Ok(&region_content[..HEADER_LENGTH])
        } else {
            Err(RegionError::InvalidRegionFile(
                "Data is shorter than expected header length.".into(),
            ))
        }
    }

    /// Parses chunk offsets from the region file header.
    fn parse_chunk_offsets(header: &[u8]) -> Vec<(u32, u32)> {
        header
            .chunks(4)
            .map(|chunk| {
                let offset = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], 0]) << 4;
                let size = u32::from(chunk[3]) * 4096;
                (offset, size)
            })
            .collect()
    }

    /// Processes all chunks in the region file and converts them to NBT compounds.
    fn process_all_chunks(&self) -> Result<Vec<NbtTagCompound>, RegionError> {
        let mut processed_chunks_list = Vec::new();

        for index in 0..self.num_chunks {
            let (offset, _) = self.chunk_offsets[index];
            if offset == 0 {
                continue; // Skip if the chunk is not present
            }

            let chunk_data = self.read_and_decompress_chunk(index)?;
            let chunk_nbt = file_parser::parse_bytes(&chunk_data)
                .map_err(|_| RegionError::ParseError("Failed to parse NBT data".into()))?;

            if let Some(compound) = chunk_nbt.compound() {
                processed_chunks_list.push(compound);
            } else {
                return Err(RegionError::ParseError(
                    "Chunk does not contain a compound tag.".into(),
                ));
            }
        }

        Ok(processed_chunks_list)
    }

    /// Reads and decompresses a chunk from the region file based on its index.
    ///
    /// # Errors
    ///
    /// Returns an error if the chunk index is out of bounds, the offset is invalid,
    /// or decompression fails.
    fn read_and_decompress_chunk(&self, index: usize) -> Result<Vec<u8>, RegionError> {
        if index >= self.chunk_offsets.len() {
            return Err(RegionError::ChunkIndexOutOfBounds);
        }

        let (offset, size) = self.chunk_offsets[index];
        let raw_data = self.bin_content.get_raw_data();

        if (offset as usize) >= raw_data.len() || (offset as usize) + (size as usize) > raw_data.len() {
            return Err(RegionError::InvalidChunkOffsetSize);
        }

        let chunk_data = &raw_data[offset as usize..offset as usize + size as usize];

        if chunk_data.len() < CHUNK_HEADER_COMPRESSION {
            return Err(RegionError::InvalidChunkHeaderLength);
        }

        let real_chunk_len = u32::from_be_bytes([
            chunk_data[0],
            chunk_data[1],
            chunk_data[2],
            chunk_data[3],
        ]) as usize;

        let compression_method = chunk_data[CHUNK_HEADER_LENGTH];
        let chunk_payload = &chunk_data[CHUNK_HEADER_COMPRESSION..CHUNK_HEADER_COMPRESSION + real_chunk_len];

        self.bin_content.decode_binary_data(chunk_payload, &[compression_method])
    }
}

/// Custom error type for region file operations.
#[derive(Error, Debug)]
pub enum RegionError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid region file: {0}")]
    InvalidRegionFile(String),

    #[error("Chunk index out of bounds.")]
    ChunkIndexOutOfBounds,

    #[error("Invalid chunk offset or size.")]
    InvalidChunkOffsetSize,

    #[error("Invalid chunk header length.")]
    InvalidChunkHeaderLength,

    #[error("Failed to parse NBT data: {0}")]
    ParseError(String),
}
