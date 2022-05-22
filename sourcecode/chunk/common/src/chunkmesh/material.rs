use std::collections::HashMap;

use gdnative::{api::SpatialMaterial, prelude::*};
use lazy_static::lazy_static;

use crate::{block::BLOCKS_JSON, prelude::BlockID};

/// Loads the texture for `block_id` from the game assets.
fn get_albedo_texture(block_id: BlockID) -> Option<Ref<Texture, Shared>> {
    let tex_path = format!("res://assets/textures/blocks/{}.png", block_id);
    let resource_loader = ResourceLoader::godot_singleton();
    let texture: Ref<Texture, Shared> = resource_loader.load(tex_path, "", false)?.cast().unwrap();
    unsafe { texture.assume_safe() }.set_flags(Texture::FLAGS_DEFAULT ^ Texture::FLAG_FILTER);
    Some(texture)
}

/// Creates the block-type-specific material for `block_id`.
///
/// This uses the resource file for `block_id` in `assets/materials`
/// if it exists, otherwise it creates a new `SpatialMaterial`.
fn create_material(block_id: BlockID) -> Option<Ref<SpatialMaterial, Shared>> {
    let resource_loader = ResourceLoader::godot_singleton();
    let material_path = format!("res://assets/materials/{}.tres", block_id); // HARDCODED
    let material = if resource_loader.exists(&material_path, "") {
        // Prevent Godot error spam by checking for the material before attempting
        // to load it.
        resource_loader.load(material_path, "", false)
    } else {
        None
    };
    Some(match material {
        Some(material) => material.cast().unwrap(),
        None => {
            // Make a new material containing the block's texture.
            let material = SpatialMaterial::new();
            material.set_texture(
                SpatialMaterial::TEXTURE_ALBEDO,
                get_albedo_texture(block_id)?,
            );
            material.set_flag(SpatialMaterial::FLAG_ALBEDO_FROM_VERTEX_COLOR, true);
            material.set_flag(SpatialMaterial::FLAG_DISABLE_AMBIENT_LIGHT, true);
            material.upcast::<SpatialMaterial>().into_shared()
        }
    })
}

lazy_static! {
    pub static ref MATERIALS: HashMap<BlockID, Ref<SpatialMaterial, Shared>> = {
        println!("Loading materials...");
        // TODO: There's some code duplication with `block` here.
        let mut game_dict: HashMap<String, serde_json::Value> =
            serde_json::from_str(BLOCKS_JSON).unwrap();
        let blocks: HashMap<String, serde_json::Map<String, serde_json::Value>> =
            serde_json::from_value(game_dict.remove("blocks").unwrap()).unwrap();
        let materials: HashMap<BlockID, Ref<SpatialMaterial, Shared>> = blocks
            .into_iter()
            .filter_map(|(_, block)| {
                let id: BlockID = block.get("id").unwrap().as_u64().unwrap() as BlockID;
                Some((id, create_material(id)?))
            })
            .collect();
        println!("Loaded {} materials.", materials.len());
        materials
    };
}
