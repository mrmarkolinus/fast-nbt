// lib.rs

//! # FastNBT
//!
//! `fastnbt` is a Rust library for parsing and interacting with Minecraft binary files.
//! It provides Python bindings via PyO3 for seamless integration with Python projects.

use std::collections::HashMap;
use std::path::PathBuf;

use log::info;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3_log::init as init_pyo3_log;
use thiserror::Error;

pub mod blocks;
pub mod chunk_format;
pub mod file_parser;
pub mod generic_bin;
pub mod nbt_tag;
pub mod region;

/// Custom error type for FastNBT operations.
#[derive(Error, Debug)]
pub enum FastNbtError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("NBT parsing error: {0}")]
    NbtParse(String),

    #[error("Unsupported file extension: {0}")]
    UnsupportedExtension(String),

    #[error("Invalid input path: {0}")]
    InvalidInputPath(String),

    #[error("Python conversion error: {0}")]
    PyConversion(String),
}

/// Wrapper for McWorldDescriptor to expose to Python.
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyMcWorldDescriptor {
    mc_world_descriptor: McWorldDescriptor,
    #[pyo3(get, set)]
    pub tag_compounds_list: Vec<Py<PyDict>>,
}

#[pymethods]
impl PyMcWorldDescriptor {
    /// Creates a new PyMcWorldDescriptor from a Rust McWorldDescriptor.
    #[new]
    pub fn new(rust_mc_world_descriptor: McWorldDescriptor) -> Result<Self, FastNbtError> {
        let py_tag_list = rust_mc_world_descriptor
            .tag_compounds_list
            .iter()
            .map(|item| {
                let tag_root = nbt_tag::NbtTag::Compound(item.clone());
                PyNbtTag::new(&tag_root).python_dict
            })
            .collect();

        Ok(PyMcWorldDescriptor {
            mc_world_descriptor: rust_mc_world_descriptor,
            tag_compounds_list: py_tag_list,
        })
    }

    /// Serializes the NBT data to a JSON file at the specified path.
    fn to_json(&self, path: String) -> PyResult<()> {
        self.mc_world_descriptor
            .to_json(&path)
            .map_err(|e| PyErr::new::<PyIOError, _>(e.to_string()))
    }

    /// Retrieves the Minecraft version.
    fn get_mc_version(&self) -> String {
        self.mc_world_descriptor.get_mc_version()
    }

    /// Searches for a compound by key.
    fn search_compound(&self, key: &str) -> (bool, Vec<Py<PyDict>>) {
        let (found, compounds) = self.mc_world_descriptor.search_compound(key, false);
        let py_tag_list = compounds
            .iter()
            .map(|item| {
                let tag_root = nbt_tag::NbtTag::Compound(item.clone());
                PyNbtTag::new(&tag_root).python_dict
            })
            .collect();
        (found, py_tag_list)
    }

    /// Searches for blocks with the specified resource locations.
    fn search_blocks(&self, block_resource_locations: Vec<String>) -> HashMap<String, Vec<blocks::MinecraftBlock>> {
        self.mc_world_descriptor
            .search_blocks(block_resource_locations)
    }
}

/// Represents a Minecraft world descriptor.
#[pyclass]
#[derive(Clone, Debug, Default)]
pub struct McWorldDescriptor {
    pub input_path: PathBuf,
    pub version: String,
    pub tag_compounds_list: Vec<nbt_tag::NbtTagCompound>,
}

impl McWorldDescriptor {
    /// Creates a new McWorldDescriptor by parsing the input path.
    pub fn new(input_path: PathBuf) -> Result<Self, FastNbtError> {
        let cloned_input_path = input_path.clone();

        let tag_compounds_list = Self::read_input_path(input_path)?;

        Ok(McWorldDescriptor {
            input_path: cloned_input_path,
            version: "0.0.0".to_string(), // Consider extracting the actual version
            tag_compounds_list,
        })
    }

