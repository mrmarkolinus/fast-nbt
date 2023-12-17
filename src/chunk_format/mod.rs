use crate::nbt_tag;
use crate::blocks;

use std::collections::{HashMap, HashSet};

/// Inspects Minecraft chunks and extracts block positions based on resource locations.
/// 
/// This function parses NBT (Named Binary Tag) data of Minecraft chunks to identify and return 
/// the positions of specific blocks. It is useful for analyzing Minecraft game data, especially 
/// for modding or data analysis purposes.
/// 
/// # Arguments
/// 
/// * `block_resource_location` - Vec<String>: A vector of strings representing the resource 
///   locations of blocks to be inspected.
/// * `tag_compounds_list` - &Vec<nbt_tag::NbtTagCompound>: A reference to a vector of 
///   NbtTagCompound, representing the NBT data of chunks.
/// 
/// # Returns
/// 
/// HashMap<String, Vec<blocks::Coordinates>>: A HashMap where each key is a resource location 
/// string, and the value is a vector of Coordinates structs representing the positions of 
/// the blocks in the Minecraft world.
/// 
pub fn inspect_chunks<'a>(block_resource_location: Vec::<String>, tag_compounds_list: &'a Vec<nbt_tag::NbtTagCompound>) -> HashMap::<String, Vec::<blocks::Coordinates>> {
    // Refer to https://minecraft.fandom.com/wiki/Chunk_format to see how a block is saved in a chunk
    //sections (TAG List)
    // block_states (TAG Compound)
    // -- palette (TAG List)
    // ---- block (TAG Compound)
    // ------ Name (TAG String)
    let mut blocks_positions_list = HashMap::<String, Vec::<blocks::Coordinates>>::new();

    for tag_compound in tag_compounds_list.iter() {
        let mut chunk_pos = get_chunk_coordinates(tag_compound);
        
        if let Some(sections_tag) = tag_compound.values.get("sections") {
            if let Some(sections_list) = sections_tag.list_as_ref(){
                for sections in sections_list.values.iter() {
                    if let Some(block_states_tag) = find_block_states_in_section(sections) {
                        //TODO: replace unwraps
                        let subchunk_y_pos = sections.compound_as_ref().unwrap().values.get("Y").unwrap().byte().unwrap().value as i32;
                        // The y position got from get_chunk_coordinates is always -4, since the chunk always starts at -4 * 16 = -64
                        // what we need is the actual subchunk position
                        chunk_pos.y = subchunk_y_pos;
                        _ = get_absolute_blocks_positions(block_states_tag, &block_resource_location, &chunk_pos, &mut blocks_positions_list);
                    }
                }
            }
        }
    }

    blocks_positions_list

}

