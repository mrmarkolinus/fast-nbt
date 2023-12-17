// ## Author
// - mrmarkolinus
//
// ## Date
// - 2023-12-17
//
// ## File Version
// - 1.0.0
//
// ## Changelog
// - 1.0.0: Initial version

use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
pub struct MinecraftBlock{
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub coord: Coordinates,
    #[pyo3(get, set)]
    pub chunk: MinecraftChunk,
    #[pyo3(get, set)]
    pub properties: HashMap<String, String>
}

#[pymethods]
impl MinecraftBlock {
    #[new]
    pub fn new (name: String, coord: Vec<i32>, chunk_coord: Vec<i32>, properties: HashMap<String, String>) -> Self {
        Self {
            name,
            coord: Coordinates::new(coord),
            chunk: MinecraftChunk::new(chunk_coord),
            properties
        }
    }
}


#[pyclass]
#[derive(Clone)]
pub struct Coordinates
{
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
    #[pyo3(get, set)]
    pub z: i32,
}

#[pymethods]
impl Coordinates {
    #[new]
    pub fn new (coord: Vec<i32>) -> Self {
        Self {
            x : coord[0],
            y : coord[1],
            z : coord[2],
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct MinecraftChunk
{
    pub coord: Coordinates,
}

#[pymethods]
impl MinecraftChunk {
    #[new]
    pub fn new (coord: Vec<i32>) -> Self {
        Self {
            coord: Coordinates::new(coord),
        }
    }
}




pub struct BlockBatch {
    pub blocks: Vec<MinecraftBlock>,
}

