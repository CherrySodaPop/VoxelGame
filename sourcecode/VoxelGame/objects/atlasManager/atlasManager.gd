extends Node

export(Resource) var atlas_config

func stitch_texture_atlas() -> Texture:
	print("Stitching texture atlas...")
	var total_ids = atlas_config.end_id - atlas_config.start_id
	var texture = ImageTexture.new()
	var image = Image.new()
	texture.create(total_ids * atlas_config.texture_width, 16, Image.FORMAT_RGBA8)
	image.create(
		total_ids * atlas_config.texture_width, 16, false, Image.FORMAT_RGBA8
	)
	for block_id in range(atlas_config.start_id, atlas_config.end_id + 1):
		var load_path = "res://assets/textures/blocks/%d.png" % block_id # HARDCODED
		if not ResourceLoader.exists(load_path):
			# Prevents error spam.
			continue;
		var block_img = load(load_path).get_data()
		block_img.convert(Image.FORMAT_RGBA8)
		var atlas_coords = Vector2(
			(block_id - atlas_config.start_id) * atlas_config.texture_width,
			0
		)
		image.blit_rect(
			block_img,
			Rect2(0, 0, atlas_config.texture_width, atlas_config.texture_width),
			atlas_coords
		)
	texture.create_from_image(
		image,
		ImageTexture.FLAGS_DEFAULT ^ ImageTexture.FLAG_FILTER)
	$ColorRect.rect_size.x = image.get_size().x
	$TextureRect.texture = texture
	$Label.text = "%s Texture atlas:" % str(image.get_size())
	return texture