/// Calculates the absolute positions of blocks within Minecraft chunks.
///
/// Analyzes a block state NBT tag and identifies the absolute positions of specified blocks within a chunk. 
/// This function is integral for understanding the spatial arrangement of blocks in Minecraft's chunk data structure.
///
/// # Arguments
///
/// * `block_states_tag` - A reference to the NbtTag, representing the block states in a Minecraft chunk.
/// * `block_resource_location` - A reference to a vector of strings, each string representing a Minecraft block resource location.
/// * `chunk_pos` - A reference to the coordinates of the chunk being inspected.
/// * `blocks_positions_list` - A mutable reference to a HashMap where keys are block names (String) and values are vectors of block coordinates (Coordinates).
///
/// # Returns
///
/// Returns `true` if the function successfully finds and processes the block positions, `false` otherwise.
///
/// # Details
///
/// The function iterates through the block states, matching them against the specified resource locations. 
/// It decodes the data array associated with each block's state to determine the exact position of each block within the chunk.
/// This process involves interpreting the palette list and the data array in accordance with the Minecraft chunk format.
/// The function updates `blocks_positions_list` with the absolute positions of the found blocks.
pub fn get_absolute_blocks_positions<'a>   (block_states_tag: &nbt_tag::NbtTag, 
                                            block_resource_location: & 'a Vec::<String>, 
                                            chunk_pos: &blocks::Coordinates, 
                                            blocks_positions_list: & 'a mut HashMap::<String, Vec::<blocks::Coordinates>>) -> bool {
    /* #10: Find palette TAG list in block states following the format https://minecraft.fandom.com/wiki/Chunk_format
    * block_states (TAG Compound)
    * -- palette (TAG List)
    */

    let mut block_found = false;
    let (palette_list_option, blocks_data_array_option) = find_palette_in_block_states(block_states_tag);

    match palette_list_option {
        Some(palette_list) => {
            let (unique_set_created, searched_blocks_palette_ids) = create_unique_palette_id_set(&palette_list, block_resource_location);

            if unique_set_created {
                match blocks_data_array_option {
                    /* #30: if the searched block was found scan the data array associated to the palette.
                    * A data array is a 64bit unsing integer array with a specific format (see Chunk_format)
                    * A data array needs to contain all the blocks in the subchunk (section), which is 16x16x16
                    * The blocks are saved as id referincing the palette and compressed in 64bit unsigned integer
                    * Example:  palette list {minecraft:bedrock, minecreat:stone, minecraft:dirt}
                    *           we need 2 bits to represent the 3 possible blocks in the section (0, 1, 2)
                    *           the chunk file format specifies that min 4 bits must be used, so we get 4 bits.
                    *           data array dec:     1180177 
                    *           data array bin:     0000 0000 0001 0010 0000 0010 0001 0001
                    *           data array palette: bedrock bedrock stone dirt bedrock dirt stone stone
                    * For more details refer to https://minecraft.fandom.com/wiki/Chunk_format
                    */
                    Some(blocks_data_array) => { 
                        let data_index_bit_size = get_palette_id_size_in_bit(palette_list);

                        let mut subchunk_x_pos = 0;
                        let mut subchunk_y_pos = 0;
                        let mut subchunk_z_pos = 0;  

                        for blocks_data in blocks_data_array {
                            let palette_ids = get_palette_ids_from_data_array_element(blocks_data.clone(), data_index_bit_size);
                        
                            /* #40: get the block position in the subchunk 
                            * block position is a tridimensional coordinate x,y,z. The blocks are stored with YZX order
                            * X increases each block
                            * Z increases each 16 blocks
                            * Y increases each 16x16 = 256 blocks
                            */                      
                            for palette_id in palette_ids {
                                //we are interested only in the searched blocks
                                for (block_name, block_palette_ids) in searched_blocks_palette_ids.iter() {
                                    if block_palette_ids.contains(&palette_id) {

                                        if !blocks_positions_list.contains_key(block_name) {
                                            blocks_positions_list.insert(block_name.clone(), vec![]);
                                        }
                                        
                                        if let Some(current_block_positions_list) = blocks_positions_list.get_mut(block_name) {
                                            current_block_positions_list.push(blocks::Coordinates::new(
                                                [(chunk_pos.x * 16) + subchunk_x_pos, 
                                                        ((chunk_pos.y * 16) + subchunk_y_pos), 
                                                        (chunk_pos.z * 16) + subchunk_z_pos].to_vec()));
                                        }
                                    }
                                    advance_block_position(&mut subchunk_x_pos, &mut subchunk_y_pos, &mut subchunk_z_pos);
                                }                 
                            }
                        }
                    },
                    None => {
                        //TODO
                    }
                }
            }
                        
        },
        None => {
            
        }
    }

    block_found
}

/// Advances the block position in a Minecraft chunk.
///
/// Increments the coordinates (x, y, z) to the next block position in the chunk, following Minecraft's storage order.
/// This function is essential for iterating through blocks in a chunk in the correct sequence.
///
/// # Arguments
///
/// * `x_pos` - A mutable reference to the x-coordinate of the current block position.
/// * `y_pos` - A mutable reference to the y-coordinate of the current block position.
/// * `z_pos` - A mutable reference to the z-coordinate of the current block position.
///
/// # Behavior
///
/// The function updates the x, y, and z coordinates to point to the next block in the chunk. 
/// Minecraft chunks store blocks in a specific order: x-coordinate increments first, 
/// then z-coordinate after every 16 blocks, and finally y-coordinate after every 256 blocks (16x16).
/// When the x-coordinate reaches 15 (the end of a row), it resets to 0, and the z-coordinate is incremented.
/// Similarly, when both x and z-coordinates reach 15, they reset to 0, and the y-coordinate is incremented.
pub fn advance_block_position(x_pos: &mut i32, y_pos: &mut i32, z_pos: &mut i32) {
    if *x_pos == 15 {
        if *z_pos == 15 {
            *y_pos += 1;
            *z_pos = 0;
            *x_pos = 0;
        }
        else {
            *z_pos += 1;
            *x_pos = 0;
        }    
    }
    else {
        *x_pos += 1;
    } 
}

