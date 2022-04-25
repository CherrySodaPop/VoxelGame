extends Node

var blocks_raw = {};
var ids_to_names = {};

func _ready():
	var blocks_file = File.new();
	blocks_file.open("res://assets/blocks.json", File.READ);
	blocks_raw = parse_json(blocks_file.get_as_text())["blocks"];
	populate_id_table()

func get_block(block_name: String):
	var block_data = BlockData.new();
	var raw_block = blocks_raw[block_name]
	block_data.id = raw_block["id"]
	block_data.name = raw_block["name"]
	block_data.durability = raw_block["durability"];
	return block_data
	
func get_block_by_id(id: int):
	return get_block(ids_to_names[id])

func populate_id_table():
	for raw_block in blocks_raw:
		ids_to_names[int(blocks_raw[raw_block]["id"])] = raw_block
