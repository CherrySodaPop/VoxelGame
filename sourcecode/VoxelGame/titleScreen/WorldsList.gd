extends ItemList

func update_list(worlds: Array):
	items = []
	visible = bool(len(worlds))
	for world in worlds:
		add_item(world)