/// Creates a unique set of palette IDs for specified blocks in a Minecraft chunk.
///
/// Scans through the block palette list and compiles unique palette IDs for each block specified in `block_resource_location`.
/// This function helps in differentiating blocks with the same name but different orientations or states, which is common in Minecraft.
///
/// # Arguments
///
/// * `palette_list` - A reference to the NbtTagList representing the palette list of a Minecraft chunk.
/// * `block_resource_location` - A reference to a vector of strings, each representing a specific block's resource location.
///
/// # Returns
///
/// Returns a tuple containing:
/// * A boolean indicating if at least one unique set was created.
/// * A HashMap where keys are block names (String) and values are sets of palette IDs (HashSet<u32>).
///
/// # Details
///
/// The function iterates over each block name in `block_resource_location`, scanning `palette_list` to find matching blocks.
/// Each match is identified by a unique palette ID (index in the palette list), which is added to a HashSet.
/// This process helps in tracking all variations of a block, which may have different IDs despite having the same name.
pub fn create_unique_palette_id_set<'a>(palette_list: &nbt_tag::NbtTagList, block_resource_location: & 'a Vec::<String>) -> (bool, HashMap<String, HashSet<u32>>){
    /*Some blocks may have different palette ids with same names (for example a repeater oriented in different ways)*/
    
    /* Init the data structure to contain multiple blocks finding */
    let mut searched_blocks_palette_ids = HashMap::<String, HashSet<u32>>::new();

    let mut unique_set_created = false;
    
    
    for block_name in block_resource_location.iter() {
        let mut palette_current_index = 0;
        let mut block_unique_set = HashSet::new();
        for blocks in palette_list.values.iter() {
            /* #20: scan every block in the palette and check if the name is the one we are looking for
            * -- palette (TAG List)
            * ---- block (TAG Compound)
            * ------ Name (TAG String)
            */
            if find_block_name_in_palette(blocks, block_name) {
                block_unique_set.insert(palette_current_index);
                
                if !unique_set_created {
                    unique_set_created = true;
                }
                
            }
            palette_current_index += 1;
        }
        searched_blocks_palette_ids.insert(block_name.clone(), block_unique_set);
    }

    (unique_set_created, searched_blocks_palette_ids)
}


/// Calculates the size of palette IDs in bits for Minecraft block data.
///
/// This function is used to determine the number of bits needed to represent palette IDs in 
/// Minecraft's chunk data. Palette IDs are used in Minecraft to efficiently store information 
/// about different types of blocks in a chunk.
///
/// # Arguments
///
/// * `palette_size` - usize: The size of the palette, which is the number of different block 
///   types in the chunk.
///
/// # Returns
///
/// usize: The size in bits required to represent a palette ID, considering the given palette size.
pub fn get_palette_id_size_in_bit(palette_list: &nbt_tag::NbtTagList) -> u32 {
    /* the number of palettes in the section determines the number of bits used for the indexes in data
    * the indexes in data are n bits long, where n is the number needed to represent all the palettes (log2(n_palettes))
    * minimum 4 bits
    * example: 4 palettes = 2 bits needed to represent them. 4 used
    * example: 36 palettes = 6 bits needed to represent them. 6 used
        */
    let num_palette_in_section = palette_list.values.len() as u32;
    let num_bits = (std::mem::size_of_val(&num_palette_in_section) * 8) as u32;
    
    //fast log2 function. index of the palette start from 0
    let mut data_index_bit_size = num_bits - (num_palette_in_section - 1).leading_zeros();
    
    //as per Chunk file specification, the indexes cannot be shorter than 4bits
    if data_index_bit_size < 4 {
        data_index_bit_size = 4;
    }

    data_index_bit_size

}

