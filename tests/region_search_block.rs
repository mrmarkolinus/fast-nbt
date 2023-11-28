//! Tests the library using the `bigtest.nbt` file provided
//! by Mojang.
use rnbt::McWorldDescriptor;
use std::path::PathBuf;

#[test]
fn read_region_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/r.0.0.mca");

    let mc_world = McWorldDescriptor::new(path);

    // Confirm that values are correct
    let mc_world = mc_world.unwrap();
    
    let mut compound_found = mc_world.search_block("minecraft:bedrock", true);
    assert_eq!(compound_found, true);
}
