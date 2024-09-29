// blocks/mod.rs

//! Module for representing Minecraft blocks and their related data structures.

use pyo3::prelude::*;
use std::collections::HashMap;

/// Represents a Minecraft block with its properties.
#[pyclass]
#[derive(Clone, Debug)]
pub struct MinecraftBlock {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub coord: Coordinates,

    #[pyo3(get, set)]
    pub chunk: MinecraftChunk,

    #[pyo3(get, set)]
    pub properties: HashMap<String, String>,
}

#[pymethods]
impl MinecraftBlock {
    /// Creates a new MinecraftBlock.
    #[new]
    pub fn new(
        name: String,
        coord: Vec<i32>,
        chunk_coord: Vec<i32>,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            coord: Coordinates::new(coord),
            chunk: MinecraftChunk::new(chunk_coord),
            properties,
        }
    }
}

/// Represents 3D coordinates.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Coordinates {
    #[pyo3(get, set)]
    pub x: i32,

    #[pyo3(get, set)]
    pub y: i32,

    #[pyo3(get, set)]
    pub z: i32,
}

#[pymethods]
impl Coordinates {
    /// Creates new Coordinates from a vector.
    #[new]
    pub fn new(coord: Vec<i32>) -> Self {
        assert!(
            coord.len() == 3,
            "Coordinates must have exactly three elements: x, y, z"
        );
        Self {
            x: coord[0],
            y: coord[1],
            z: coord[2],
        }
    }
}

/// Represents a Minecraft chunk.
#[pyclass]
#[derive(Clone, Debug)]
pub struct MinecraftChunk {
    #[pyo3(get, set)]
    pub coord: Coordinates,
}

#[pymethods]
impl MinecraftChunk {
    /// Creates a new MinecraftChunk from a coordinate vector.
    #[new]
    pub fn new(coord: Vec<i32>) -> Self {
        assert!(
            coord.len() == 3,
            "Chunk coordinates must have exactly three elements: x, y, z"
        );
        Self {
            coord: Coordinates::new(coord),
        }
    }
}

/// Represents a batch of Minecraft blocks.
pub struct BlockBatch {
    pub blocks: Vec<MinecraftBlock>,
}