/// Extracts palette IDs from a data array element in Minecraft chunk data.
///
/// This function decodes and retrieves palette IDs from a given data array element. These IDs 
/// are used in Minecraft to reference different types of blocks within the chunk's palette. 
/// The function aids in parsing and understanding the structure of Minecraft chunk data.
///
/// # Arguments
///
/// * `data_array_element` - u64: A 64-bit unsigned integer representing a data array element 
///   from Minecraft chunk data.
/// * `palette_id_size_in_bit` - usize: The size in bits of each palette ID within the data array.
/// * `offset_in_bit` - usize: The bit offset within the data array element from which the palette 
///   ID should be extracted.
///
/// # Returns
///
/// u64: The extracted palette ID from the specified data array element.
pub fn get_palette_ids_from_data_array_element(data_array_element : i64, index_size_in_bit : u32) -> Vec<u32> {

    /* given a 64bit unsigned integer it splits it into n indexes and n values.
        * where n is the number of indexes that fits into a 64bit unsigned integer (calculate with get_palette_id_size_in_bit)
        * The value represent the block palette id in the palette TAG list
        * As per Chunk file specification, the indexes cannot be split between more data elements, so some bits may be unused
        * Example: if index_size_in_bit is 4, there are 16 indexes
        * Example: if index_size in bit is 6, there are 10 indexes, the 4 most significant bits are not used
        */

    let bit_mask = (0xFFFFFFFFFFFFFFFF as u64 >> (64 - index_size_in_bit)) as u64;
    let indexes_in_data_element = (64 / index_size_in_bit) as u32;
    let mut palette_id_array: Vec<u32> = Vec::with_capacity(indexes_in_data_element as usize);

    for element_data_index in 0..indexes_in_data_element {
        let shift_amount = element_data_index * index_size_in_bit;
        let block_palette_id = (data_array_element as u64 >> shift_amount) & bit_mask;

        palette_id_array.push(block_palette_id as u32);
    }

    palette_id_array
}

/// Retrieves the coordinates of a chunk from its NBT tag compound.
///
/// This function parses the NBT (Named Binary Tag) data of a Minecraft chunk to extract its 
/// coordinates. Chunk coordinates are essential for identifying the location of chunks in the 
/// Minecraft world, especially for tasks like map rendering or data analysis.
///
/// # Arguments
///
/// * `chunk_compound` - &nbt_tag::NbtTagCompound: A reference to the NbtTagCompound representing 
///   the NBT data of a single Minecraft chunk.
///
/// # Returns
///
/// blocks::Coordinates: A Coordinates struct representing the x and z coordinates of the chunk.
pub fn get_chunk_coordinates(chunk_compound: &nbt_tag::NbtTagCompound) -> blocks::Coordinates {

    let mut result: blocks::Coordinates = blocks::Coordinates::new(vec![0, 0, 0]);
    
    if let Some(x_coord_tag) = chunk_compound.values.get("xPos") {
        if let Some(x_coord) = x_coord_tag.int() {
            result.x = x_coord.value;
        }
        
    }

    if let Some(y_coord_tag) = chunk_compound.values.get("yPos") {
        if let Some(y_coord) = y_coord_tag.int() {
            result.y = y_coord.value;
        }
        
    }

    if let Some(z_coord_tag) = chunk_compound.values.get("zPos") {
        if let Some(z_coord) = z_coord_tag.int() {
            result.z = z_coord.value;
        }
        
    }
    
    result

}

