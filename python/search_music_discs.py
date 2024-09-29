import fastnbt as fnbt # This should match the name of your .so or .pyd file.
#import logging
#logging.basicConfig(level=logging.INFO)
#mc_binary = rnbt.load_binary('tests/resources/test_world/r.-1.0.mca')

#READ THE ENTIRE TEST WORLD AND SEARCH FOR REDSTONE COMPONENTS
mc_binary = fnbt.load_binary('C:/MultiMC/MultiMC/instances/Hermitcraft S9/.minecraft/saves/DO Standalone Test')
tag_found, blocks_list_tag = mc_binary.search_compound('minecraft:music_disc_wait')

if tag_found:
    for block_tag in blocks_list_tag:
        print(block_tag)
else:
    print('Tag not found')
    