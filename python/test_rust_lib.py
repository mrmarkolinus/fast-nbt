import rnbt as rnbt # This should match the name of your .so or .pyd file.
#import logging

#logging.basicConfig(level=logging.INFO)
mc_binary = rnbt.load_binary('tests/resources/test_world/r.-1.0.mca')
redstone = mc_binary.search_blocks(['minecraft:repeater', 'minecraft:lever', 'minecraft:iron_block', 'minecraft:piston'])

print(redstone['minecraft:repeater'][0].x)
print(redstone['minecraft:repeater'][0].y)
print(redstone['minecraft:repeater'][0].z)

# tag_found, blocks_list_tag = mc_binary.search_compound('block_states')

# if tag_found:
#     for block_tag in blocks_list_tag:
#         print(block_tag)
# else:
#     print('Tag not found')
    