/// Finds and returns the block states in a given section of a Minecraft chunk.
///
/// Examines a provided NBT tag to locate the "block_states" compound, which represents the state of each block in a Minecraft chunk section.
/// This function is essential for accessing detailed block information within a chunk.
///
/// # Arguments
///
/// * `block_states_tag` - A reference to the NbtTag, representing a section of a Minecraft chunk.
///
/// # Returns
///
/// Returns an `Option` containing a reference to the 'block_states' NbtTag if found, otherwise `None`.
///
/// # Details
///
/// The function first checks if the provided NbtTag is a compound tag. If it is, the function then looks for the "block_states" key within the compound.
/// If the "block_states" compound is found, a reference to it is returned. If not found, or if the initial NbtTag is not a compound tag, the function returns `None`.
/// This approach ensures that only relevant and existing block state information is retrieved, avoiding potential errors or misinterpretations of the chunk data.
pub fn find_block_states_in_section<'a>(block_states_tag: & 'a nbt_tag::NbtTag) -> Option<& 'a nbt_tag::NbtTag> {    

    if let Some(block_states_compound) = block_states_tag.compound_as_ref() {
        if let Some(block_states) = block_states_compound.values.get("block_states") {
            Some(block_states)
        }
        else {
            None
        }
    }
    else {
        None
    }
}

/// Retrieves the palette and data array from the block states of a Minecraft chunk.
///
/// Analyzes a block states NBT tag to extract the palette list and the corresponding data values. 
/// This function plays a crucial role in decoding the block information within a Minecraft chunk.
///
/// # Arguments
///
/// * `block_states_tag` - A reference to the NbtTag, representing the block states of a Minecraft chunk.
///
/// # Returns
///
/// Returns a tuple containing:
/// * An `Option` for a reference to the NbtTagList, representing the palette list of the chunk.
/// * An `Option` for a reference to a Vec of i64, representing the data array of the chunk.
///
/// # Details
///
/// The function first attempts to interpret the provided NbtTag as a compound tag.
/// If successful, it looks for the "palette" and "data" keys within this compound.
/// The "palette" key is expected to point to a list of block states, while the "data" key should point to a long array representing the data values of these states.
/// If either the palette list or the data array is not found, `None` is returned for the missing part.
/// This ensures a robust and error-tolerant approach to extracting essential block data from Minecraft chunk information.
pub fn find_palette_in_block_states<'a>(block_states_tag: & 'a nbt_tag::NbtTag) -> (Option<&'a nbt_tag::NbtTagList>, Option<&'a Vec::<i64>>) {
    
    //let mut data_values = &Vec::<i64>::new();

    if let Some(block_states_compound) = block_states_tag.compound_as_ref() {
        if let Some(palette_tag) = block_states_compound.values.get("palette") {
            if let Some(palette_list) = palette_tag.list_as_ref() {
                if let Some(data_values_tag) = block_states_compound.values.get("data") {
                    if let Some(data_values_taglong) = data_values_tag.long_array_as_ref() {

                        (Some(palette_list), Some(&data_values_taglong.values))
                    }
                    else {
                        (Some(palette_list), None)
                    }
                }
                else {
                    (Some(palette_list), None)
                }    
                
            }
            else {
                (None, None)
            }
        }
        else {
            (None, None)
        }
    }
    else {
        (None, None)
    }

}

/// Determines if a specified block name exists within a block tag in a Minecraft palette.
///
/// Searches within a given NbtTag (representing a block in a Minecraft palette) to find if it matches the specified block resource location.
/// This function is essential for identifying specific block types within the complex structure of a Minecraft chunk.
///
/// # Arguments
///
/// * `blocks_tag` - A reference to the NbtTag, representing a block in the Minecraft palette.
/// * `block_resouce_location` - A string slice representing the resource location of the block to find.
///
/// # Returns
///
/// Returns `true` if the block name matches the specified resource location, `false` otherwise.
///
/// # Details
///
/// The function first checks if the provided NbtTag is a compound tag. If it is, the function then looks for the "Name" key within the compound.
/// If the "Name" tag is found and matches the specified `block_resouce_location`, the function returns `true`.
/// This approach enables precise identification of blocks, accounting for the variety and complexity of block types in Minecraft.
pub fn find_block_name_in_palette(blocks_tag: &nbt_tag::NbtTag, block_resouce_location: &str) -> bool {
    
    let mut block_name_found = false;
    
    if let Some(block_compound) = blocks_tag.compound_as_ref() {
        if let Some(block_name_tag) = block_compound.values.get("Name") {
            if let Some(block_name) = block_name_tag.string() {
                if block_name.value == block_resouce_location {
                    block_name_found = true
                }
            }
        }
    }

    block_name_found
}
