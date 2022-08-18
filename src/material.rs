use bevy::asset::{Handle, LoadContext, LoadedAsset};
use bevy::pbr::{AlphaMode, StandardMaterial};
use dot_vox::Material;

// constants used in magicavoxel's material property map
const MATERIAL_TYPE: &str = "_type";
const MATERIAL_ALPHA: &str = "_alpha";
const MATERIAL_GLASS: &str = "_glass";
const MATERIAL_EMIT: &str = "_emit";

pub(crate) fn load_material(ctx: &mut LoadContext, palette: &[[f32; 4]], materials: &[Material]) -> Handle<StandardMaterial> {
    if !materials.is_empty() {
        assert_eq!(palette.len(), materials.len(), "Expected a material for every palette color");
    }

    let (opaque, _emissive) = get_properties(palette, materials);

    ctx.set_labeled_asset("material", LoadedAsset::new(StandardMaterial {
        // TODO support emmission again
        // emissive: if emissive { Color::WHITE } else { Color::BLACK },
        alpha_mode: if opaque { AlphaMode::Opaque } else { AlphaMode::Blend },
        ..StandardMaterial::default()
    }))
}

fn get_properties(palette: &[[f32; 4]], materials: &[Material]) -> (bool, bool) {
    let mut opaque = true;
    let mut emissive = false;

    for i in 0..palette.len() {
        if let Some(material) = materials.get(i) {
            if material.properties.get(MATERIAL_TYPE).filter(|x| *x == MATERIAL_GLASS).is_some() {
                if let Some(_) = material.properties.get(MATERIAL_ALPHA).and_then(|x| x.parse::<f32>().ok()) {
                    opaque = false;
                }
            }
            if material.properties.get(MATERIAL_TYPE).filter(|x| *x == MATERIAL_EMIT).is_some() {
                if let Some(_) = material.properties.get(MATERIAL_EMIT).and_then(|x| x.parse::<f32>().ok()) {
                    emissive = true;
                }
            }
        }
    }

    (opaque, emissive)
}