    /// Reads and parses the input path, handling both directories and files.
    fn read_input_path(input_path: PathBuf) -> Result<Vec<nbt_tag::NbtTagCompound>, FastNbtError> {
        let mut nbt_tag_compounds_list = Vec::new();

        if input_path.is_dir() {
            if !input_path.exists() {
                return Err(FastNbtError::InvalidInputPath(
                    "World directory does not exist".into(),
                ));
            }

            let region_path = input_path.join("region");
            if !region_path.exists() || !region_path.is_dir() {
                return Err(FastNbtError::InvalidInputPath(
                    "Subdirectory './region' does not exist".into(),
                ));
            }

            for entry in std::fs::read_dir(&region_path)
                .map_err(|_| FastNbtError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error reading region files",
                )))?
            {
                let entry = entry.map_err(|_| FastNbtError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Error reading a region file entry",
                )))?;
                let file_path = entry.path();
                nbt_tag_compounds_list.append(&mut Self::read_file_format(file_path)?);
            }
        } else {
            nbt_tag_compounds_list.append(&mut Self::read_file_format(input_path)?);
        }

        Ok(nbt_tag_compounds_list)
    }

    /// Determines the file format based on the extension and parses accordingly.
    fn read_file_format(input_path: PathBuf) -> Result<Vec<nbt_tag::NbtTagCompound>, FastNbtError> {
        match input_path.extension().and_then(|e| e.to_str()) {
            Some(ext) if ["mcr", "mca"].contains(&ext) => {
                let region_file = region::RegionFile::new(input_path)?;
                Ok(region_file.to_compounds_list()?)
            }
            Some(ext) if ["nbt", "litematic"].contains(&ext) => {
                let bin_content =
                    generic_bin::GenericBinFile::new(input_path, generic_bin::FileType::Nbt)?;
                Ok(bin_content.to_compounds_list()?)
            }
            Some("json") => {
                let json_content = nbt_tag::NbtTagCompound::from_json(input_path)?;
                Ok(vec![json_content])
            }
            Some(ext) => Err(FastNbtError::UnsupportedExtension(ext.to_string())),
            None => Err(FastNbtError::UnsupportedExtension(
                "File without extension".into(),
            )),
        }
    }

    /// Retrieves the Minecraft version.
    pub fn get_mc_version(&self) -> String {
        self.version.clone()
    }

    /// Serializes the first tag compound to a JSON file.
    pub fn to_json<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), FastNbtError> {
        self.tag_compounds_list
            .get(0)
            .ok_or_else(|| FastNbtError::NbtParse("No tag compounds available".into()))?
            .to_json(path)?;
        Ok(())
    }

    /// Searches for blocks based on resource locations.
    pub fn search_blocks(&self, block_resource_locations: Vec<String>) -> HashMap<String, Vec<blocks::MinecraftBlock>> {
        chunk_format::inspect_chunks(block_resource_locations, &self.tag_compounds_list)
    }

    /// Searches for a compound by key, optionally stopping at the first match.
    pub fn search_compound(&self, key: &str, stop_at_first: bool) -> (bool, Vec<&nbt_tag::NbtTagCompound>) {
        let mut result_list = Vec::new();

        for tag_compound in &self.tag_compounds_list {
            let compound_found = self.recursive_compound_search(tag_compound, &mut result_list, key, stop_at_first);

            if compound_found && stop_at_first {
                return (true, result_list);
            }
        }

        (result_list.is_empty(), result_list)
    }

    /// Recursively searches for compounds matching the key.
    fn recursive_compound_search<'a>(
        &'a self,
        tag_compound: &'a nbt_tag::NbtTagCompound,
        result_list: &mut Vec<&'a nbt_tag::NbtTagCompound>,
        key: &str,
        stop_at_first: bool,
    ) -> bool {
        if tag_compound.name == key {
            result_list.push(tag_compound);
            return true;
        }

        for (_, value) in &tag_compound.values {
            match value.ty() {
                nbt_tag::NbtTagType::Compound => {
                    if let Some(compound) = value.compound_as_ref() {
                        if self.recursive_compound_search(compound, result_list, key, stop_at_first) && stop_at_first {
                            return true;
                        }
                    }
                }
                nbt_tag::NbtTagType::List => {
                    if let Some(list) = value.list_as_ref() {
                        for item in &list.values {
                            if item.ty() == nbt_tag::NbtTagType::Compound {
                                if let Some(compound) = item.compound_as_ref() {
                                    if self.recursive_compound_search(compound, result_list, key, stop_at_first) && stop_at_first {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }
}

/// Python bindings module for FastNBT.
#[pymodule]
fn fastnbt(py: Python, m: &PyModule) -> PyResult<()> {
    // Initialize Python logging
    init_pyo3_log();

    // Register Python classes
    m.add_class::<PyMcWorldDescriptor>()?;
    m.add_class::<PyNbtTag>()?;
    m.add_class::<blocks::MinecraftBlock>()?;
    m.add_class::<blocks::Coordinates>()?;
    m.add_class::<blocks::MinecraftChunk>()?;

    // Register Python functions
    m.add_function(wrap_pyfunction!(load_binary, m)?)?;
    m.add_function(wrap_pyfunction!(py_log, m)?)?;

    Ok(())
}

/// Logs a message from Python using Rust's `log` crate.
#[pyfunction]
fn py_log(message: String) {
    info!("{}", message);
}

/// Loads a binary Minecraft file and returns a PyMcWorldDescriptor.
#[pyfunction]
fn load_binary(input_path: String) -> PyResult<PyMcWorldDescriptor> {
    let path_buf = PathBuf::from(input_path);
    let mc_world = McWorldDescriptor::new(path_buf).map_err(|e| {
        PyIOError::new_err(format!(
            "Failed to load binary: {}",
            e.to_string()
        ))
    })?;
    PyMcWorldDescriptor::new(mc_world).map_err(|e| PyIOError::new_err(e.to_string()))
}

/// Represents a Python-exposed NBT tag.
#[pyclass(get_all)]
#[derive(Clone, Debug)]
pub struct PyNbtTag {
    pub python_dict: Py<PyDict>,
}

impl PyNbtTag {
    /// Creates a new PyNbtTag from an NbtTag.
    pub fn new(nbt_tag: &nbt_tag::NbtTag) -> Self {
        let python_dict = Self::to_python_dictionary(nbt_tag);
        Self { python_dict }
    }

    /// Converts an NbtTag to a Python dictionary.
    fn to_python_dictionary(nbt_tag: &nbt_tag::NbtTag) -> Py<PyDict> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);

            match nbt_tag.ty() {
                nbt_tag::NbtTagType::End => {
                    dict.set_item("END_TAG", 0).unwrap();
                }
                nbt_tag::NbtTagType::Byte => {
                    let tag = nbt_tag.byte().unwrap();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::Short => {
                    let tag = nbt_tag.short().unwrap();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::Int => {
                    let tag = nbt_tag.int().unwrap_or_default();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::Long => {
                    let tag = nbt_tag.long().unwrap();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::Float => {
                    let tag = nbt_tag.float().unwrap();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::Double => {
                    let tag = nbt_tag.double().unwrap();
                    dict.set_item(tag.name, tag.value).unwrap();
                }
                nbt_tag::NbtTagType::ByteArray => {
                    let tag = nbt_tag.byte_array().unwrap();
                    dict.set_item(tag.name, tag.values.clone()).unwrap();
                }
                nbt_tag::NbtTagType::String => {
                    let tag = nbt_tag.string().unwrap();
                    dict.set_item(tag.name, tag.value.clone()).unwrap();
                }
                nbt_tag::NbtTagType::List => {
                    let tag = nbt_tag.list().unwrap();
                    let py_list = PyList::new(py, tag.values.iter().map(|v| PyNbtTag::new(v).python_dict));
                    dict.set_item(tag.name, py_list).unwrap();
                }
                nbt_tag::NbtTagType::Compound => {
                    let tag = nbt_tag.compound().unwrap();
                    let py_dict = PyDict::new(py);
                    for (key, value) in &tag.values {
                        let py_tag = PyNbtTag::new(value);
                        py_dict.set_item(key, py_tag.python_dict).unwrap();
                    }
                    dict.set_item(tag.name, py_dict).unwrap();
                }
                nbt_tag::NbtTagType::IntArray => {
                    let tag = nbt_tag.int_array().unwrap();
                    dict.set_item(tag.name, tag.values.clone()).unwrap();
                }
                nbt_tag::NbtTagType::LongArray => {
                    let tag = nbt_tag.long_array().unwrap();
                    dict.set_item(tag.name, tag.values.clone()).unwrap();
                }
            }

            dict.into()
        })
    }
}
