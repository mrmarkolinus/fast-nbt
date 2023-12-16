use pyo3::prelude::*;

#[pyclass]
pub struct MinecraftBlock{
    pub name: String,
    pub coord: Coordinates,
    pub chunk: MinecraftChunk,
}

#[pyclass]
pub struct Coordinates
{
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[pyclass]
pub struct MinecraftChunk
{
    pub coord: Coordinates,
}

impl MinecraftBlock {
    pub fn new (name: String, coord: Vec<i32>, chunk_coord: Vec<i32>) -> Self {
        Self {
            name,
            coord: Coordinates::new(coord),
            chunk: MinecraftChunk::new(chunk_coord),
        }
    }
}

impl Coordinates {
    pub fn new (coord: Vec<i32>) -> Self {
        Self {
            x : coord[0],
            y : coord[1],
            z : coord[2],
        }
    }
}

impl MinecraftChunk {
    pub fn new (coord: Vec<i32>) -> Self {
        Self {
            coord: Coordinates::new(coord),
        }
    }
}

