//! Tests the library using the `bigtest.nbt` file provided
//! by Mojang.
use rnbt::McWorldDescriptor;
use std::path::PathBuf;

#[test]
fn region_search_blocks() {
    
    //let mut path = PathBuf::from("C:/MultiMC/MultiMC/instances/Fabulously.Optimized.MC.1.20.1.auto-update/.minecraft/saves/fast-nbt test/");
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/resources/test_world/r.-1.0.mca");

    let mc_world = McWorldDescriptor::new(path);

    // Confirm that values are correct
    let mc_world = mc_world.unwrap();
    
    let block_positions = mc_world.search_blocks(vec![
                                                                        "minecraft:repeater".to_string(), 
                                                                        "minecraft:lever".to_string(), 
                                                                        "minecraft:iron_block".to_string(),
                                                                        "minecraft:piston".to_string(),]);
    print!("End");
}
