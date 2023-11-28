import rnbt as rnbt # This should match the name of your .so or .pyd file.

#logging.basicConfig(level=logging.INFO)
mc_binary = rnbt.load_binary('tests/resources/r.0.0.mca')
#mc_binary = rnbt.load_binary('tests/outputs/output_bt_py.json')
#err = mc_binary.to_json('tests/outputs/output_bt_py.json')
mc_version = mc_binary.get_mc_version()

tag_found, blocks_list_tag = mc_binary.search_compound('block_states')

if tag_found:
    for block_tag in blocks_list_tag:
        print(block_tag)
else:
    print('Tag not found')
    